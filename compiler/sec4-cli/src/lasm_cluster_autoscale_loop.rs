use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::lasm_cluster_accept_dispatch::LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH;
use crate::lasm_cluster_lifecycle::{
    prune_dead_lasm_cluster_workers, recover_lasm_cluster_min_workers,
    spawn_and_wait_lasm_cluster_worker,
};
use crate::lasm_cluster_runtime_config::{
    desired_lasm_cluster_instances, lasm_cluster_remaining_cooldown_ms,
    refresh_lasm_cluster_worker_ports_snapshot_if_changed,
};
use crate::{LasmClusterConfig, LasmClusterState};

pub(crate) struct LasmClusterAutoscaleLoopConfig {
    pub(crate) state: Arc<RwLock<LasmClusterState>>,
    pub(crate) shared_config: Arc<LasmClusterConfig>,
    pub(crate) active_connections: Arc<AtomicUsize>,
    pub(crate) saturation_events: Arc<AtomicUsize>,
    pub(crate) stop_flag: Arc<AtomicBool>,
    pub(crate) worker_ports_snapshot: Arc<ArcSwap<Vec<u16>>>,
    pub(crate) autoscale_last_desired_instances: Arc<AtomicUsize>,
    pub(crate) autoscale_last_saturation_events: Arc<AtomicUsize>,
    pub(crate) autoscale_last_dynamic_boost_step: Arc<AtomicUsize>,
    pub(crate) autoscale_scale_up_cooldown_remaining_ms: Arc<AtomicU64>,
    pub(crate) autoscale_scale_down_cooldown_remaining_ms: Arc<AtomicU64>,
    pub(crate) autoscale_enabled: bool,
    pub(crate) maintenance_interval_ms: u64,
    pub(crate) saturation_priority_interval_ms: u64,
}

