use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

fn run_test<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let bytes = crate::to_vec(&value).unwrap();
    let actual = crate::from_slice(&bytes).unwrap();
    assert_eq!(value, actual);
}

#[test]
fn u64() {
    run_test(u64::MAX)
}

#[test]
fn u128() {
    run_test(0u128);
    run_test(u128::MAX);
    run_test(1u128 << 115);
}

#[test]
fn i128() {
    run_test(0i128);
    run_test(i128::MIN);
    run_test(i128::MAX);
    run_test(1i128 << 115);
    run_test(-(1i128 << 115));
}
