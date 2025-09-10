use std::borrow::Cow;
use std::collections::HashMap;

const LIMIT: usize = 1024;

pub struct StringCache {
    map: HashMap<Cow<'static, str>, u16>,
    vec: Vec<Cow<'static, str>>,
    count: usize,  // Track insertion count for Jackson-compatible wrap-around
}

impl StringCache {
    pub fn new() -> Self {
        StringCache {
            map: HashMap::new(),
            vec: vec![],
            count: 0,
        }
    }

    pub fn intern(&mut self, s: Cow<'static, str>) {
        // Jackson exact behavior: when reaching LIMIT (1024), 
        // wrap around and start overwriting from index 0
        if self.vec.len() >= LIMIT {
            // Cache is full - implement Jackson's exact wrap-around behavior
            let index = self.count % LIMIT;
            
            // Remove old entry from map
            if let Some(old_string) = self.vec.get(index) {
                self.map.remove(old_string);
            }
            
            // Replace with new entry
            self.vec[index] = s.clone();
            self.map.insert(s, index as u16);
            self.count += 1;
        } else {
            // Cache not full - normal append
            let id = self.vec.len() as u16;
            self.vec.push(s.clone());
            self.map.insert(s, id);
            self.count = self.vec.len();
        }
    }

    pub fn get(&mut self, s: &str) -> Option<u16> {
        self.map.get(s).copied()
    }
}
