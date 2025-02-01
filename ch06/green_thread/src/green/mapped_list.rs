use std::collections::{HashMap, LinkedList};

pub struct MappedList<T> {
    map: HashMap<u64, LinkedList<T>>,
}

impl<T> MappedList<T> {
    pub fn new() -> Self {
        MappedList {
            map: HashMap::new(),
        }
    }

    pub fn push_back(&mut self, key: u64, value: T) {
        if let Some(list) = self.map.get_mut(&key) {
            list.push_back(value);
        } else {
            let mut list = LinkedList::new();
            list.push_back(value);
            self.map.insert(key, list);
        }
    }

    pub fn pop_front(&mut self, key: u64) -> Option<T> {
        if let Some(list) = self.map.get_mut(&key) {
            let val = list.pop_front();
            if list.is_empty() {
                self.map.remove(&key);
            }
            val
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}
