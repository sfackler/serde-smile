use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum TestEnum {
    Unit,
    Newtype(i32),
    Tuple(i32, bool),
    Struct { a: i32, b: bool },
}

#[derive(Serialize)]
struct NewtypeEquivalent {
    #[serde(rename = "Newtype")]
    newtype: i32,
}

#[derive(Serialize)]
struct TupleEquivalent {
    #[serde(rename = "Tuple")]
    tuple: (i32, bool),
}

#[derive(Serialize)]
struct StructEquivalent {
    #[serde(rename = "Struct")]
    struct_: StructEquivalentInner,
}

#[derive(Serialize)]
struct StructEquivalentInner {
    a: i32,
    b: bool,
}

#[test]
fn unit_variant() {
    let expected = TestEnum::Unit;

    let expected_bytes = crate::to_vec("Unit").unwrap();
    let actual_bytes = crate::to_vec(&expected).unwrap();
    assert_eq!(expected_bytes, actual_bytes);

    let actual = crate::from_slice::<TestEnum>(&expected_bytes).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn newtype_variant() {
    let expected = TestEnum::Newtype(42);

    let expected_bytes = crate::to_vec(&NewtypeEquivalent { newtype: 42 }).unwrap();
    let actual_bytes = crate::to_vec(&expected).unwrap();
    assert_eq!(expected_bytes, actual_bytes);

    let actual = crate::from_slice::<TestEnum>(&expected_bytes).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn tuple_variant() {
    let expected = TestEnum::Tuple(42, true);

    let expected_bytes = crate::to_vec(&TupleEquivalent { tuple: (42, true) }).unwrap();
    let actual_bytes = crate::to_vec(&expected).unwrap();
    assert_eq!(expected_bytes, actual_bytes);

    let actual = crate::from_slice::<TestEnum>(&expected_bytes).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn struct_variant() {
    let expected = TestEnum::Struct { a: 42, b: true };

    let expected_bytes = crate::to_vec(&StructEquivalent {
        struct_: StructEquivalentInner { a: 42, b: true },
    })
    .unwrap();
    let actual_bytes = crate::to_vec(&expected).unwrap();
    assert_eq!(expected_bytes, actual_bytes);

    let actual = crate::from_slice::<TestEnum>(&expected_bytes).unwrap();
    assert_eq!(expected, actual);
}
