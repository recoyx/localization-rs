use std::{collections::{HashMap}};
use regex::Regex;
use lazy_static::lazy_static;
use lazy_regex::regex;
use maplit::hashmap;

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

fn exp_dt_components_meta<'a>(_0: &str, format_obj: &mut serde_json::Value) -> &'a str {
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
        _ => "",
    }
}

/// Converts the CLDR availableFormats into the objects and patterns required by
/// the ECMAScript Internationalization API specification.
pub fn create_date_time_format(skeleton: String, pattern: String) -> serde_json::Value {
    // We ignore certain patterns that are unsupported to avoid this expensive operation.
    if UNWANTED_DTCS.is_match(pattern.clone().as_ref()) {
        return serde_json::Value::Null;
    }
    
    let mut format_obj = serde_json::Value::Object(serde_json::Map::new());
    format_obj["originalPattern"] = serde_json::Value::String(pattern.clone());
    format_obj["_"] = serde_json::Value::Object(serde_json::Map::new());

    // Replace the pattern string with the one required by the specification, whilst
    // at the same time evaluating it for the subsets and formats
    format_obj["extendedPattern"] = serde_json::Value::String(EXP_DT_COMPONENTS.replace_all(pattern.as_ref(), |cap: &regex::Captures| {
        exp_dt_components_meta(&cap[0], &mut format_obj["_"]).clone()
    }).into());

    // Match the skeleton string with the one required by the specification
    // this implementation is based on the Date Field Symbol Table:
    // http://unicode.org/reports/tr35/tr35-dates.html#Date_Field_Symbol_Table
    // Note: we are adding extra data to the formatObject even though this polyfill
    //       might not support it.
    EXP_DT_COMPONENTS.replace_all(skeleton.as_ref(), |cap: &regex::Captures| {
        exp_dt_components_meta(&cap[0], &mut format_obj)
    });

    compute_final_patterns(&mut format_obj).clone()
}

/// Processes DateTime formats from CLDR to an easier-to-parse format.
/// the result of this operation should be cached the first time a particular
/// calendar is analyzed.
///
/// The specification requires we support at least the following subsets of
/// date/time components:
///
///   - 'weekday', 'year', 'month', 'day', 'hour', 'minute', 'second'
///   - 'weekday', 'year', 'month', 'day'
///   - 'year', 'month', 'day'
///   - 'year', 'month'
///   - 'month', 'day'
///   - 'hour', 'minute', 'second'
///   - 'hour', 'minute'
///
/// We need to cherry pick at least these subsets from the CLDR data and convert
/// them into the pattern objects used in the ECMA-402 API.
pub fn create_date_time_formats(formats: &serde_json::Value) -> serde_json::Value {
    let available_formats = &formats["availableFormats"].as_object().unwrap();
    let time_formats = &formats["timeFormats"].as_object().unwrap();
    let date_formats = &formats["dateFormats"].as_object().unwrap();
    let mut result: Vec<serde_json::Value> = vec![];
    let mut time_related_formats: Vec<serde_json::Value> = vec![];
    let mut date_related_formats: Vec<serde_json::Value> = vec![];

    // Map available (custom) formats into a pattern for create_date_time_formats
    for (skeleton, pattern) in available_formats.iter() {
        let computed = create_date_time_format(skeleton.clone(), pattern.as_str().unwrap().to_string());
        if computed.is_object() {
            result.push(computed.clone());
            // in some cases, the format is only displaying date specific props
            // or time specific props, in which case we need to also produce the
            // combined formats.
            if is_date_format_only(&computed) {
                date_related_formats.push(computed);
            } else if is_time_format_only(&computed) {
                time_related_formats.push(computed);
            }
        }
    }

    // Map time formats into a pattern for create_date_time_formats
    for (skeleton, pattern) in time_formats.iter() {
        let computed = create_date_time_format(skeleton.clone(), pattern.as_str().unwrap().to_string());
        if computed.is_object() {
            result.push(computed.clone());
            time_related_formats.push(computed);
        }
    }

    // Map date formats into a pattern for create_date_time_formats
    for (skeleton, pattern) in date_formats.iter() {
        let computed = create_date_time_format(skeleton.clone(), pattern.as_str().unwrap().to_string());
        if computed.is_object() {
            result.push(computed.clone());
            date_related_formats.push(computed);
        }
    }

    // combine custom time and custom date formats when they are orthogonals to complete the
    // formats supported by CLDR.
    // This Algo is based on section "Missing Skeleton Fields" from:
    // http://unicode.org/reports/tr35/tr35-dates.html#availableFormats_appendItems

    for i in 0..time_related_formats.len() {
        for j in 0..date_related_formats.len() {
            let mut pattern: String = "".to_string();
            if date_related_formats[j].get("month").is_some() && date_related_formats[j]["month"] == serde_json::Value::String("long".to_string()) {
                pattern = formats[if date_related_formats[j].get("weekday").is_some() { "full" } else { "long" }].as_str().unwrap().to_string();
            } else if date_related_formats[j].get("month").is_some() && date_related_formats[j]["month"] == serde_json::Value::String("short".to_string()) {
                pattern = formats["medium"].as_str().unwrap().to_string();
            } else {
                pattern = formats["short"].as_str().unwrap().to_string();
            }
            let mut computed = join_date_and_time_formats(&date_related_formats[j], &time_related_formats[i]);
            computed["originalPattern"] = serde_json::Value::String(pattern.clone());
            computed["extendedPattern"] = serde_json::Value::String(regex!(r"(?i)^[,\s]+|[,\s]+$").replace_all(
                pattern
                    .replace("{0}", time_related_formats[i]["extendedPattern"].as_str().unwrap())
                    .replace("{1}", date_related_formats[j]["extendedPattern"].as_str().unwrap())
                    .as_ref(),
            "").into());
            result.push(compute_final_patterns(&mut computed).clone());
        }
    }

    serde_json::Value::Array(result)
}

