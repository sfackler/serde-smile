use std::borrow::Cow;
use std::collections::HashMap;

const LIMIT: u16 = 1024;

pub struct StringCache {
    map: HashMap<Cow<'static, str>, u16>,
    next_idx: u16,
}

impl StringCache {
    pub fn new() -> Self {
        StringCache {
            map: HashMap::new(),
            next_idx: 0,
        }
    }

    pub fn intern(&mut self, s: Cow<'static, str>) {
        if self.next_idx >= LIMIT {
            self.map.clear();
            self.next_idx = 0;
        }

        let id = self.next_idx;
        if valid_back_ref(id) {
            self.map.insert(s, id);
        }
        self.next_idx += 1;
    }

    pub fn get(&mut self, s: &str) -> Option<u16> {
        self.map.get(s).copied()
    }
}

fn valid_back_ref(id: u16) -> bool {
    id & 0xff < 0xfe
}
