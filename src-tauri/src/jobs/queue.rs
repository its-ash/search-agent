use std::collections::VecDeque;

use parking_lot::Mutex;

pub struct JobQueue<T> {
    inner: Mutex<VecDeque<T>>,
}

impl<T> JobQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push(&self, item: T) {
        self.inner.lock().push_back(item);
    }

    pub fn pop(&self) -> Option<T> {
        self.inner.lock().pop_front()
    }
}
