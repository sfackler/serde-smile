use crate::{Deserializer, Serializer};
use serde::Serialize;

#[test]
fn empty() {
    let buf = Serializer::new(vec![]).unwrap().into_inner();

    let mut it = Deserializer::from_slice(&buf).unwrap().into_iter::<()>();
    assert!(it.next().is_none());
}

#[test]
fn empty_eos() {
    let mut ser = Serializer::new(vec![]).unwrap();
    ser.end().unwrap();
    let buf = ser.into_inner();

    let mut it = Deserializer::from_slice(&buf).unwrap().into_iter::<()>();
    assert!(it.next().is_none());
}

#[test]
fn multiple() {
    let mut ser = Serializer::new(vec![]).unwrap();
    1i32.serialize(&mut ser).unwrap();
    2i32.serialize(&mut ser).unwrap();
    3i32.serialize(&mut ser).unwrap();
    let buf = ser.into_inner();

    let values = Deserializer::from_slice(&buf)
        .unwrap()
        .into_iter::<i32>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(values, [1, 2, 3]);
}

#[test]
fn stop_at_eos() {
    let mut ser = Serializer::new(vec![]).unwrap();
    1i32.serialize(&mut ser).unwrap();
    2i32.serialize(&mut ser).unwrap();
    3i32.serialize(&mut ser).unwrap();
    ser.end().unwrap();
    let mut buf = ser.into_inner();
    buf.push(0);

    let mut buf = &buf[..];
    let values = Deserializer::from_reader(&mut buf)
        .unwrap()
        .into_iter::<i32>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(values, [1, 2, 3]);
    assert_eq!(buf, [0]);
}
