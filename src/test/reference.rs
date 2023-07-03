use crate::ser::Serializer;
use crate::value::{BigDecimal, BigInteger};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use indexmap::IndexMap;
use serde::de::{self, DeserializeOwned, IntoDeserializer};
use serde::{Deserialize, Deserializer, Serialize};
use serde_bytes::ByteBuf;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

macro_rules! category {
    ($category:ident, $t:ty) => {
        #[test]
        fn $category() {
            run_category::<$t>(stringify!($category));
        }
    };
}

category!(integer, i32);
category!(long, i64);
category!(string, String);
category!(float, f32);
category!(double, f64);
category!(boolean, bool);
category!(binary, Base64Binary);
category!(null, ());
category!(list, Vec<String>);
category!(map, IndexMap<String, i32>);
category!(shared_property, Vec<IndexMap<String, i32>>);
category!(shared_string, Vec<IndexMap<String, String>>);
category!(big_integer, TextBigInteger);
category!(big_decimal, TextBigDecimal);

fn run_category<T>(name: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    for r in fs::read_dir(format!("tests/{}", name)).unwrap() {
        let path = r.unwrap().path();
        if path.extension() != Some(OsStr::new("json")) {
            continue;
        }

        run_test::<T>(&path);
    }
}

fn run_test<T>(path: &Path)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    println!("testing {}", path.display());

    let test_case = fs::read(path).unwrap();
    let test_case = serde_json::from_slice::<TestCase<T>>(&test_case).unwrap();

    let mut expected = fs::read(path.with_extension("smile")).unwrap();

    let mut serializer = Serializer::builder()
        .raw_binary(test_case.raw_binary)
        .shared_strings(test_case.shared_strings)
        .shared_properties(test_case.shared_properties)
        .build(vec![]);
    test_case.value.serialize(&mut serializer).unwrap();
    if test_case.write_end_marker {
        serializer.end().unwrap()
    }
    let actual = serializer.into_inner();

    assert_eq!(expected, actual);

    let actual = crate::from_slice::<T>(&expected).unwrap();
    assert_eq!(test_case.value, actual);

    let actual = crate::from_reader::<T, _>(&*expected).unwrap();
    assert_eq!(test_case.value, actual);

    let actual = crate::from_mut_slice::<T>(&mut expected).unwrap();
    assert_eq!(test_case.value, actual);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCase<T> {
    #[serde(default)]
    raw_binary: bool,
    #[serde(default)]
    shared_strings: bool,
    #[serde(default)]
    shared_properties: bool,
    #[serde(default)]
    write_end_marker: bool,
    value: T,
}

// serde-json doesn't use base64 for binary so we need a shim
#[derive(PartialEq, Debug)]
struct Base64Binary(Vec<u8>);

impl Serialize for Base64Binary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Base64Binary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            STANDARD
                .decode(s)
                .map(Base64Binary)
                .map_err(de::Error::custom)
        } else {
            ByteBuf::deserialize(deserializer).map(|v| Base64Binary(v.into_vec()))
        }
    }
}

// BigInteger can't deserialize from JSON so we need a shim
#[derive(PartialEq, Debug)]
struct TextBigInteger(BigInteger);

impl Serialize for TextBigInteger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TextBigInteger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let v = i128::deserialize(deserializer)?;
            let padding_bits = if v < 0 {
                v.leading_ones()
            } else {
                v.leading_zeros()
            } - 1;
            let padding_bytes = padding_bits / 8;
            Ok(TextBigInteger(BigInteger::from_be_bytes(
                v.to_be_bytes()[padding_bytes as usize..].to_vec(),
            )))
        } else {
            BigInteger::deserialize(deserializer).map(TextBigInteger)
        }
    }
}

// BigDecimal can't deserialize from JSON so we need a shim
#[derive(PartialEq, Debug)]
struct TextBigDecimal(BigDecimal);

impl Serialize for TextBigDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TextBigDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let v = f32::deserialize(deserializer)?.to_string();
            let value = v.replace('.', "");
            let value = value.parse::<i128>().unwrap();
            let value = TextBigInteger::deserialize(value.into_deserializer())?.0;
            let scale = match v.find('.') {
                Some(idx) => v.len() - idx - 1,
                None => 0,
            };

            Ok(TextBigDecimal(BigDecimal::new(value, scale as i32)))
        } else {
            BigDecimal::deserialize(deserializer).map(TextBigDecimal)
        }
    }
}
