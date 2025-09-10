use std::borrow::Cow;

const LIMIT: usize = 1024; // Smile spec: maximum 1024 cached strings

pub struct StringCache<'de> {
    vec: Vec<Cow<'de, str>>,
    count: usize,  // Track insertion count for Jackson-compatible wrap-around
}

impl<'de> StringCache<'de> {
    pub fn new() -> Self {
        StringCache { 
            vec: vec![],
            count: 0,
        }
    }

    pub fn intern(&mut self, s: Cow<'de, str>) {
        // Jackson exact behavior: when reaching LIMIT (1024), 
        // wrap around and start overwriting from index 0
        if self.vec.len() >= LIMIT {
            // Cache is full - implement Jackson's exact wrap-around behavior
            let index = self.count % LIMIT;
            self.vec[index] = s;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_wrap_around_behavior() {
        let mut cache = StringCache::new();
        
        // Fill cache to limit
        for i in 0..1024 {
            cache.intern(Cow::Owned(format!("string_{}", i)));
        }
        
        assert_eq!(cache.vec.len(), 1024);
        assert_eq!(cache.count, 1024);
        
        // Verify first entry
        if let Some(s) = cache.get(0) {
            assert_eq!(s, "string_0");
        }
        
        // Add more entries - should wrap around
        cache.intern(Cow::Owned("new_string_0".to_string()));
        
        // Should still have 1024 entries
        assert_eq!(cache.vec.len(), 1024);
        assert_eq!(cache.count, 1025);
        
        // Index 0 should now contain the new string
        if let Some(s) = cache.get(0) {
            assert_eq!(s, "new_string_0");
        }
        
        // Index 1 should still contain original
        if let Some(s) = cache.get(1) {
            assert_eq!(s, "string_1");
        }
        
        // Add another - should overwrite index 1
        cache.intern(Cow::Owned("new_string_1".to_string()));
        
        assert_eq!(cache.count, 1026);
        if let Some(s) = cache.get(1) {
            assert_eq!(s, "new_string_1");
        }
        
        // Index 2 should still be original
        if let Some(s) = cache.get(2) {
            assert_eq!(s, "string_2");
        }
    }
}