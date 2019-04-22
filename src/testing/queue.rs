use std::sync::Mutex;
pub struct ThreadSafeQueue<T> {
    queue: Mutex<Vec<T>>,
}

impl<T> ThreadSafeQueue<T> {
    pub fn new(vec: Vec<T>) -> Self {
        ThreadSafeQueue {
            queue: Mutex::new(vec),
        }
    }
    pub fn pop(&self) -> Option<T> {
        let mut data = self.queue.lock().unwrap();
        (*data).pop()
    }
    pub fn push(&self, item: T) {
        let mut data = self.queue.lock().unwrap();
        (*data).push(item);
    }
}
