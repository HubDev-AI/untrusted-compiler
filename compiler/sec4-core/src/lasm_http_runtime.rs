use crate::lasm_runtime::{LasmAsyncRuntime, RunReport, RuntimeAction, TaskId};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn text(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body: body.into().into_bytes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExchange {
    pub request_id: u64,
    pub request_started_at_ms: u64,
    pub response_ready_at_ms: u64,
    pub path_params: BTreeMap<String, String>,
    pub response: HttpResponse,
}

impl HttpExchange {
    pub fn duration_ms(&self) -> u64 {
        self.response_ready_at_ms
            .saturating_sub(self.request_started_at_ms)
    }
}

#[derive(Debug, Clone)]
struct RoutePlan {
    actions: Vec<RuntimeAction>,
    response: HttpResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RouteSegment {
    Literal(String),
    Param(String),
}

#[derive(Debug, Clone)]
struct PatternRoute {
    method: String,
    pattern_segments: Vec<RouteSegment>,
    plan: RoutePlan,
}

#[derive(Debug, Clone)]
struct PendingRequest {
    request_id: u64,
    request_started_at_ms: u64,
    actions: Vec<RuntimeAction>,
    path_params: BTreeMap<String, String>,
    response: HttpResponse,
}

#[derive(Debug)]
pub struct LasmHttpRuntime {
    scheduler: LasmAsyncRuntime,
    exact_routes: HashMap<(String, String), RoutePlan>,
    pattern_routes: Vec<PatternRoute>,
    max_in_flight: Option<usize>,
    max_pending: Option<usize>,
    max_request_duration_ms: Option<u64>,
    task_to_request: HashMap<TaskId, u64>,
    task_started_at_ms: HashMap<TaskId, u64>,
    task_to_path_params: HashMap<TaskId, BTreeMap<String, String>>,
    task_to_response: HashMap<TaskId, HttpResponse>,
    pending_requests: VecDeque<PendingRequest>,
    ready_responses: VecDeque<HttpExchange>,
    next_request_id: u64,
}

impl Default for LasmHttpRuntime {
    fn default() -> Self {
        Self::with_start_time(0)
    }
}

impl LasmHttpRuntime {
    pub fn with_start_time(start_ms: u64) -> Self {
        Self {
            scheduler: LasmAsyncRuntime::with_start_time(start_ms),
            exact_routes: HashMap::new(),
            pattern_routes: Vec::new(),
            max_in_flight: None,
            max_pending: None,
            max_request_duration_ms: None,
            task_to_request: HashMap::new(),
            task_started_at_ms: HashMap::new(),
            task_to_path_params: HashMap::new(),
            task_to_response: HashMap::new(),
            pending_requests: VecDeque::new(),
            ready_responses: VecDeque::new(),
            next_request_id: 1,
        }
    }

    pub fn register_route(
        &mut self,
        method: impl Into<String>,
        path: impl Into<String>,
        actions: Vec<RuntimeAction>,
        response: HttpResponse,
    ) -> Result<(), String> {
        let method = method.into();
        let path = path.into();
        let key = canonical_route_key(method.clone(), path.clone());
        let plan = RoutePlan { actions, response };
        let pattern_segments = split_route_pattern_segments(path.as_str())?;
        if pattern_segments
            .iter()
            .any(|segment| matches!(segment, RouteSegment::Param(_)))
        {
            self.pattern_routes.push(PatternRoute {
                method: key.0,
                pattern_segments,
                plan,
            });
            return Ok(());
        }
        self.exact_routes.insert(key, plan);
        Ok(())
    }

    pub fn set_max_in_flight(&mut self, limit: usize) -> Result<(), String> {
        if limit == 0 {
            return Err("invalid max in-flight limit: must be >= 1".to_string());
        }
        self.max_in_flight = Some(limit);
        self.drain_pending_requests();
        Ok(())
    }

    pub fn clear_max_in_flight(&mut self) {
        self.max_in_flight = None;
        self.drain_pending_requests();
    }

    pub fn set_max_pending(&mut self, limit: usize) -> Result<(), String> {
        if limit == 0 {
            return Err("invalid max pending limit: must be >= 1".to_string());
        }
        self.max_pending = Some(limit);
        Ok(())
    }

    pub fn clear_max_pending(&mut self) {
        self.max_pending = None;
    }

    pub fn set_max_request_duration_ms(&mut self, limit_ms: u64) -> Result<(), String> {
        if limit_ms == 0 {
            return Err("invalid max request duration: must be >= 1ms".to_string());
        }
        self.max_request_duration_ms = Some(limit_ms);
        self.cancel_timed_out_tasks();
        self.drain_pending_requests();
        Ok(())
    }

    pub fn clear_max_request_duration_ms(&mut self) {
        self.max_request_duration_ms = None;
    }

    pub fn in_flight_request_count(&self) -> usize {
        self.task_to_request.len()
    }

    pub fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }

    pub fn submit(&mut self, request: HttpRequest) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let request_started_at_ms = self.scheduler.now_ms();

        if let Some(route) = self.resolve_route_plan(request.method.as_str(), request.path.as_str())
        {
            self.enqueue_or_start_request(request_id, request_started_at_ms, route);
        } else {
            self.ready_responses.push_back(HttpExchange {
                request_id,
                request_started_at_ms,
                response_ready_at_ms: request_started_at_ms,
                path_params: BTreeMap::new(),
                response: HttpResponse::text(404, "route not found"),
            });
        }

        request_id
    }

    pub fn run_until_idle(&mut self, max_steps: usize) -> RunReport {
        let mut total_steps = 0usize;

        while total_steps < max_steps {
            self.drain_pending_requests();

            let remaining = max_steps - total_steps;
            let cycle = self.scheduler.run_until_idle(remaining);
            total_steps = total_steps.saturating_add(cycle.steps);

            for completed in self.scheduler.drain_completed() {
                let Some(request_id) = self.task_to_request.remove(&completed.task_id) else {
                    continue;
                };
                let started_at_ms = self
                    .task_started_at_ms
                    .remove(&completed.task_id)
                    .unwrap_or_else(|| self.scheduler.now_ms());

                let path_params = self
                    .task_to_path_params
                    .remove(&completed.task_id)
                    .unwrap_or_default();

                let response = if self.is_request_timed_out(started_at_ms) {
                    self.task_to_response.remove(&completed.task_id);
                    self.timeout_response()
                } else if completed.code == 0 {
                    self.task_to_response
                        .remove(&completed.task_id)
                        .unwrap_or_else(|| HttpResponse::text(500, "missing response plan"))
                } else {
                    self.task_to_response.remove(&completed.task_id);
                    HttpResponse::text(
                        500,
                        format!("handler exited with status {}", completed.code),
                    )
                };

                self.ready_responses.push_back(HttpExchange {
                    request_id,
                    request_started_at_ms: started_at_ms,
                    response_ready_at_ms: self.scheduler.now_ms(),
                    path_params,
                    response,
                });
            }

            self.cancel_timed_out_tasks();
            self.drain_pending_requests();
            if self.pending_requests.is_empty() && !self.scheduler.has_live_tasks() {
                break;
            }
            if cycle.steps == 0 {
                break;
            }
        }

        RunReport {
            steps: total_steps,
            idle: self.pending_requests.is_empty() && !self.scheduler.has_live_tasks(),
            now_ms: self.scheduler.now_ms(),
        }
    }

    pub fn pop_response(&mut self) -> Option<HttpExchange> {
        self.ready_responses.pop_front()
    }

    pub fn scheduler_now_ms(&self) -> u64 {
        self.scheduler.now_ms()
    }

    fn can_start_request_now(&self) -> bool {
        self.max_in_flight
            .map(|limit| self.task_to_request.len() < limit)
            .unwrap_or(true)
    }

    fn enqueue_or_start_request(
        &mut self,
        request_id: u64,
        request_started_at_ms: u64,
        route: ResolvedRoutePlan,
    ) {
        if self.can_start_request_now() {
            self.start_request_task(
                request_id,
                request_started_at_ms,
                route.actions,
                route.path_params,
                route.response,
            );
            return;
        }

        if self.max_pending_reached() {
            self.ready_responses.push_back(HttpExchange {
                request_id,
                request_started_at_ms,
                response_ready_at_ms: self.scheduler.now_ms(),
                path_params: route.path_params,
                response: HttpResponse::text(503, "runtime queue full"),
            });
            return;
        }

        self.pending_requests.push_back(PendingRequest {
            request_id,
            request_started_at_ms,
            actions: route.actions,
            path_params: route.path_params,
            response: route.response,
        });
    }

    fn start_request_task(
        &mut self,
        request_id: u64,
        request_started_at_ms: u64,
        actions: Vec<RuntimeAction>,
        path_params: BTreeMap<String, String>,
        response: HttpResponse,
    ) {
        let task_id = self.scheduler.spawn_scripted(actions);
        self.task_to_request.insert(task_id, request_id);
        self.task_started_at_ms
            .insert(task_id, request_started_at_ms);
        self.task_to_path_params.insert(task_id, path_params);
        self.task_to_response.insert(task_id, response);
    }

    fn drain_pending_requests(&mut self) {
        while self.can_start_request_now() {
            let Some(pending) = self.pending_requests.pop_front() else {
                break;
            };
            if self.is_request_timed_out(pending.request_started_at_ms) {
                self.ready_responses.push_back(HttpExchange {
                    request_id: pending.request_id,
                    request_started_at_ms: pending.request_started_at_ms,
                    response_ready_at_ms: self.scheduler.now_ms(),
                    path_params: pending.path_params,
                    response: self.timeout_response(),
                });
                continue;
            }
            self.start_request_task(
                pending.request_id,
                pending.request_started_at_ms,
                pending.actions,
                pending.path_params,
                pending.response,
            );
        }
    }

    fn max_pending_reached(&self) -> bool {
        self.max_pending
            .map(|limit| self.pending_requests.len() >= limit)
            .unwrap_or(false)
    }

    fn is_request_timed_out(&self, started_at_ms: u64) -> bool {
        self.max_request_duration_ms
            .map(|limit| self.scheduler.now_ms().saturating_sub(started_at_ms) >= limit)
            .unwrap_or(false)
    }

    fn timeout_response(&self) -> HttpResponse {
        let limit_ms = self.max_request_duration_ms.unwrap_or(0);
        HttpResponse::text(504, format!("handler timed out after {limit_ms}ms"))
    }

    fn cancel_timed_out_tasks(&mut self) {
        if self.max_request_duration_ms.is_none() {
            return;
        }
        let now_ms = self.scheduler.now_ms();
        let mut timed_out = Vec::new();
        for (task_id, started_at_ms) in &self.task_started_at_ms {
            if self
                .max_request_duration_ms
                .is_some_and(|limit| now_ms.saturating_sub(*started_at_ms) >= limit)
            {
                timed_out.push(*task_id);
            }
        }

        for task_id in timed_out {
            let cancelled = self.scheduler.cancel_task(task_id);
            let Some(request_id) = self.task_to_request.remove(&task_id) else {
                continue;
            };
            let Some(started_at_ms) = self.task_started_at_ms.remove(&task_id) else {
                continue;
            };
            let path_params = self.task_to_path_params.remove(&task_id).unwrap_or_default();
            self.task_to_response.remove(&task_id);
            if cancelled {
                self.ready_responses.push_back(HttpExchange {
                    request_id,
                    request_started_at_ms: started_at_ms,
                    response_ready_at_ms: self.scheduler.now_ms(),
                    path_params,
                    response: self.timeout_response(),
                });
            }
        }
    }

    fn resolve_route_plan(&self, method: &str, path: &str) -> Option<ResolvedRoutePlan> {
        let request_match_path = normalized_request_match_path(path).to_string();
        let normalized_method = method.trim().to_ascii_uppercase();
        let request_segments = split_request_segments(request_match_path.as_str());
        if let Some(plan) = self.resolve_route_plan_for_method(
            normalized_method.as_str(),
            request_match_path.as_str(),
            &request_segments,
        ) {
            return Some(plan);
        }
        if normalized_method == "HEAD" {
            return self.resolve_route_plan_for_method(
                "GET",
                request_match_path.as_str(),
                &request_segments,
            );
        }
        None
    }

    fn resolve_route_plan_for_method(
        &self,
        method: &str,
        request_match_path: &str,
        request_segments: &[String],
    ) -> Option<ResolvedRoutePlan> {
        let key = canonical_route_key(method.to_string(), request_match_path.to_string());
        if let Some(route) = self.exact_routes.get(&key) {
            return Some(ResolvedRoutePlan {
                actions: route.actions.clone(),
                response: route.response.clone(),
                path_params: BTreeMap::new(),
            });
        }
        for route in self.pattern_routes.iter().rev() {
            if route.method != key.0 || route.pattern_segments.len() != request_segments.len() {
                continue;
            }
            let mut path_params = BTreeMap::new();
            let mut matches = true;
            for (pattern, request) in route.pattern_segments.iter().zip(request_segments.iter()) {
                match pattern {
                    RouteSegment::Literal(literal) => {
                        if literal != request {
                            matches = false;
                            break;
                        }
                    }
                    RouteSegment::Param(name) => {
                        path_params.insert(name.clone(), request.clone());
                    }
                }
            }
            if matches {
                return Some(ResolvedRoutePlan {
                    actions: route.plan.actions.clone(),
                    response: route.plan.response.clone(),
                    path_params,
                });
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
struct ResolvedRoutePlan {
    actions: Vec<RuntimeAction>,
    response: HttpResponse,
    path_params: BTreeMap<String, String>,
}

fn canonical_route_key(method: String, path: String) -> (String, String) {
    (method.trim().to_ascii_uppercase(), path)
}

fn split_route_pattern_segments(path: &str) -> Result<Vec<RouteSegment>, String> {
    let mut seen_params = HashSet::new();
    let mut segments = Vec::new();
    for segment in path.trim_matches('/').split('/').filter(|entry| !entry.is_empty()) {
        if let Some(param_name) = segment.strip_prefix(':') {
            if !is_valid_route_param_name(param_name) {
                return Err(format!(
                    "invalid route pattern `{path}`: parameter segment `{segment}` must use `:name` with [A-Za-z_][A-Za-z0-9_]*"
                ));
            }
            if !seen_params.insert(param_name.to_string()) {
                return Err(format!(
                    "invalid route pattern `{path}`: duplicate parameter `{param_name}`"
                ));
            }
            segments.push(RouteSegment::Param(param_name.to_string()));
            continue;
        }
        segments.push(RouteSegment::Literal(segment.to_string()));
    }
    Ok(segments)
}

fn is_valid_route_param_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn split_request_segments(path: &str) -> Vec<String> {
    normalized_request_match_path(path)
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect()
}

fn normalized_request_match_path(path: &str) -> &str {
    let mut end = path.len();
    if let Some(index) = path.find('?') {
        end = end.min(index);
    }
    if let Some(index) = path.find('#') {
        end = end.min(index);
    }
    &path[..end]
}

#[cfg(test)]
mod tests {
    use super::{HttpRequest, HttpResponse, LasmHttpRuntime};
    use crate::lasm_runtime::RuntimeAction;

    #[test]
    fn missing_route_returns_404_without_scheduler_work() {
        let mut runtime = LasmHttpRuntime::default();
        let request_id = runtime.submit(HttpRequest::new("GET", "/missing"));

        let report = runtime.run_until_idle(16);
        assert!(report.idle, "runtime should stay idle for missing route");

        let exchange = runtime
            .pop_response()
            .expect("missing route should emit immediate response");
        assert_eq!(exchange.request_id, request_id);
        assert!(
            exchange.path_params.is_empty(),
            "missing route should not emit path params"
        );
        assert_eq!(exchange.response.status, 404);
    }

    #[test]
    fn scripted_route_returns_configured_response() {
        let mut runtime = LasmHttpRuntime::with_start_time(100);
        runtime
            .register_route(
                "GET",
                "/health",
                vec![RuntimeAction::Yield, RuntimeAction::Complete(0)],
                HttpResponse::text(200, "ok"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("get", "/health"));
        let report = runtime.run_until_idle(64);
        assert!(
            report.idle,
            "runtime should become idle after handler completion"
        );

        let exchange = runtime
            .pop_response()
            .expect("successful route should emit response");
        assert_eq!(exchange.request_id, request_id);
        assert!(
            exchange.path_params.is_empty(),
            "exact route should not emit path params"
        );
        assert_eq!(exchange.response.status, 200);
        assert_eq!(exchange.response.body, b"ok");
    }

    #[test]
    fn non_zero_handler_exit_maps_to_deterministic_500_response() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .register_route(
                "POST",
                "/users",
                vec![RuntimeAction::Complete(9)],
                HttpResponse::text(201, "created"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("POST", "/users"));
        let _report = runtime.run_until_idle(16);

        let exchange = runtime.pop_response().expect("response should be ready");
        assert_eq!(exchange.request_id, request_id);
        assert!(
            exchange.path_params.is_empty(),
            "exact route should not emit path params"
        );
        assert_eq!(exchange.response.status, 500);
        let body = String::from_utf8(exchange.response.body).expect("body should be utf-8");
        assert!(
            body.contains("handler exited with status 9"),
            "500 body should include deterministic status detail: {body}"
        );
    }

    #[test]
    fn faster_sleeping_handler_completes_before_slower_handler() {
        let mut runtime = LasmHttpRuntime::with_start_time(10);
        runtime
            .register_route(
                "GET",
                "/slow",
                vec![RuntimeAction::SleepMs(20), RuntimeAction::Complete(0)],
                HttpResponse::text(200, "slow"),
            )
            .expect("route registration should succeed");
        runtime
            .register_route(
                "GET",
                "/fast",
                vec![RuntimeAction::SleepMs(5), RuntimeAction::Complete(0)],
                HttpResponse::text(200, "fast"),
            )
            .expect("route registration should succeed");

        runtime.submit(HttpRequest::new("GET", "/slow"));
        runtime.submit(HttpRequest::new("GET", "/fast"));
        let report = runtime.run_until_idle(128);

        assert!(report.idle, "runtime should complete both requests");
        assert_eq!(runtime.scheduler_now_ms(), 30);

        let first = runtime
            .pop_response()
            .expect("first response should be available");
        let second = runtime
            .pop_response()
            .expect("second response should be available");

        assert_eq!(first.response.body, b"fast");
        assert_eq!(second.response.body, b"slow");
        assert!(
            first.path_params.is_empty() && second.path_params.is_empty(),
            "exact routes should not emit path params"
        );
    }

    #[test]
    fn parameterized_route_matches_path_segments_deterministically() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .register_route(
                "GET",
                "/users/:id",
                vec![RuntimeAction::Complete(0)],
                HttpResponse::text(200, "user"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("GET", "/users/42"));
        let report = runtime.run_until_idle(16);
        assert!(
            report.idle,
            "runtime should complete parameterized route request"
        );

        let exchange = runtime.pop_response().expect("response should be ready");
        assert_eq!(exchange.request_id, request_id);
        assert_eq!(exchange.response.status, 200);
        assert_eq!(exchange.response.body, b"user");
        assert_eq!(
            exchange.path_params.get("id").map(String::as_str),
            Some("42"),
            "runtime should expose deterministic captured path params"
        );
    }

    #[test]
    fn route_matching_ignores_query_and_fragment_in_request_path() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .register_route(
                "GET",
                "/users/:id",
                vec![RuntimeAction::Complete(0)],
                HttpResponse::text(200, "user"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("GET", "/users/42?expand=1#section"));
        let report = runtime.run_until_idle(16);
        assert!(
            report.idle,
            "runtime should complete request with query/fragment"
        );

        let exchange = runtime.pop_response().expect("response should be ready");
        assert_eq!(exchange.request_id, request_id);
        assert_eq!(exchange.response.status, 200);
        assert_eq!(
            exchange.path_params.get("id").map(String::as_str),
            Some("42"),
            "runtime should capture params from normalized request path"
        );
    }

    #[test]
    fn head_request_falls_back_to_get_route_when_head_missing() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .register_route(
                "GET",
                "/health",
                vec![RuntimeAction::Complete(0)],
                HttpResponse::text(200, "ok"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("HEAD", "/health"));
        let report = runtime.run_until_idle(16);
        assert!(report.idle, "runtime should complete HEAD fallback request");

        let exchange = runtime.pop_response().expect("response should be ready");
        assert_eq!(exchange.request_id, request_id);
        assert_eq!(exchange.response.status, 200);
        assert_eq!(exchange.response.body, b"ok");
    }

    #[test]
    fn later_pattern_registration_overrides_earlier_pattern() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .register_route(
                "GET",
                "/users/:id",
                vec![RuntimeAction::Complete(0)],
                HttpResponse::text(200, "first"),
            )
            .expect("route registration should succeed");
        runtime
            .register_route(
                "GET",
                "/users/:id",
                vec![RuntimeAction::Complete(0)],
                HttpResponse::text(200, "second"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("GET", "/users/42"));
        let report = runtime.run_until_idle(16);
        assert!(report.idle, "runtime should complete pattern route request");

        let exchange = runtime.pop_response().expect("response should be ready");
        assert_eq!(exchange.request_id, request_id);
        assert_eq!(exchange.response.status, 200);
        assert_eq!(
            exchange.response.body, b"second",
            "latest pattern route registration should win deterministically"
        );
    }

    #[test]
    fn invalid_param_name_route_pattern_is_rejected() {
        let mut runtime = LasmHttpRuntime::default();
        let error = runtime
            .register_route(
                "GET",
                "/users/:1id",
                vec![RuntimeAction::Complete(0)],
                HttpResponse::text(200, "user"),
            )
            .expect_err("invalid param name should fail route registration");
        assert!(
            error.contains("invalid route pattern"),
            "error should include deterministic pattern context: {error}"
        );
        assert!(
            error.contains("parameter segment `:1id`"),
            "error should include invalid parameter segment detail: {error}"
        );
    }

    #[test]
    fn duplicate_param_names_in_route_pattern_are_rejected() {
        let mut runtime = LasmHttpRuntime::default();
        let error = runtime
            .register_route(
                "GET",
                "/users/:id/orders/:id",
                vec![RuntimeAction::Complete(0)],
                HttpResponse::text(200, "user"),
            )
            .expect_err("duplicate param name should fail route registration");
        assert!(
            error.contains("duplicate parameter `id`"),
            "error should include duplicate parameter detail: {error}"
        );
    }

    #[test]
    fn set_max_in_flight_rejects_zero() {
        let mut runtime = LasmHttpRuntime::default();
        let error = runtime
            .set_max_in_flight(0)
            .expect_err("zero in-flight limit should be rejected");
        assert!(
            error.contains("must be >= 1"),
            "error should include deterministic non-zero guidance: {error}"
        );
    }

    #[test]
    fn max_in_flight_queues_pending_requests_fifo() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .set_max_in_flight(1)
            .expect("in-flight limit should be accepted");
        runtime
            .register_route(
                "GET",
                "/work",
                vec![RuntimeAction::Yield, RuntimeAction::Complete(0)],
                HttpResponse::text(200, "ok"),
            )
            .expect("route registration should succeed");

        let first_request = runtime.submit(HttpRequest::new("GET", "/work"));
        let second_request = runtime.submit(HttpRequest::new("GET", "/work"));
        assert_eq!(
            runtime.pending_request_count(),
            1,
            "second request should be queued while one is in flight"
        );

        let first_report = runtime.run_until_idle(2);
        assert!(
            !first_report.idle,
            "runtime should not be idle because queued request should have started"
        );
        assert_eq!(
            runtime.in_flight_request_count(),
            1,
            "second request should be running after first completion"
        );

        let first_exchange = runtime
            .pop_response()
            .expect("first completed exchange should be available");
        assert_eq!(
            first_exchange.request_id, first_request,
            "queued mode should preserve deterministic FIFO response order"
        );
        assert!(
            runtime.pop_response().is_none(),
            "second request should not complete in the first step budget"
        );

        let second_report = runtime.run_until_idle(2);
        assert!(second_report.idle, "runtime should become idle after queued completion");
        assert_eq!(
            runtime.pending_request_count(),
            0,
            "pending queue should be fully drained"
        );

        let second_exchange = runtime
            .pop_response()
            .expect("second completed exchange should be available");
        assert_eq!(
            second_exchange.request_id, second_request,
            "second queued request should complete deterministically after first"
        );
    }

    #[test]
    fn set_max_pending_rejects_zero() {
        let mut runtime = LasmHttpRuntime::default();
        let error = runtime
            .set_max_pending(0)
            .expect_err("zero max pending limit should be rejected");
        assert!(
            error.contains("must be >= 1"),
            "error should include deterministic non-zero guidance: {error}"
        );
    }

    #[test]
    fn queue_overflow_returns_deterministic_503_response() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .set_max_in_flight(1)
            .expect("in-flight limit should be accepted");
        runtime
            .set_max_pending(1)
            .expect("pending queue limit should be accepted");
        runtime
            .register_route(
                "GET",
                "/users/:id",
                vec![RuntimeAction::Yield, RuntimeAction::Complete(0)],
                HttpResponse::text(200, "ok"),
            )
            .expect("route registration should succeed");

        let first_request = runtime.submit(HttpRequest::new("GET", "/users/1"));
        let second_request = runtime.submit(HttpRequest::new("GET", "/users/2"));
        let third_request = runtime.submit(HttpRequest::new("GET", "/users/3"));
        assert_eq!(
            runtime.pending_request_count(),
            1,
            "one request should be queued up to configured pending limit"
        );

        let overflow_exchange = runtime
            .pop_response()
            .expect("overflow response should be emitted immediately");
        assert_eq!(
            overflow_exchange.request_id, third_request,
            "overflow response should map to the rejected request id"
        );
        assert_eq!(
            overflow_exchange.response.status, 503,
            "overflow should emit deterministic queue-full 503"
        );
        assert_eq!(
            overflow_exchange.response.body,
            b"runtime queue full",
            "overflow response should include deterministic queue-full body"
        );
        assert_eq!(
            overflow_exchange.path_params.get("id").map(String::as_str),
            Some("3"),
            "overflow response should preserve resolved path params for observability"
        );

        let report = runtime.run_until_idle(4);
        assert!(
            report.idle,
            "runtime should still drain in-flight + pending requests after overflow"
        );

        let first_exchange = runtime
            .pop_response()
            .expect("first accepted request should complete");
        let second_exchange = runtime
            .pop_response()
            .expect("second accepted request should complete");
        assert_eq!(first_exchange.request_id, first_request);
        assert_eq!(second_exchange.request_id, second_request);
        assert_eq!(first_exchange.response.status, 200);
        assert_eq!(second_exchange.response.status, 200);
    }

    #[test]
    fn set_max_request_duration_rejects_zero() {
        let mut runtime = LasmHttpRuntime::default();
        let error = runtime
            .set_max_request_duration_ms(0)
            .expect_err("zero max request duration should be rejected");
        assert!(
            error.contains("must be >= 1ms"),
            "error should include deterministic non-zero millisecond guidance: {error}"
        );
    }

    #[test]
    fn completed_request_exceeding_timeout_maps_to_504() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .set_max_request_duration_ms(10)
            .expect("max request duration should be accepted");
        runtime
            .register_route(
                "GET",
                "/slow",
                vec![RuntimeAction::SleepMs(25), RuntimeAction::Complete(0)],
                HttpResponse::text(200, "ok"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("GET", "/slow"));
        let report = runtime.run_until_idle(8);
        assert!(report.idle, "runtime should become idle after timeout mapping");

        let exchange = runtime
            .pop_response()
            .expect("timeout-mapped response should be emitted");
        assert_eq!(exchange.request_id, request_id);
        assert_eq!(exchange.response.status, 504);
        assert_eq!(exchange.response.body, b"handler timed out after 10ms");
    }

    #[test]
    fn live_request_exceeding_timeout_is_cancelled_and_emits_504() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .set_max_request_duration_ms(10)
            .expect("max request duration should be accepted");
        runtime
            .register_route(
                "GET",
                "/slow",
                vec![
                    RuntimeAction::SleepMs(25),
                    RuntimeAction::Yield,
                    RuntimeAction::Complete(0),
                ],
                HttpResponse::text(200, "ok"),
            )
            .expect("route registration should succeed");

        let request_id = runtime.submit(HttpRequest::new("GET", "/slow"));
        let report = runtime.run_until_idle(2);
        assert!(
            report.idle,
            "runtime should cancel timed-out in-flight task and become idle"
        );

        let exchange = runtime
            .pop_response()
            .expect("timed-out cancelled request should emit response");
        assert_eq!(exchange.request_id, request_id);
        assert_eq!(exchange.response.status, 504);
        assert_eq!(exchange.response.body, b"handler timed out after 10ms");

        let follow_up = runtime.run_until_idle(4);
        assert!(follow_up.idle, "runtime should remain idle after cancellation");
        assert!(
            runtime.pop_response().is_none(),
            "cancelled task should not later emit an additional completion response"
        );
    }

    #[test]
    fn pending_request_timeout_is_evaluated_from_submit_time() {
        let mut runtime = LasmHttpRuntime::default();
        runtime
            .set_max_in_flight(1)
            .expect("in-flight limit should be accepted");
        runtime
            .set_max_request_duration_ms(10)
            .expect("max request duration should be accepted");
        runtime
            .register_route(
                "GET",
                "/slow",
                vec![RuntimeAction::SleepMs(20), RuntimeAction::Complete(0)],
                HttpResponse::text(200, "ok"),
            )
            .expect("route registration should succeed");

        let first_request = runtime.submit(HttpRequest::new("GET", "/slow"));
        let second_request = runtime.submit(HttpRequest::new("GET", "/slow"));
        assert_eq!(
            runtime.pending_request_count(),
            1,
            "second request should be queued while first request is in flight"
        );

        let report = runtime.run_until_idle(16);
        assert!(report.idle, "runtime should drain after timeout responses");
        assert_eq!(
            report.now_ms, 20,
            "queued request should timeout from original submit time and avoid extra task execution"
        );

        let first_exchange = runtime
            .pop_response()
            .expect("first request response should be available");
        let second_exchange = runtime
            .pop_response()
            .expect("second request response should be available");
        assert_eq!(first_exchange.request_id, first_request);
        assert_eq!(second_exchange.request_id, second_request);
        assert_eq!(first_exchange.response.status, 504);
        assert_eq!(second_exchange.response.status, 504);
        assert_eq!(first_exchange.response.body, b"handler timed out after 10ms");
        assert_eq!(second_exchange.response.body, b"handler timed out after 10ms");
    }
}
