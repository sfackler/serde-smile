use std::borrow::Cow;

const LIMIT: usize = 1024;

pub struct StringCache<'de> {
    vec: Vec<Cow<'de, str>>,
}

impl<'de> StringCache<'de> {
    pub fn new() -> Self {
        StringCache { vec: vec![] }
    }

    pub fn intern(&mut self, s: Cow<'de, str>) {
        if self.vec.len() >= LIMIT {
            self.vec.clear();
        }

        self.vec.push(s);
    }

    pub fn get(&self, reference: u16) -> Option<&Cow<'de, str>> {
        self.vec.get(reference as usize)
    }
}
