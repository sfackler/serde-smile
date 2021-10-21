#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::de::IgnoredAny;
use serde_smile::value::Value;

fuzz_target!(|data: &[u8]| {
    let mut reader = data;
    let _ = serde_smile::from_reader::<Value, _>(&mut reader);
    let mut reader = data;
    let _ = serde_smile::from_reader::<IgnoredAny, _>(&mut reader);
});
