use std::time::Instant;

pub(crate) const LASM_CLUSTER_SELECTION_LOOKUP_NONE: usize = usize::MAX;

pub(crate) fn rebuild_lasm_cluster_backend_selection_lookup(
    worker_port_count: usize,
    unhealthy_ports_until_by_index: &[Option<Instant>],
    unhealthy_port_count: usize,
    lookup: &mut Vec<usize>,
) -> (bool, bool, usize) {
    debug_assert!(unhealthy_ports_until_by_index.len() >= worker_port_count);
    lookup.clear();
    if worker_port_count == 0 {
        return (false, false, 0);
    }
    if unhealthy_port_count >= worker_port_count {
        return (false, false, 0);
    }
    if unhealthy_port_count == 0 {
        return (true, true, worker_port_count);
    }
    for (index, unhealthy_until) in unhealthy_ports_until_by_index
        .iter()
        .take(worker_port_count)
        .enumerate()
    {
        if unhealthy_until.is_none() {
            lookup.push(index);
        }
    }
    let healthy_count = lookup.len();
    if healthy_count == 0 {
        return (false, false, 0);
    }
    (true, false, healthy_count)
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
