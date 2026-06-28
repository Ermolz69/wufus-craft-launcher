use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Cheaply cloneable cancellation flag.
/// Shared between the caller and all download tasks.
#[derive(Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_cancelled() {
        assert!(!CancelToken::new().is_cancelled());
    }

    #[test]
    fn cancel_propagates_to_all_clones() {
        let token = CancelToken::new();
        let clone = token.clone();

        token.cancel();

        assert!(clone.is_cancelled());
    }
}
