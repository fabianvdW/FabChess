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
        //(*data).pop()
        if (*data).len() == 0 {
            return None;
        }
        Some((*data).remove(0))
    }
    pub fn push(&self, item: T) {
        let mut data = self.queue.lock().unwrap();
        (*data).push(item);
    }
}

pub struct ThreadSafeString {
    string: Mutex<String>,
}

impl ThreadSafeString {
    pub fn new() -> Self {
        ThreadSafeString {
            string: Mutex::new(String::new()),
        }
    }

    pub fn push(&self, str: &String) {
        let mut data = self.string.lock().unwrap();
        (*data).push_str(str);
    }

    pub fn get_inner(&self) -> String {
        self.string.lock().unwrap().clone()
    }
}
