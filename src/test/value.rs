use crate::value::{BigDecimal, BigInteger, Value};
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bytes::ByteBuf;
use std::fmt::Debug;
use std::iter::FromIterator;

fn run_test<T>(raw: T, value: Value)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let expected = crate::to_vec(&raw).unwrap();
    let actual = crate::to_vec(&raw).unwrap();
    assert_eq!(expected, actual);

    let actual = crate::from_slice::<Value>(&actual).unwrap();
    assert_eq!(value, actual);
}

#[test]
fn null() {
    run_test((), Value::Null);
}

#[test]
fn boolean() {
    run_test(true, Value::Boolean(true));
    run_test(false, Value::Boolean(false));
}

#[test]
fn integer() {
    run_test(0i32, Value::Integer(0));
    run_test(-10i32, Value::Integer(-10));
    run_test(10i32, Value::Integer(10));
}

#[test]
fn long() {
    run_test(1i64 << 50, Value::Long(1 << 50));
    run_test(-(1i64 << 50), Value::Long(-(1 << 50)));
}

#[test]
fn big_integer() {
    run_test(
        BigInteger::from_be_bytes(vec![0]),
        Value::BigInteger(BigInteger::from_be_bytes(vec![0])),
    );
    run_test(
        BigInteger::from_be_bytes(vec![0xff; 30]),
        Value::BigInteger(BigInteger::from_be_bytes(vec![0xff; 30])),
    );
}

#[test]
fn float() {
    run_test(0f32, Value::Float(0f32));
    run_test(f32::INFINITY, Value::Float(f32::INFINITY));
}

#[test]
fn double() {
    run_test(0f64, Value::Double(0f64));
    run_test(f64::INFINITY, Value::Double(f64::INFINITY));
}

#[test]
fn big_decimal() {
    run_test(
        BigDecimal::new(BigInteger::from_be_bytes(vec![0]), 0),
        Value::BigDecimal(BigDecimal::new(BigInteger::from_be_bytes(vec![0]), 0)),
    );
    run_test(
        BigDecimal::new(BigInteger::from_be_bytes(vec![0xff; 30]), -10),
        Value::BigDecimal(BigDecimal::new(
            BigInteger::from_be_bytes(vec![0xff; 30]),
            -10,
        )),
    );
}

#[test]
fn string() {
    run_test("".to_string(), Value::String("".to_string()));
    run_test(
        "hello world".to_string(),
        Value::String("hello world".to_string()),
    );
}

#[test]
fn binary() {
    run_test(ByteBuf::from(vec![]), Value::Binary(vec![]));
    run_test(ByteBuf::from(vec![0xff; 30]), Value::Binary(vec![0xff; 30]));
}

#[test]
fn array() {
    run_test(Vec::<i32>::new(), Value::Array(vec![]));
    run_test(
        vec![1, 2],
        Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
    );
}

#[test]
fn object() {
    run_test(
        IndexMap::<String, i32>::new(),
        Value::Object(IndexMap::new()),
    );
    run_test(
        IndexMap::<_, _>::from_iter([("Hello".to_string(), 123)]),
        Value::Object(IndexMap::<_, _>::from_iter([(
            "Hello".to_string(),
            Value::Integer(123),
        )])),
    );
}
