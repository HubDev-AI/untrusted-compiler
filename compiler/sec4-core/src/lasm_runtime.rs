use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeAction {
    Yield,
    SleepMs(u64),
    Complete(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskExit {
    pub task_id: TaskId,
    pub code: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub steps: usize,
    pub idle: bool,
    pub now_ms: u64,
}

#[derive(Debug, Clone)]
struct Task {
    actions: Vec<RuntimeAction>,
    pc: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SleepEntry {
    wake_at_ms: u64,
    task_id: TaskId,
}

#[derive(Debug, Default)]
pub struct LasmAsyncRuntime {
    now_ms: u64,
    next_task_id: u64,
    tasks: HashMap<TaskId, Task>,
    ready: VecDeque<TaskId>,
    sleeping: BinaryHeap<Reverse<SleepEntry>>,
    completed: Vec<TaskExit>,
}

impl LasmAsyncRuntime {
    pub fn with_start_time(start_ms: u64) -> Self {
        Self {
            now_ms: start_ms,
            next_task_id: 1,
            tasks: HashMap::new(),
            ready: VecDeque::new(),
            sleeping: BinaryHeap::new(),
            completed: Vec::new(),
        }
    }

    pub fn now_ms(&self) -> u64 {
        self.now_ms
    }

    pub fn spawn_scripted(&mut self, actions: Vec<RuntimeAction>) -> TaskId {
        let task_id = TaskId(self.next_task_id);
        self.next_task_id += 1;
        self.tasks.insert(task_id, Task { actions, pc: 0 });
        self.ready.push_back(task_id);
        task_id
    }

    pub fn completed(&self) -> &[TaskExit] {
        &self.completed
    }

    pub fn drain_completed(&mut self) -> Vec<TaskExit> {
        std::mem::take(&mut self.completed)
    }

    pub fn has_live_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    pub fn live_task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn cancel_task(&mut self, task_id: TaskId) -> bool {
        if self.tasks.remove(&task_id).is_none() {
            return false;
        }
        self.ready.retain(|entry| *entry != task_id);
        true
    }

    pub fn run_until_idle(&mut self, max_steps: usize) -> RunReport {
        let mut steps = 0usize;

        while steps < max_steps {
            self.wake_ready_tasks();

            if let Some(task_id) = self.ready.pop_front() {
                steps += 1;
                self.step_task(task_id);
                continue;
            }

            if let Some(next_wakeup_ms) = self.next_wakeup_ms() {
                self.now_ms = next_wakeup_ms;
                self.wake_ready_tasks();
                if self.ready.is_empty() {
                    break;
                }
                continue;
            }

            break;
        }

        RunReport {
            steps,
            idle: self.ready.is_empty() && self.sleeping.is_empty(),
            now_ms: self.now_ms,
        }
    }

    fn step_task(&mut self, task_id: TaskId) {
        let Some(task) = self.tasks.get_mut(&task_id) else {
            return;
        };

        let action = task.actions.get(task.pc).cloned();
        task.pc += 1;

        match action {
            Some(RuntimeAction::Yield) => {
                self.ready.push_back(task_id);
            }
            Some(RuntimeAction::SleepMs(duration_ms)) => {
                let wake_at_ms = self.now_ms.saturating_add(duration_ms);
                self.sleeping.push(Reverse(SleepEntry {
                    wake_at_ms,
                    task_id,
                }));
            }
            Some(RuntimeAction::Complete(code)) => {
                self.finish_task(task_id, code);
            }
            None => {
                self.finish_task(task_id, 0);
            }
        }
    }

    fn finish_task(&mut self, task_id: TaskId, code: i64) {
        self.tasks.remove(&task_id);
        self.completed.push(TaskExit { task_id, code });
    }

    fn wake_ready_tasks(&mut self) {
        while let Some(Reverse(sleep)) = self.sleeping.peek().copied() {
            if sleep.wake_at_ms > self.now_ms {
                break;
            }
            self.sleeping.pop();
            if self.tasks.contains_key(&sleep.task_id) {
                self.ready.push_back(sleep.task_id);
            }
        }
    }

    fn next_wakeup_ms(&self) -> Option<u64> {
        self.sleeping.peek().map(|entry| entry.0.wake_at_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::{LasmAsyncRuntime, RuntimeAction, TaskId};

    #[test]
    fn runtime_runs_tasks_until_idle_and_collects_exit_codes() {
        let mut runtime = LasmAsyncRuntime::with_start_time(1_000);
        runtime.spawn_scripted(vec![RuntimeAction::Yield, RuntimeAction::Complete(7)]);

        let report = runtime.run_until_idle(64);

        assert!(
            report.idle,
            "runtime should be idle after scripted completion"
        );
        assert_eq!(report.steps, 2, "runtime should execute yield + complete");
        assert_eq!(runtime.completed().len(), 1, "one task should complete");
        assert_eq!(
            runtime.completed()[0].code,
            7,
            "exit code should match script"
        );
    }

    #[test]
    fn runtime_advances_clock_for_sleep_and_wakes_tasks_deterministically() {
        let mut runtime = LasmAsyncRuntime::with_start_time(10);
        runtime.spawn_scripted(vec![RuntimeAction::SleepMs(15), RuntimeAction::Complete(1)]);
        runtime.spawn_scripted(vec![RuntimeAction::SleepMs(5), RuntimeAction::Complete(2)]);

        let report = runtime.run_until_idle(64);

        assert!(
            report.idle,
            "runtime should be idle after both tasks complete"
        );
        assert_eq!(report.now_ms, 25, "clock should advance to latest wakeup");
        assert_eq!(runtime.completed().len(), 2, "both tasks should complete");
        assert_eq!(
            runtime.completed()[0].code,
            2,
            "shorter sleep should complete first"
        );
        assert_eq!(
            runtime.completed()[1].code,
            1,
            "longer sleep should complete second"
        );
    }

    #[test]
    fn runtime_reports_non_idle_when_step_budget_is_exhausted() {
        let mut runtime = LasmAsyncRuntime::with_start_time(0);
        runtime.spawn_scripted(vec![
            RuntimeAction::Yield,
            RuntimeAction::Yield,
            RuntimeAction::Yield,
            RuntimeAction::Complete(0),
        ]);

        let report = runtime.run_until_idle(2);

        assert!(
            !report.idle,
            "runtime should not be idle when budget is exhausted"
        );
        assert!(
            runtime.has_live_tasks(),
            "task should still be live after partial run"
        );
        assert!(
            runtime.completed().is_empty(),
            "task should not complete within limited steps"
        );
    }

    #[test]
    fn cancel_task_removes_live_task_without_completion() {
        let mut runtime = LasmAsyncRuntime::with_start_time(0);
        let task =
            runtime.spawn_scripted(vec![RuntimeAction::SleepMs(25), RuntimeAction::Complete(7)]);
        assert_eq!(
            runtime.live_task_count(),
            1,
            "task should be live before cancel"
        );

        let cancelled = runtime.cancel_task(task);
        assert!(cancelled, "existing task cancellation should return true");
        assert_eq!(
            runtime.live_task_count(),
            0,
            "cancelled task should be removed"
        );

        let report = runtime.run_until_idle(8);
        assert!(
            report.idle,
            "runtime should remain idle after cancelled task"
        );
        assert!(
            runtime.completed().is_empty(),
            "cancelled task should not produce completion record"
        );
        assert!(
            !runtime.cancel_task(TaskId(999)),
            "cancelling unknown task should return false"
        );
    }
}
