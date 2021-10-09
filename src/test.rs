use crate::ser::Serializer;
use serde::de::{self, DeserializeOwned};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
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

fn run_category<T>(name: &str)
where
    T: Serialize + DeserializeOwned,
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
    T: Serialize + DeserializeOwned,
{
    println!("testing {}", path.display());

    let test_case = fs::read(path).unwrap();
    let test_case = serde_json::from_slice::<TestCase<T>>(&test_case).unwrap();

    let expected = fs::read(path.with_extension("smile")).unwrap();

    let mut serializer = Serializer::builder()
        .raw_binary(test_case.raw_binary)
        .shared_strings(test_case.shared_strings)
        .shared_properties(test_case.shared_properties)
        .build(vec![])
        .unwrap();
    test_case.value.serialize(&mut serializer).unwrap();
    let actual = if test_case.write_end_marker {
        serializer.end().unwrap()
    } else {
        serializer.into_inner()
    };

    assert_eq!(expected, actual);
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
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        base64::decode(&s)
            .map(Base64Binary)
            .map_err(|e| de::Error::custom(e))
    }
}
