use std::{
    borrow::Borrow,
    collections::{HashMap},
    ops::Index,
    sync::Once,
};
use regex::Regex;
use lazy_static::lazy_static;
use lazy_regex::regex;

lazy_static! {
    // Match these date-time components in a CLDR pattern, except those in single quotes
    static ref EXP_DT_COMPONENTS: Regex = Regex::new(r"(?:[Eec]{1,6}|G{1,5}|[Qq]{1,5}|(?:[yYur]+|U{1,5})|[ML]{1,5}|d{1,2}|D{1,3}|F{1}|[abB]{1,5}|[hkHK]{1,2}|w{1,2}|W{1}|m{1,2}|s{1,2}|[zZOvVxX]{1,4})(?=([^']*'[^']*')*[^']*$)").unwrap();
    // Trim patterns after transformations
    static ref EXP_PATTERN_TRIMMER: Regex = Regex::new(r"^[\s\uFEFF\xA0]+|[\s\uFEFF\xA0]+$").unwrap();
    // Skip over patterns with these date-time components because we don't have data to back them up:
    // timezone, weekday, among others
    static ref UNWANTED_DTCS: Regex = Regex::new(r"[rqQASjJgwWIQq]").unwrap();

    static ref DT_KEYS: Vec<String> = vec!["era", "year", "month", "day", "weekday", "quarter"].iter().map(|&e| String::from(e)).collect();
    static ref TM_KEYS: Vec<String> = vec!["hour", "minute", "second", "hour12", "timeZoneName"].iter().map(|&e| String::from(e)).collect();
}

fn is_date_format_only(obj: &serde_json::Value) -> bool {
    for key in TM_KEYS.iter() {
        if obj.get(key).is_some() {
            return false;
        }
    }
    true
}

fn is_time_format_only(obj: &serde_json::Value) -> bool {
    for key in DT_KEYS.iter() {
        if obj.get(key).is_some() {
            return false;
        }
    }
    true
}

fn join_date_and_time_formats(date_format_obj: &serde_json::Value, time_format_obj: &serde_json::Value) -> serde_json::Value {
    let mut o = serde_json::Value::Object(serde_json::Map::new());
    o["_"] = serde_json::Value::Object(serde_json::Map::new());
    for key in DT_KEYS.iter() {
        if let Some(v) = date_format_obj.get(key) {
            o[key] = v.clone();
        }
        if let Some(v) = date_format_obj["_"].get(key) {
            o["_"][key] = v.clone();
        }
    }
    for key in TM_KEYS.iter() {
        if let Some(v) = time_format_obj.get(key) {
            o[key] = v.clone();
        }
        if let Some(v) = time_format_obj["_"].get(key) {
            o["_"][key] = v.clone();
        }
    }
    o
}

fn compute_final_patterns(format_obj: &mut serde_json::Value) -> &serde_json::Value {
    // From http://www.unicode.org/reports/tr35/tr35-dates.html#Date_Format_Patterns:
    //  'In patterns, two single quotes represents a literal single quote, either
    //   inside or outside single quotes. Text within single quotes is not
    //   interpreted in any way (except for two adjacent single quotes).'
    format_obj["pattern12"] = serde_json::Value::String(regex!(r"'([^']*)'").replace_all(format_obj["extendedPattern"].as_str().unwrap(), |s: &regex::Captures| -> String {
        let literal = &s[1];
        if literal.len() == 0 { "'".into() } else { literal.into() }
    }).into());

    // Pattern 12 is always the default. We can produce the 24 by removing {ampm}
    format_obj["pattern"] = serde_json::Value::String(EXP_PATTERN_TRIMMER.replace_all(format_obj["pattern12"].as_str().unwrap().replace("{ampm}", "").as_ref(), "").into());
    format_obj
}