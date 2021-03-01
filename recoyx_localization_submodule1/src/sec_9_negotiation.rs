//! Sect 9.2 Abstract Operations
//  ============================

use std::{collections::{HashMap}};
use super::errors::LocaleError;
use super::sec_6_locales_currencies_tz::{
    is_structurally_valid_language_tag,
    canonicalize_language_tag,
    default_locale,
};
use regex::Regex;
use lazy_static::lazy_static;
use lazy_regex::regex;

lazy_static! {
    static ref EXP_UNICODE_EX_SEQ: Regex = Regex::new(r"(?i)-u(?:-[0-9a-z]{2,8})+").unwrap();
}

pub fn /* 9.2.1 */ canonicalize_locale_list<S: ToString>(locales: Vec<S>) -> Result<Vec<String>, LocaleError> {
    let locales: Vec<String> = locales.iter().map(|e| e.to_string()).collect();
    let mut seen = vec![];
    let len = locales.len();
    let mut k = 0;
    while k < len {
        let k_present = locales.get(k);
        if let Some(tag) = k_present {
            if !is_structurally_valid_language_tag(tag.clone()) {
                return Err(LocaleError::InvalidTagStructure(tag.clone()));
            }
            let tag = canonicalize_language_tag(tag.clone());
            if !seen.contains(&tag.clone()) {
                seen.push(tag);
            }
        }
        k += 1;
    }

    Ok(seen)
}

/// The BestAvailableLocale abstract operation compares the provided argument
/// locale, which must be a String value with a structurally valid and
/// canonicalized BCP 47 language tag, against the locales in availableLocales and
/// returns either the longest non-empty prefix of locale that is an element of
/// availableLocales, or undefined if there is no such element. It uses the
/// fallback mechanism of RFC 4647, section 3.4.
pub fn /* 9.2.2 */ best_available_locale<Sa: ToString, Sb: ToString>(available_locales: Vec<Sa>, locale: Option<Sb>) -> Option<String> {
    let available_locales: Vec<String> = available_locales.iter().map(|s| s.to_string()).collect();
    let locale = if let Some(s) = locale { Some(s.to_string()) } else { None };
    let mut candidate = locale;

    while candidate.clone().is_some() {
        if available_locales.contains(&candidate.clone().unwrap()) {
            return candidate.clone();
        }
        let c = candidate.clone().unwrap();
        let hyphens: Vec<regex::Match> = regex!(r"-").find_iter(&c).collect();
        let pos = if let Some(h) = hyphens.last() { Some(h.start()) } else { None };
        if let Some(mut pos) = pos {
            let candidate_chars: Vec<char> = candidate.unwrap().chars().collect();
            if pos >= 2 && candidate_chars[pos - 2] == '-' {
                pos -= 2;
            }
            candidate = Some(candidate_chars[..pos].iter().map(|&ch| String::from(ch)).collect());
        }
    }
    None
}

/// The LookupMatcher abstract operation compares requestedLocales, which must be
/// a List as returned by CanonicalizeLocaleList, against the locales in
/// availableLocales and determines the best available language to meet the
/// request. The following steps are taken:
pub fn /* 9.2.3 */ lookup_matcher<Sa, Sb>(available_locales: Vec<Sa>, requested_locales: Vec<Sb>) -> HashMap<String, String>
    where Sa: ToString, Sb: ToString
{
    let available_locales: Vec<String> = available_locales.iter().map(|s| s.to_string()).collect();
    let requested_locales: Vec<String> = requested_locales.iter().map(|s| s.to_string()).collect();
    let mut i = 0;
    let len = requested_locales.len();
    let mut available_locale: Option<String> = None;
    let mut locale: Option<String> = None;
    let mut no_extensions_locale: Option<String> = None;

    while i < len && available_locale.is_none() {
        locale = Some(requested_locales[i].clone());
        no_extensions_locale = Some(EXP_UNICODE_EX_SEQ.replace_all(locale.clone().unwrap().as_ref(), "").into());
        available_locale = best_available_locale(available_locales.clone(), no_extensions_locale.clone());
        i += 1;
    }

    let mut result = HashMap::new();
    if let Some(available_locale) = available_locale {
        result.insert("[[locale]]".to_string(), available_locale.clone());

        if locale.clone().unwrap() != no_extensions_locale.clone().unwrap() {
            let locale = locale.clone().unwrap();
            let extension = &EXP_UNICODE_EX_SEQ.captures(locale.as_ref()).unwrap()[0];
            let extension_index = if let Some(m) = regex!(r"-u-").find(locale.as_ref()) { m.start() } else { 0 };
            result.insert("[[extension]]".to_string(), extension.to_string());
            result.insert("[[extensionIndex]]".to_string(), extension_index.to_string());
        }
        else if let Some(locale) = default_locale() {
            result.insert("[[locale]]".to_string(), locale);
        }
    }
    result
}