lazy_static! {
    // this represents the exceptions of the rule that are not covered by CLDR availableFormats
    // for single property configurations, they play no role when using multiple properties, and
    // those that are not in this table, are not exceptions or are not covered by the data we
    // provide.
    static ref VALID_SYNTHETIC_PROPS: HashMap<String, HashMap<String, String>> = (hashmap! {
        "second" => hashmap! {
            "numeric" => "s",
            "2-digit" => "ss"
        },
        "minute" => hashmap! {
            "numeric" => "m",
            "2-digit" => "mm"
        },
        "year" => hashmap! {
            "numeric" => "y",
            "2-digit" => "yy"
        },
        "day" => hashmap! {
            "numeric" => "d",
            "2-digit" => "dd"
        },
        "month" => hashmap! {
            "numeric" => "L",
            "2-digit" => "LL",
            "narrow" => "LLLLL",
            "short" => "LLL",
            "long" => "LLLL"
        },
        "weekday" => hashmap! {
            "narrow" => "ccccc",
            "short" => "ccc",
            "long" => "cccc"
        },
    })
        .iter().map(|(&k, v)| {
            (String::from(k), v.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
        }).collect();
}

pub fn generate_synthetic_format(prop_name: String, prop_value: serde_json::Value) -> serde_json::Value {
    if VALID_SYNTHETIC_PROPS.get::<String>(&prop_name).is_some() && VALID_SYNTHETIC_PROPS[&prop_name.clone()].get(prop_value.as_str().unwrap()).is_some() {
        let mut m = serde_json::Map::new();
        m.insert("originalPattern".to_string(), serde_json::Value::String(VALID_SYNTHETIC_PROPS[&prop_name.clone()][prop_value.as_str().unwrap()].to_string()));
        let mut um = serde_json::Map::new();
        um[&prop_name] = prop_value.clone();
        m.insert("_".to_string(), serde_json::Value::Object(um));
        m.insert("extendedPattern".to_string(), serde_json::Value::String(format!("{{{}}}", prop_name.clone())));
        m.insert(prop_name.clone(), prop_value);
        m.insert("pattern12".to_string(), serde_json::Value::String(format!("{{{}}}", prop_name.clone())));
        m.insert("pattern".to_string(), serde_json::Value::String(format!("{{{}}}", prop_name)));
        return serde_json::Value::Object(m);
    }
    serde_json::Value::Null
}