//! Per-Engine test observation of the actual native compiler callback.

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub(crate) struct NativeFunctionCompilationObserver {
    requests: AtomicUsize,
    completed_factories: AtomicUsize,
    syntax_rejections: AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeFunctionCompilationSnapshot {
    pub(crate) requests: usize,
    pub(crate) completed_factories: usize,
    pub(crate) syntax_rejections: usize,
}

impl NativeFunctionCompilationObserver {
    pub(super) fn request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn completed_factory(&self) {
        self.completed_factories.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn syntax_rejection(&self) {
        self.syntax_rejections.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self) -> NativeFunctionCompilationSnapshot {
        NativeFunctionCompilationSnapshot {
            requests: self.requests.load(Ordering::Relaxed),
            completed_factories: self.completed_factories.load(Ordering::Relaxed),
            syntax_rejections: self.syntax_rejections.load(Ordering::Relaxed),
        }
    }
}
