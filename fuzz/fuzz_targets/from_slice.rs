#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::de::IgnoredAny;
use serde_smile::value::Value;

fuzz_target!(|data: &[u8]| {
    let _ = serde_smile::from_slice::<Value>(data);
    let _ = serde_smile::from_slice::<IgnoredAny>(data);
});
