use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Default, Clone)]
pub struct TranslationCancelState {
    flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

pub struct CancelHandle {
    file_id: String,
    flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    flag: Arc<AtomicBool>,
}

impl TranslationCancelState {
    pub fn register(&self, file_id: &str) -> CancelHandle {
        let mut flags = self.flags.lock().unwrap();
        let flag = flags
            .entry(file_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone();

        CancelHandle {
            file_id: file_id.to_string(),
            flags: Arc::clone(&self.flags),
            flag,
        }
    }

    pub fn cancel(&self, file_id: &str) {
        let mut flags = self.flags.lock().unwrap();
        let flag = flags
            .entry(file_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)));
        flag.store(true, Ordering::Relaxed);
    }

    pub fn cancel_all(&self) {
        let flags = self.flags.lock().unwrap();
        for flag in flags.values() {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_cancelled(&self, file_id: &str) -> bool {
        self.flags
            .lock()
            .unwrap()
            .get(file_id)
            .map(|flag| flag.load(Ordering::Relaxed))
            .unwrap_or(false)
    }
}

impl CancelHandle {
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.flag)
    }
}

impl Drop for CancelHandle {
    fn drop(&mut self) {
        if let Ok(mut flags) = self.flags.lock() {
            flags.remove(&self.file_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_state_register_and_cancel() {
        let state = TranslationCancelState::default();
        let handle = state.register("file-1");
        assert!(!handle.is_cancelled());

        state.cancel("file-1");
        assert!(handle.is_cancelled());
    }

    #[test]
    fn cancel_state_cancel_before_register() {
        let state = TranslationCancelState::default();
        state.cancel("file-2");

        let handle = state.register("file-2");
        assert!(handle.is_cancelled());
    }

    #[test]
    fn cancel_state_cancel_all() {
        let state = TranslationCancelState::default();
        let handle_a = state.register("file-a");
        let handle_b = state.register("file-b");

        state.cancel_all();

        assert!(handle_a.is_cancelled());
        assert!(handle_b.is_cancelled());
    }
}
