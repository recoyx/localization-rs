use serde::{Serialize, Deserialize};
use serde_repr::*;
use std::{borrow::Borrow, collections::HashMap, sync::Once};

static mut GLOBAL_DATA: Option<HashMap<String, BasicLanguageInfo>> = None;
static START: Once = Once::new();

pub fn init_static_data() {
    START.call_once(|| unsafe {
        if GLOBAL_DATA.is_none() {
            GLOBAL_DATA = Some(serde_json::from_str::<HashMap<String, BasicLanguageInfo>>(&String::from_utf8_lossy(include_bytes!("basic_language_info_data.json"))).unwrap());
        }
    });
}

pub fn global_basic_locale_data() -> &'static HashMap<String, BasicLanguageInfo> {
    unsafe {
        init_static_data();
        GLOBAL_DATA.as_ref().unwrap().borrow()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BasicLanguageInfo {
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