/**
 * The BestFitMatcher abstract operation compares requestedLocales, which must be
 * a List as returned by CanonicalizeLocaleList, against the locales in
 * availableLocales and determines the best available language to meet the
 * request. The algorithm is implementation dependent, but should produce results
 * that a typical user of the requested locales would perceive as at least as
 * good as those produced by the LookupMatcher abstract operation. Options
 * specified through Unicode locale extension sequences must be ignored by the
 * algorithm. Information about such subsequences is returned separately.
 * The abstract operation returns a record with a [[locale]] field, whose value
 * is the language tag of the selected locale, which must be an element of
 * availableLocales. If the language tag of the request locale that led to the
 * selected locale contained a Unicode locale extension sequence, then the
 * returned record also contains an [[extension]] field whose value is the first
 * Unicode locale extension sequence, and an [[extensionIndex]] field whose value
 * is the index of the first Unicode locale extension sequence within the request
 * locale language tag.
 */
pub fn /* 9.2.4 */ best_fit_matcher<Sa, Sb>(available_locales: Vec<Sa>, requested_locales: Vec<Sb>) -> HashMap<String, String>
    where Sa: ToString, Sb: ToString
{
    lookup_matcher(available_locales, requested_locales)
}

pub fn unicode_extension_subtags<S: ToString>(extension: S) -> Vec<String> {
    let extension = extension.to_string();
    let size = extension.len();
    if size == 0 {
        return vec![];
    }
    let mut extension_subtags = vec![];
    let mut attribute = true;
    let mut q = 3;
    let mut p = q;
    let mut t = q;
    let extension_chars: Vec<char> = extension.chars().collect();
    while q < size {
        let c = extension_chars[q];
        if c == '\x2d' {
            if q - p == 2 {
                if p - t > 1 {
                    let _type = &extension[t..(p - 1)];
                    extension_subtags.push(_type.to_string());
                }
                let key = &extension[p..q];
                extension_subtags.push(key.to_string());
                t = q + 1;
                attribute = false;
            } else if attribute {
                let attr = &extension[p..q];
                extension_subtags.push(attr.to_string());
                t = q + 1;
            }
            p = q + 1;
        }
        q = q + 1;
    }
    if size - p == 2 {
        if p - t > 1 {
            let _type = &extension[t..(p - 1)];
            extension_subtags.push(_type.to_string());
        }
        t = p;
    }
    let tail = &extension[t..size];
    extension_subtags.push(tail.to_string());
    extension_subtags
}

/// The ResolveLocale abstract operation compares a BCP 47 language priority list
/// requestedLocales against the locales in availableLocales and determines the
/// best available language to meet the request. availableLocales and
/// requestedLocales must be provided as List values, options as a Record.
pub fn /* 9.2.5 */ resolve_locale<Sa, Sb, Sc>(available_locales: Vec<Sa>, requested_locales: Vec<Sb>, options: &HashMap<String, String>, relevant_extension_keys: Vec<Sc>, locale_data: &serde_json::Value) -> HashMap<String, String>
    where Sa: ToString, Sb: ToString, Sc: ToString
{
    let available_locales: Vec<String> = available_locales.iter().map(|s| s.to_string()).collect();
    let requested_locales: Vec<String> = requested_locales.iter().map(|s| s.to_string()).collect();
    let relevant_extension_keys: Vec<String> = relevant_extension_keys.iter().map(|s| s.to_string()).collect();

    if available_locales.len() == 0 {
        panic!("No locale data has been provided for this object yet.");
    }

    let matcher = options.get("[[localeMatcher]]").unwrap();
    let mut r: Option<HashMap<String, String>> = None;

    if matcher == "lookup" {
        r = Some(lookup_matcher(available_locales, requested_locales));
    } else {
        r = Some(best_fit_matcher(available_locales, requested_locales));
    }

    let r = r.unwrap();
    let found_locale = &r["[[locale]]"];

    let mut extension_subtags: Option<Vec<String>> = None;
    let mut extension_subtags_len = 0;

    if let Some(extension) = r.get("[[extension]]") {
        extension_subtags = Some(unicode_extension_subtags(extension));
        extension_subtags_len = extension_subtags.unwrap().len();
    }

    let mut result = HashMap::<String, String>::new();
    result.insert("[[dataLocale]]".to_string(), found_locale.clone());

    let mut i = 0;
    let len = relevant_extension_keys.len();

    while i < len {
        let key = relevant_extension_keys[i].clone();
        let found_locale_data = locale_data[found_locale];
        let key_locale_data = found_locale_data[key];
        let value = key_locale_data[];
    }

    result
}