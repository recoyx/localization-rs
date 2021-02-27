use std::{collections::{HashMap}};
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

fn exp_dt_components_meta<'a>(_0: &'a str, format_obj: &mut serde_json::Value) -> &'a str {
    match _0.chars().collect::<Vec<char>>().get(0).unwrap_or(&'0') {
        // --- Era
        'G' => {
            format_obj["era"] = serde_json::Value::String(vec!["short", "short", "short", "long", "narrow"][_0.len() - 1].to_string());
            "{era}"
        },
        // --- Year
        'y' | 'Y' | 'u' | 'U' | 'r' => {
            format_obj["year"] = serde_json::Value::String((if _0.len() == 2 { "2-digit" } else { "numeric" }).to_string());
            "{year}"
        },
        // --- Quarter (not supported in this polyfill)
        'Q' | 'q' => {
            format_obj["quarter"] = serde_json::Value::String(vec!["numeric", "2-digit", "short", "long", "narrow"][_0.len() - 1].to_string());
            "{quarter}"
        },
        // --- Month
        'M' | 'L' => {
            format_obj["month"] = serde_json::Value::String(vec!["numeric", "2-digit", "short", "long", "narrow"][_0.len() - 1].to_string());
            "{month}"
        },
        // --- Week (not supported in this polyfill)
        'w' => {
            // week of the year
            format_obj["week"] = serde_json::Value::String((if _0.len() == 2 { "2-digit" } else { "numeric" }).to_string());
            "{weekday}"
        },
        'W' => {
            // week of the month
            format_obj["week"] = serde_json::Value::String("numeric".to_string());
            "{weekday}"
        },
        // --- Day
        'd' => {
            // day of the month
            format_obj["day"] = serde_json::Value::String((if _0.len() == 2 { "2-digit" } else { "numeric" }).to_string());
            "{day}"
        },
        'D' | 'F' | 'g' => {
            // 1..n: Modified Julian day
            format_obj["day"] = serde_json::Value::String("numeric".to_string());
            "{day}"
        },
        // --- Week Day
        'E' => {
            // day of the week
            format_obj["weekday"] = serde_json::Value::String(vec!["short", "short", "short", "long", "narrow", "short"][_0.len() - 1].to_string());
            "{weekday}"
        },
        'e' => {
            // local day of the week
            format_obj["weekday"] = serde_json::Value::String(vec!["numeric", "2-digit", "short", "long", "narrow", "short"][_0.len() - 1].to_string());
            "{weekday}"
        },
        'c' => {
            // stand alone local day of the week
            format_obj["weekday"] = if _0.len() == 2 { serde_json::Value::Null } else { serde_json::Value::String(vec!["numeric", "2-digit", "short", "long", "narrow", "short"][_0.len() - 1].to_string()) };
            "{weekday}"
        },
        // --- Period
        'a' | // AM, PM
        'b' | // am, pm, noon, midnight
        'B' => { // flexible day periods
            format_obj["hour12"] = serde_json::Value::Bool(true);
            "{ampm}"
        },
        // --- Hour
        'h' | 'H' => {
            format_obj["hour"] = serde_json::Value::String((if _0.len() == 2 { "2-digit" } else { "numeric" }).to_string());
            "{hour}"
        },
        'k' | 'K' => {
            format_obj["hour12"] = serde_json::Value::Bool(true); // 12-hour-cycle time formats (using h or K)
            format_obj["hour"] = serde_json::Value::String((if _0.len() == 2 { "2-digit" } else { "numeric" }).to_string());
            "{hour}"
        },
        // --- Minute
        'm' => {
            format_obj["minute"] = serde_json::Value::String((if _0.len() == 2 { "2-digit" } else { "numeric" }).to_string());
            "{minute}"
        },
        // --- Second
        's' => {
            format_obj["second"] = serde_json::Value::String((if _0.len() == 2 { "2-digit" } else { "numeric" }).to_string());
            "{second}"
        },
        'S' | 'A' => {
            format_obj["second"] = serde_json::Value::String("numeric".to_string());
            "{second}"
        },
        // --- Timezone
        'z' | // 1..3, 4: specific non-location format
        'Z' | // 1..3, 4, 5: The ISO8601 varios formats
        'O' | // 1, 4: miliseconds in day short, long
        'v' | // 1, 4: generic non-location format
        'V' | // 1, 2, 3, 4: time zone ID or city
        'X' | // 1, 2, 3, 4: The ISO8601 various formats
        'x' => { // 1, 2, 3, 4: The ISO8601 various formats
            // This polyfill only supports much, for now, we are just doing something dummy.
            format_obj["timeZoneName"] = serde_json::Value::String((if _0.len() < 4 { "short "} else { "long" }).to_string());
            "{timeZoneName}"
        },
        _ => "{}",
    }
}