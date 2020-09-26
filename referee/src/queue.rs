use std::sync::Mutex;

pub struct ThreadSafeQueue<T> {
    queue: Mutex<Vec<T>>,
}

impl<T> ThreadSafeQueue<T> {
    pub fn new(vec: Vec<T>) -> Self {
        ThreadSafeQueue { queue: Mutex::new(vec) }
    }
    pub fn pop(&self) -> Option<T> {
        let mut data = self.queue.lock().unwrap();
        //(*data).pop()
        if (*data).is_empty() {
            return None;
        }
        Some((*data).remove(0))
    }
    pub fn push(&self, item: T) {
        let mut data = self.queue.lock().unwrap();
        (*data).push(item);
    }

    pub fn len(&self) -> usize {
        let data = self.queue.lock().unwrap();
        data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct ThreadSafeString {
    string: Mutex<String>,
}

impl Default for ThreadSafeString {
    fn default() -> Self {
        ThreadSafeString {
            string: Mutex::new(String::new()),
        }
    }
}

impl ThreadSafeString {
    pub fn push(&self, str: &str) {
        let mut data = self.string.lock().unwrap();
        (*data).push_str(str);
    }

    pub fn get_inner(&self) -> String {
        self.string.lock().unwrap().clone()
    }
}
