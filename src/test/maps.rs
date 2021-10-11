use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::iter::FromIterator;

fn run_test<T>(key: T)
where
    T: Display + Serialize + DeserializeOwned + PartialEq + Eq + Debug + Hash,
{
    let expected_bytes =
        crate::to_vec(&HashMap::<_, _>::from_iter([(key.to_string(), "a")])).unwrap();

    let expected = HashMap::<_, _>::from_iter([(key, "a")]);
    let actual_bytes = crate::to_vec(&expected).unwrap();
    assert_eq!(expected_bytes, actual_bytes);

    let actual = crate::from_slice::<HashMap<_, _>>(&expected_bytes).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn u8_keys() {
    run_test(1u8);
}

#[test]
fn u16_keys() {
    run_test(1u16);
}

#[test]
fn u32_keys() {
    run_test(1u32);
}

#[test]
fn u64_keys() {
    run_test(1u64);
}

#[test]
fn u128_keys() {
    run_test(1u128);
}

#[test]
fn i8_keys() {
    run_test(1i8);
}

#[test]
fn i16_keys() {
    run_test(1i16);
}

#[test]
fn i32_keys() {
    run_test(1i32);
}

#[test]
fn i64_keys() {
    run_test(1i64);
}

#[test]
fn i128_keys() {
    run_test(1i128);
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
enum TestEnum {
    Variant,
}

impl Display for TestEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestEnum::Variant => f.write_str("Variant"),
        }
    }
}

#[test]
fn enum_keys() {
    run_test(TestEnum::Variant);
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
struct TestNewtype(String);

impl Display for TestNewtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[test]
fn newtype_keys() {
    run_test(TestNewtype("hello".to_string()))
}
