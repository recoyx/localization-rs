use serde::{Serialize, Deserialize};
use serde_repr::*;
use std::{collections::HashMap};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOCALE_BASIC_DATA: HashMap<String, LocaleBasicData> = serde_json::from_str::<HashMap<String, LocaleBasicData>>(&String::from_utf8_lossy(include_bytes!("../locale-data/basic_data.json"))).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct LocaleBasicData {
    pub universal_name: String,
    pub native_name: String,
    pub direction: Direction
}

#[repr(u64)]
#[derive(Copy, Clone, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum Direction {
    LeftToRight = 1,
    RightToLeft = 0,
}