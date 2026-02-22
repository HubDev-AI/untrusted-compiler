use std::time::Instant;

pub(crate) const LASM_CLUSTER_SELECTION_LOOKUP_NONE: usize = usize::MAX;

pub(crate) fn rebuild_lasm_cluster_backend_selection_lookup(
    worker_port_count: usize,
    unhealthy_ports_until_by_index: &[Option<Instant>],
    unhealthy_port_count: usize,
    lookup: &mut Vec<usize>,
) -> (bool, bool) {
    debug_assert!(unhealthy_ports_until_by_index.len() >= worker_port_count);
    lookup.clear();
    if worker_port_count == 0 {
        return (false, false);
    }
    if unhealthy_port_count >= worker_port_count {
        return (false, false);
    }
    if unhealthy_port_count == 0 {
        return (true, true);
    }
    lookup.resize(worker_port_count, LASM_CLUSTER_SELECTION_LOOKUP_NONE);

    let mut first_healthy_index: Option<usize> = None;
    let mut next_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
    for index in (0..worker_port_count).rev() {
        if unhealthy_ports_until_by_index[index].is_none() {
            next_healthy_index = index;
            first_healthy_index = Some(index);
        }
        lookup[index] = next_healthy_index;
    }

    let Some(first_healthy_index) = first_healthy_index else {
        return (false, false);
    };
    let wrap_fill_start = first_healthy_index + 1;
    if wrap_fill_start < worker_port_count {
        lookup[wrap_fill_start..worker_port_count].fill(first_healthy_index);
    }
    (true, false)
}

pub(crate) fn rebuild_lasm_cluster_worker_backend_addrs(
    worker_ports: &[u16],
    addrs: &mut Vec<std::net::SocketAddr>,
) {
    addrs.clear();
    addrs.reserve(worker_ports.len());
    for port in worker_ports {
        addrs.push(std::net::SocketAddr::from((
            std::net::Ipv4Addr::LOCALHOST,
            *port,
        )));
    }
}

pub(crate) fn remap_lasm_cluster_relay_port_state_by_index(
    previous_ports: &[u16],
    next_ports: &[u16],
    previous_unhealthy_ports_until_by_index: &[Option<Instant>],
    previous_connect_warning_next_allowed_by_index: &[Option<Instant>],
    now: Instant,
    unhealthy_ports_until_by_index: &mut Vec<Option<Instant>>,
    connect_warning_next_allowed_by_index: &mut Vec<Option<Instant>>,
) -> usize {
    unhealthy_ports_until_by_index.clear();
    unhealthy_ports_until_by_index.resize(next_ports.len(), None);
    connect_warning_next_allowed_by_index.clear();
    connect_warning_next_allowed_by_index.resize(next_ports.len(), None);

    let mut previous_index = 0_usize;
    let mut next_index = 0_usize;
    let mut unhealthy_port_count = 0_usize;
    while previous_index < previous_ports.len() && next_index < next_ports.len() {
        match previous_ports[previous_index].cmp(&next_ports[next_index]) {
            std::cmp::Ordering::Less => {
                previous_index += 1;
            }
            std::cmp::Ordering::Greater => {
                next_index += 1;
            }
            std::cmp::Ordering::Equal => {
                if let Some(until) = previous_unhealthy_ports_until_by_index
                    .get(previous_index)
                    .and_then(|value| *value)
                {
                    if until > now {
                        unhealthy_ports_until_by_index[next_index] = Some(until);
                        unhealthy_port_count += 1;
                    }
                }
                if let Some(next_allowed) = previous_connect_warning_next_allowed_by_index
                    .get(previous_index)
                    .and_then(|value| *value)
                {
                    connect_warning_next_allowed_by_index[next_index] = Some(next_allowed);
                }
                previous_index += 1;
                next_index += 1;
            }
        }
    }
    unhealthy_port_count
}