pub(crate) fn spawn_lasm_cluster_autoscale_loop(
    config: LasmClusterAutoscaleLoopConfig,
) -> std::thread::JoinHandle<()> {
    let LasmClusterAutoscaleLoopConfig {
        state,
        shared_config,
        active_connections,
        saturation_events,
        stop_flag,
        worker_ports_snapshot,
        autoscale_last_desired_instances,
        autoscale_last_saturation_events,
        autoscale_last_dynamic_boost_step,
        autoscale_scale_up_cooldown_remaining_ms,
        autoscale_scale_down_cooldown_remaining_ms,
        autoscale_enabled,
        maintenance_interval_ms,
        saturation_priority_interval_ms,
    } = config;

    std::thread::spawn(move || {
        let autoscale_check_interval = Duration::from_millis(shared_config.autoscale_check_ms);
        let autoscale_scale_up_cooldown =
            Duration::from_millis(shared_config.autoscale_scale_up_cooldown_ms);
        let autoscale_scale_down_cooldown =
            Duration::from_millis(shared_config.autoscale_scale_down_cooldown_ms);
        let mut last_scale_up_at: Option<Instant> = None;
        let mut last_scale_down_at: Option<Instant> = None;
        let mut last_published_worker_ports = worker_ports_snapshot.load().as_ref().clone();
        let mut last_scale_eval_at = Instant::now()
            .checked_sub(autoscale_check_interval)
            .unwrap_or_else(Instant::now);
        while !stop_flag.load(Ordering::Relaxed) {
            let saturation_pending_before_sleep = saturation_events.load(Ordering::Relaxed);
            let sleep_ms = if autoscale_enabled && saturation_pending_before_sleep > 0 {
                saturation_priority_interval_ms
            } else {
                maintenance_interval_ms
            };
            std::thread::sleep(Duration::from_millis(sleep_ms));
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            let now = Instant::now();
            let mut workers_to_spawn_ports = Vec::new();
            let mut workers_to_stop = Vec::new();
            let mut skip_scale_actions = false;
            let mut cooldown_anchor_changed = false;
            {
                let mut state = match state.write() {
                    Ok(state) => state,
                    Err(_) => break,
                };
                prune_dead_lasm_cluster_workers(&mut state);
                recover_lasm_cluster_min_workers(&mut state, &shared_config, "worker recovery");
                refresh_lasm_cluster_worker_ports_snapshot_if_changed(
                    &state,
                    &worker_ports_snapshot,
                    &mut last_published_worker_ports,
                );
                if !autoscale_enabled {
                    autoscale_last_desired_instances.store(state.workers.len(), Ordering::Relaxed);
                    autoscale_last_saturation_events.store(0, Ordering::Relaxed);
                    autoscale_last_dynamic_boost_step.store(
                        shared_config.autoscale_scale_up_step.max(1),
                        Ordering::Relaxed,
                    );
                    autoscale_scale_up_cooldown_remaining_ms.store(0, Ordering::Relaxed);
                    autoscale_scale_down_cooldown_remaining_ms.store(0, Ordering::Relaxed);
                    skip_scale_actions = true;
                } else {
                    autoscale_scale_up_cooldown_remaining_ms.store(
                        lasm_cluster_remaining_cooldown_ms(
                            now,
                            last_scale_up_at,
                            shared_config.autoscale_scale_up_cooldown_ms,
                        ),
                        Ordering::Relaxed,
                    );
                    autoscale_scale_down_cooldown_remaining_ms.store(
                        lasm_cluster_remaining_cooldown_ms(
                            now,
                            last_scale_down_at,
                            shared_config.autoscale_scale_down_cooldown_ms,
                        ),
                        Ordering::Relaxed,
                    );
                    let saturation_events_pending = saturation_events.load(Ordering::Relaxed);
                    if now.duration_since(last_scale_eval_at) < autoscale_check_interval
                        && saturation_events_pending == 0
                    {
                        skip_scale_actions = true;
                    } else {
                        last_scale_eval_at = now;

                        let active = active_connections.load(Ordering::Relaxed);
                        let mut desired = desired_lasm_cluster_instances(
                            active,
                            shared_config.min_instances,
                            shared_config.max_instances,
                            shared_config.target_connections_per_instance,
                        );
                        let saturation_events = saturation_events.swap(0, Ordering::Relaxed);
                        let mut scale_up_step_budget = shared_config.autoscale_scale_up_step;
                        let current_workers = state.workers.len();
                        if saturation_events > 0 {
                            let saturation_batch_size = LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH;
                            debug_assert!(saturation_batch_size > 0);
                            let saturation_batches = saturation_events / saturation_batch_size
                                + usize::from(saturation_events % saturation_batch_size != 0);
                            let dynamic_boost_step = shared_config
                                .autoscale_saturation_boost_step
                                .saturating_mul(saturation_batches)
                                .min(shared_config.max_instances);
                            let boosted_target = if current_workers >= shared_config.max_instances {
                                shared_config.max_instances
                            } else {
                                let remaining_capacity =
                                    shared_config.max_instances - current_workers;
                                current_workers + dynamic_boost_step.min(remaining_capacity)
                            };
                            desired = desired.max(boosted_target);
                            scale_up_step_budget = scale_up_step_budget.max(dynamic_boost_step);
                        }
                        autoscale_last_desired_instances.store(desired, Ordering::Relaxed);
                        autoscale_last_saturation_events
                            .store(saturation_events, Ordering::Relaxed);
                        autoscale_last_dynamic_boost_step
                            .store(scale_up_step_budget, Ordering::Relaxed);
                        let up_budget_target = if current_workers >= shared_config.max_instances {
                            shared_config.max_instances
                        } else {
                            let remaining_capacity = shared_config.max_instances - current_workers;
                            current_workers + scale_up_step_budget.min(remaining_capacity)
                        };
                        let up_target = if desired > current_workers {
                            desired.min(up_budget_target)
                        } else {
                            current_workers
                        };
                        let scale_up_cooldown_elapsed = match last_scale_up_at {
                            Some(at) => now.duration_since(at) >= autoscale_scale_up_cooldown,
                            None => true,
                        };
                        if desired > state.workers.len() && scale_up_cooldown_elapsed {
                            let spawn_count = up_target.saturating_sub(state.workers.len());
                            workers_to_spawn_ports.reserve(spawn_count);
                            for _ in 0..spawn_count {
                                let worker_port = state.next_port;
                                state.next_port = state.next_port.saturating_add(1);
                                workers_to_spawn_ports.push(worker_port);
                            }
                            last_scale_up_at = Some(now);
                            cooldown_anchor_changed = true;
                        }
                        let current_workers = state.workers.len();
                        let min_down_target =
                            if shared_config.autoscale_scale_down_step >= current_workers {
                                0
                            } else {
                                current_workers - shared_config.autoscale_scale_down_step
                            };
                        let down_target = if desired < current_workers {
                            desired.max(min_down_target)
                        } else {
                            current_workers
                        };
                        let scale_down_cooldown_elapsed = match last_scale_down_at {
                            Some(at) => now.duration_since(at) >= autoscale_scale_down_cooldown,
                            None => true,
                        };
                        if desired < state.workers.len() && scale_down_cooldown_elapsed {
                            while state.workers.len() > down_target {
                                if let Some(worker) = state.workers.pop() {
                                    workers_to_stop.push(worker);
                                }
                            }
                            last_scale_down_at = Some(now);
                            cooldown_anchor_changed = true;
                        }
                    }
                }
            }
            if skip_scale_actions {
                continue;
            }
            let mut workers_changed_after_initial_refresh = false;
            for mut worker in workers_to_stop {
                let _ = worker.child.kill();
                let _ = worker.child.wait();
                workers_changed_after_initial_refresh = true;
            }
            let mut spawned_workers = Vec::new();
            spawned_workers.reserve(workers_to_spawn_ports.len());
            for worker_port in workers_to_spawn_ports {
                match spawn_and_wait_lasm_cluster_worker(&shared_config, worker_port) {
                    Ok(worker) => {
                        spawned_workers.push(worker);
                        workers_changed_after_initial_refresh = true;
                    }
                    Err(message) => {
                        eprintln!("warning: LASM cluster autoscale-up failed: {message}");
                        break;
                    }
                }
            }
            if workers_changed_after_initial_refresh {
                let mut state = match state.write() {
                    Ok(state) => state,
                    Err(_) => break,
                };
                if !spawned_workers.is_empty() {
                    state.workers.extend(spawned_workers);
                }
                refresh_lasm_cluster_worker_ports_snapshot_if_changed(
                    &state,
                    &worker_ports_snapshot,
                    &mut last_published_worker_ports,
                );
            }
            if cooldown_anchor_changed {
                autoscale_scale_up_cooldown_remaining_ms.store(
                    lasm_cluster_remaining_cooldown_ms(
                        now,
                        last_scale_up_at,
                        shared_config.autoscale_scale_up_cooldown_ms,
                    ),
                    Ordering::Relaxed,
                );
                autoscale_scale_down_cooldown_remaining_ms.store(
                    lasm_cluster_remaining_cooldown_ms(
                        now,
                        last_scale_down_at,
                        shared_config.autoscale_scale_down_cooldown_ms,
                    ),
                    Ordering::Relaxed,
                );
            }
        }
    })
}
