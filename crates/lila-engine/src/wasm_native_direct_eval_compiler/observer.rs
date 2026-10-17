//! Observation belongs to one actual Engine, never a process-wide counter.
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Default)]
pub(crate) struct NativeDirectEvalCompilationObserver {
    requests: AtomicUsize,
    completed_factories: AtomicUsize,
    syntax_rejections: AtomicUsize,
    private_environment_floor: AtomicUsize,
    visible_private_floor: AtomicUsize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeDirectEvalCompilationSnapshot {
    pub(crate) requests: usize,
    pub(crate) completed_factories: usize,
    pub(crate) syntax_rejections: usize,
    pub(crate) private_environment_floor: usize,
    pub(crate) visible_private_floor: usize,
}
impl NativeDirectEvalCompilationObserver {
    pub(super) fn request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn completed_factory(&self) {
        self.completed_factories.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn syntax_rejection(&self) {
        self.syntax_rejections.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn validated_caller(&self, caller: &lila_ir::RuntimeDirectEvalCaller) {
        self.private_environment_floor.fetch_max(
            caller.private_environment_floor() as usize,
            Ordering::Relaxed,
        );
        let visible = caller
            .descriptor()
            .private_names()
            .values()
            .map(|id| id.class_scope() as usize + 1)
            .max()
            .unwrap_or(0);
        self.visible_private_floor
            .fetch_max(visible, Ordering::Relaxed);
    }
    pub(crate) fn snapshot(&self) -> NativeDirectEvalCompilationSnapshot {
        NativeDirectEvalCompilationSnapshot {
            requests: self.requests.load(Ordering::Relaxed),
            completed_factories: self.completed_factories.load(Ordering::Relaxed),
            syntax_rejections: self.syntax_rejections.load(Ordering::Relaxed),
            private_environment_floor: self.private_environment_floor.load(Ordering::Relaxed),
            visible_private_floor: self.visible_private_floor.load(Ordering::Relaxed),
        }
    }
}
