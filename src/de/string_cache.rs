use std::borrow::Cow;

const LIMIT: usize = 1024; // Smile spec: maximum 1024 cached strings

pub struct StringCache<'de> {
    vec: Vec<Cow<'de, str>>,
    count: usize,  // Track insertion count for Jackson-compatible wrap-around
}

impl<'de> StringCache<'de> {
    pub fn new() -> Self {
        StringCache { 
            vec: Vec::with_capacity(LIMIT),
            count: 0,
        }
    }

    pub fn intern(&mut self, s: Cow<'de, str>) {
        // Jackson exact behavior: when reaching LIMIT (1024), 
        // wrap around and start overwriting from index 0
        if self.vec.len() >= LIMIT {
            // Cache is full - implement Jackson's exact wrap-around behavior
            let index = self.count % LIMIT;
            if index < self.vec.len() {
                self.vec[index] = s;
            } else {
                self.vec.push(s);
            }
            self.count += 1;
        } else {
            // Cache not full - normal append
            self.vec.push(s);
            self.count = self.vec.len();
        }
    }

    pub fn get(&self, reference: u16) -> Option<&Cow<'de, str>> {
        self.vec.get(reference as usize)
    }
}
