#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::de::IgnoredAny;
use serde_smile::value::Value;

fuzz_target!(|data: &[u8]| {
    let mut buf = data.to_vec();
    let _ = serde_smile::from_mut_slice::<Value>(&mut buf);
    let mut buf = data.to_vec();
    let _ = serde_smile::from_slice::<IgnoredAny>(&mut buf);
});
