use std::borrow::Cow;
use std::collections::HashMap;

const LIMIT: usize = 1024;

pub struct StringCache {
    map: HashMap<Cow<'static, str>, u16>,
}

impl StringCache {
    pub fn new() -> Self {
        StringCache {
            map: HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: Cow<'static, str>) {
        if self.map.len() >= LIMIT {
            self.map.clear();
        }

        let id = self.map.len() as u16;
        self.map.insert(s, id);
    }

    pub fn get(&mut self, s: &str) -> Option<u16> {
        self.map.get(s).copied()
    }
}
