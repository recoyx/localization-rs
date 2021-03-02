use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    convert::TryInto,
    rc::Rc,
};
use super::*;
use super::pluralrules::{PluralCategory, PluralRuleType};
use maplit::{hashmap, hashset};
use lazy_static::lazy_static;
use lazy_regex::regex;

#[derive(Copy, Clone)]
pub enum Gender {
    Male,
    Female,
}

#[macro_export]
/// Creates a `HashMap<String, String>` from a list of key-value pairs.
/// This is based on the [`maplit`](https://github.com/bluss/maplit) crate.
///
/// ## Example
///
/// ```
/// fn main() {
///     let map = localization_vars!{
///         "a" => "foo",
///         "b" => "bar",
///     };
///     assert_eq!(map["a".to_string()], "foo");
///     assert_eq!(map["b".to_string()], "bar");
///     assert_eq!(map.get("c".to_string()), None);
/// }
/// ```
macro_rules! localization_vars {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(localization_vars!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { localization_vars!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = localization_vars!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::<String, String>::with_capacity(_cap);
            $(
                let _ = _map.insert($key.to_string(), $value.to_string());
            )*
            _map
        }
    };
}

pub struct LocaleMap {
    _current_locale: Option<Locale>,
    _current_ordinal_plural_rules: Option<intl_pluralrules::PluralRules>,
    _current_cardinal_plural_rules: Option<intl_pluralrules::PluralRules>,
    _current_timeago_language: Option<Rc<Box<dyn timeago::Language>>>,
    _locale_path_components: Rc<HashMap<Locale, String>>,
    _supported_locales: Rc<HashSet<Locale>>,
    _default_locale: Locale,
    _fallbacks: Rc<HashMap<Locale, Vec<Locale>>>,
    _assets: Rc<RefCell<HashMap<Locale, serde_json::Value>>>,
    _assets_src: String,
    _assets_base_file_names: Vec<String>,
    _assets_auto_clean: bool,
    _assets_loader_type: LocaleMapLoaderType,
}

impl LocaleMap {
    pub fn new(options: &LocaleMapOptions) -> Self {
        let mut locale_path_components = HashMap::<Locale, String>::new();
        let mut supported_locales = HashSet::<Locale>::new();
        for code in options._supported_locales.borrow().iter() {
            let locale_parse = parse_locale(code).unwrap();
            locale_path_components.insert(locale_parse.clone(), code.clone());
            supported_locales.insert(locale_parse);
        }
        let mut fallbacks = HashMap::<Locale, Vec<Locale>>::new();
        for (k, v) in options._fallbacks.borrow().iter() {
            fallbacks.insert(parse_locale(k).unwrap(), v.iter().map(|s| parse_locale(s).unwrap()).collect());
        }
        let default_locale = options._default_locale.borrow().clone();
        Self {
            _current_locale: None,
            _current_cardinal_plural_rules: None,
            _current_ordinal_plural_rules: None,
            _current_timeago_language: None,
            _locale_path_components: Rc::new(locale_path_components),
            _supported_locales: Rc::new(supported_locales),
            _default_locale: parse_locale(&default_locale).unwrap(),
            _fallbacks: Rc::new(fallbacks),
            _assets: Rc::new(RefCell::new(HashMap::new())),
            _assets_src: options._assets.borrow()._src.borrow().clone(),
            _assets_base_file_names: options._assets.borrow()._base_file_names.borrow().iter().map(|s| s.clone()).collect(),
            _assets_auto_clean: options._assets.borrow()._auto_clean.get(),
            _assets_loader_type: options._assets.borrow()._loader_type.get(),
        }
    }

    pub fn supported_locales(&self) -> HashSet<Locale> {
        self._supported_locales.as_ref().clone()
    }

    pub fn supports_locale(&self, arg: &Locale) -> bool {
        self._supported_locales.contains(arg)
    }

    pub fn current_locale(&self) -> Option<Locale> {
        self._current_locale.clone()
    }

    /// Equivalent to `load()` method.
    pub async fn update_locale(&mut self, new_locale: Locale) -> bool {
        self.load(Some(new_locale)).await
    }

    /// Attempts to load specified, current or default locale.
    pub async fn load(&mut self, mut new_locale: Option<Locale>) -> bool {
        if new_locale.is_none() { new_locale = self.current_locale(); }
        if new_locale.is_none() { new_locale = Some(self._default_locale.clone()); }
        let new_locale = new_locale.unwrap();
        if !self.supports_locale(&new_locale) {
            panic!("Unsupported locale {}", new_locale.standard_tag());
        }
        let mut to_load: HashSet<Locale> = hashset![new_locale.clone()];
        self.enumerate_fallbacks(new_locale.clone(), &mut to_load);

        let mut new_assets: HashMap<Locale, serde_json::Value> = hashmap![];
        for locale in to_load {
            let res = self.load_single_locale(&locale).await;
            if res.is_none() {
                return false;
            }
            new_assets.insert(locale.clone(), res.unwrap());
        }

        if self._assets_auto_clean {
            self._assets.borrow_mut().clear();
        }

        let mut assets_output = self._assets.borrow_mut();
        for (locale, root) in new_assets {
            assets_output.insert(locale, root);
        }
        self._current_locale = Some(new_locale.clone());
        let new_locale_code = unic_langid::LanguageIdentifier::from_bytes(new_locale.clone().standard_tag().to_string().as_ref()).unwrap();
        self._current_ordinal_plural_rules = self.load_plural_rules(new_locale_code.clone(), intl_pluralrules::PluralRuleType::ORDINAL);
        self._current_cardinal_plural_rules = self.load_plural_rules(new_locale_code.clone(), intl_pluralrules::PluralRuleType::CARDINAL);
        self._current_timeago_language = None;

        let new_isolang_lang = isolang::Language::from_639_1(new_locale_code.clone().language.as_str()).unwrap();
        let new_timeago_lang = timeago::from_isolang(new_isolang_lang);

        if let Some(l) = new_timeago_lang {
            self._current_timeago_language = Some(Rc::new(l));
        }

        if self._current_timeago_language.is_none() {
            self._current_timeago_language = Some(Rc::new(Box::new(timeago::languages::english::English)));
        }

        true
    }

    fn load_plural_rules(&self, new_locale_code: unic_langid::LanguageIdentifier, prt: intl_pluralrules::PluralRuleType) -> Option<intl_pluralrules::PluralRules> {
        if let Ok(pr) = intl_pluralrules::PluralRules::create(new_locale_code.clone(), prt) {
            Some(pr)
        }
        else if let Ok(pr) = intl_pluralrules::PluralRules::create(unic_langid::LanguageIdentifier::from_parts(new_locale_code.language, None, None, &[]), prt) {
            Some(pr)
        }
        else {
            Some(intl_pluralrules::PluralRules::create(unic_langid::LanguageIdentifier::from_parts(unic_langid::subtags::Language::from_bytes(&"en".as_ref()).unwrap(), None, None, &[]), prt).unwrap())
        }
    }

    async fn load_single_locale(&self, locale: &Locale) -> Option<serde_json::Value> {
        let mut r = serde_json::Value::Object(serde_json::Map::new());
        match self._assets_loader_type {
            LocaleMapLoaderType::FileSystem => {
                for base_name in self._assets_base_file_names.iter() {
                    let locale_path_comp = self._locale_path_components.get(locale);
                    if locale_path_comp.is_none() {
                        panic!("Fallback locale is not supported a locale: {}", locale.standard_tag().to_string());
                    }
                    let res_path = format!("{}/{}/{}.json", self._assets_src, locale_path_comp.unwrap(), base_name);
                    let content = std::fs::read(res_path.clone());
                    if content.is_err() {
                        println!("Failed to load resource at {}.", res_path);
                        return None;
                    }
                    LocaleMap::apply_deep(base_name, serde_json::from_str(String::from_utf8(content.unwrap()).unwrap().as_ref()).unwrap(), &mut r);
                }
            },
            LocaleMapLoaderType::Http => {
                for base_name in self._assets_base_file_names.iter() {
                    let res_path = format!("{}/{}/{}.json", self._assets_src, self._locale_path_components.get(locale).unwrap(), base_name);
                    let content = reqwest::get(reqwest::Url::parse(res_path.clone().as_ref()).unwrap()).await;
                    if content.is_err() {
                        println!("Failed to load resource at {}.", res_path);
                        return None;
                    }
                    let content = if content.is_ok() { Some(content.unwrap().text().await) } else { None };
                    LocaleMap::apply_deep(base_name, serde_json::from_str(content.unwrap().unwrap().as_ref()).unwrap(), &mut r);
                }
            },
        }
        Some(r)
    }

    fn apply_deep(name: &String, assign: serde_json::Value, mut output: &mut serde_json::Value) {
        let mut names: Vec<&str> = name.split("/").collect();
        let last_name = names.pop();
        for name in names {
            let r = output.get(name);
            if r.is_none() || r.unwrap().as_object().is_none() {
                let r = serde_json::Value::Object(serde_json::Map::new());
                output.as_object_mut().unwrap().insert(String::from(name), r);
            }
            output = output.get_mut(name).unwrap();
        }
        output.as_object_mut().unwrap().insert(String::from(last_name.unwrap()), assign);
    }

    fn enumerate_fallbacks(&self, locale: Locale, output: &mut HashSet<Locale>) {
        for list in self._fallbacks.get(&locale).iter() {
            for item in list.iter() {
                output.insert(item.clone());
                self.enumerate_fallbacks(item.clone(), output);
            }
        }
    }

    /// Retrieves message by identifier.
    pub fn get<S: ToString>(&self, id: S) -> String {
        self.get_formatted(id, vec![])
    }

    /// Retrieves message by identifier with formatting arguments.
    pub fn get_formatted<S: ToString>(&self, id: S, options: Vec<&dyn LocaleMapFormatArgument>) -> String {
        let mut variables: Option<HashMap<String, String>> = None;
        let mut gender: Option<Gender> = None;
        let mut amount_u64: Option<u64> = None;
        let mut amount_i64: Option<i64> = None;
        let mut amount_u128: Option<u128> = None;
        let mut amount_i128: Option<i128> = None;
        let mut amount_f64: Option<f64> = None;

        for option in options.iter() {
            if let Some(r) = option.as_gender() {
                gender = Some(r);
            }
            else if let Some(r) = option.as_string_map() {
                variables = Some(r.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            }
            else if let Some(r) = option.as_i64() { amount_i64 = Some(r) }
            else if let Some(r) = option.as_u64() { amount_u64 = Some(r) }
            else if let Some(r) = option.as_i128() { amount_i128 = Some(r) }
            else if let Some(r) = option.as_u128() { amount_u128 = Some(r) }
            else if let Some(r) = option.as_f64() { amount_f64 = Some(r) }
        }

        let mut id = id.to_string();
        if let Some(g) = gender {
            match g {
                Gender::Male => { id.push_str("_male"); },
                Gender::Female => { id.push_str("_female"); }
            }
        }

        if variables.is_none() { variables = Some(HashMap::new()); }
        let mut variables = variables.unwrap();

        // id_empty, id_one, id_multiple and $number variable
        if let Some(qty) = amount_u64 { id.push_str( if qty == 0 { "_empty" } else if qty == 1 { "_one" } else { "_multiple" } ); variables.insert("number".to_string(), qty.to_string()); }
        else if let Some(qty) = amount_i64 { id.push_str( if qty == 0 { "_empty" } else if qty == 1 { "_one" } else { "_multiple" } ); variables.insert("number".to_string(), qty.to_string()); }
        else if let Some(qty) = amount_u128 { id.push_str( if qty == 0 { "_empty" } else if qty == 1 { "_one" } else { "_multiple" } ); variables.insert("number".to_string(), qty.to_string()); }
        else if let Some(qty) = amount_i128 { id.push_str( if qty == 0 { "_empty" } else if qty == 1 { "_one" } else { "_multiple" } ); variables.insert("number".to_string(), qty.to_string()); }
        else if let Some(qty) = amount_f64 { id.push_str( if qty == 0.0 { "_empty" } else if qty == 1.0 { "_one" } else { "_multiple" } ); variables.insert("number".to_string(), qty.to_string()); }

        let id: Vec<String> = id.split(".").map(|s| s.to_string()).collect();
        if self._current_locale.is_none() {
            return id.join(".");
        }
        let r = self.get_formatted_with_locale(self._current_locale.clone().unwrap(), &id, &variables);
        if let Some(r) = r { r } else { id.join(".") }
    }

    fn get_formatted_with_locale(&self, locale: Locale, id: &Vec<String>, vars: &HashMap<String, String>) -> Option<String> {
        let message = self.resolve_id(self._assets.borrow().get(&locale), id);
        if message.is_some() {
            return Some(self.apply_message(message.unwrap(), vars));
        }

        let fallbacks = self._fallbacks.get(&locale);
        if fallbacks.is_some() {
            for fl in fallbacks.unwrap().iter() {
                let r = self.get_formatted_with_locale(fl.clone(), id, vars);
                if r.is_some() {
                    return r;
                }
            }
        }
        None
    }

    fn apply_message(&self, message: String, vars: &HashMap<String, String>) -> String {
        // regex!(r"\$(\$|[A-Za-z0-9_-]+)").replace_all(&message, R { _vars: vars }).as_ref().to_string()
        regex!(r"\$(\$|[A-Za-z0-9_-]+)").replace_all(&message, |s: &regex::Captures<'_>| {
            let s = s.get(0).unwrap().as_str();
            if s == "$" {
                "$"
            } else {
                let v = vars.get(&s.to_string().replace("$", ""));
                if let Some(v) = v { v } else { "undefined" }
            }
        }).as_ref().to_string()
    }

    fn resolve_id(&self, root: Option<&serde_json::Value>, id: &Vec<String>) -> Option<String> {
        let mut r = root;
        for frag in id.iter() {
            if r.is_none() {
                return None;
            }
            r = r.unwrap().get(frag);
        }
        if r.is_none() {
            return None;
        }
        let r = r.unwrap().as_str();
        if let Some(r) = r { Some(r.to_string()) } else { None }
    }

    pub fn select_plural_rule<N: TryInto<intl_pluralrules::operands::PluralOperands>>(&self, prt: PluralRuleType, number: N) -> Result<PluralCategory, &'static str> {
        if prt == PluralRuleType::ORDINAL {
            if let Some(pr) = self._current_ordinal_plural_rules.clone() {
                pr.select::<N>(number)
            }
            else {
                Err(&"Plural rules missing.")
            }
        }
        else {
            if let Some(pr) = self._current_cardinal_plural_rules.clone() {
                pr.select::<N>(number)
            }
            else {
                Err(&"Plural rules missing.")
            }
        }
    }

    pub fn format_relative_time(&self, duration: std::time::Duration) -> String {
        let l = self._current_timeago_language.clone();
        if l.is_none() {
            return "undefined".to_string();
        }
        let l = l.unwrap();
        let secs = duration.as_secs();
        if secs < 60 {
            return l.too_low().to_string();
        }
        let mins = secs / 60;
        if mins < 60 {
            let m = mins.to_string() + " " + l.get_word(timeago::TimeUnit::Minutes, mins);
            let ago = l.ago().to_string();
            return format!("{} {}", if l.place_ago_before() { ago.clone() } else { m.clone() }, if l.place_ago_before() { m } else { ago });
        }
        let hours = mins / 60;
        if hours < 60 {
            let h = hours.to_string() + " " + l.get_word(timeago::TimeUnit::Hours, hours);
            let ago = l.ago().to_string();
            return format!("{} {}", if l.place_ago_before() { ago.clone() } else { h.clone() }, if l.place_ago_before() { h } else { ago });
        }
        let days = hours / 24;
        if days < 30 {
            let d = days.to_string() + " " + l.get_word(timeago::TimeUnit::Days, days);
            let ago = l.ago().to_string();
            return format!("{} {}", if l.place_ago_before() { ago.clone() } else { d.clone() }, if l.place_ago_before() { d } else { ago });
        }
        let weeks = days / 7;
        if weeks < 5 {
            let w = weeks.to_string() + " " + l.get_word(timeago::TimeUnit::Weeks, weeks);
            let ago = l.ago().to_string();
            return format!("{} {}", if l.place_ago_before() { ago.clone() } else { w.clone() }, if l.place_ago_before() { w } else { ago });
        }
        let mut months = weeks / 4;
        if months == 0 {
            months = 1;
        }
        if months < 13 {
            let m = months.to_string() + " " + l.get_word(timeago::TimeUnit::Months, months);
            let ago = l.ago().to_string();
            return format!("{} {}", if l.place_ago_before() { ago.clone() } else { m.clone() }, if l.place_ago_before() { m } else { ago });
        }
        let years = months / 12;
        let y = years.to_string() + " " + l.get_word(timeago::TimeUnit::Years, years);
        let ago = l.ago().to_string();
        return format!("{} {}", if l.place_ago_before() { ago.clone() } else { y.clone() }, if l.place_ago_before() { y } else { ago });
    }
}

impl Clone for LocaleMap {
    fn clone(&self) -> Self {
        Self {
            _current_locale: self._current_locale.clone(),
            _current_cardinal_plural_rules: self._current_cardinal_plural_rules.clone(),
            _current_ordinal_plural_rules: self._current_ordinal_plural_rules.clone(),
            _current_timeago_language: self._current_timeago_language.clone(),
            _locale_path_components: self._locale_path_components.clone(),
            _supported_locales: self._supported_locales.clone(),
            _default_locale: self._default_locale.clone(),
            _fallbacks: self._fallbacks.clone(),
            _assets: self._assets.clone(),
            _assets_src: self._assets_src.clone(),
            _assets_base_file_names: self._assets_base_file_names.clone(),
            _assets_auto_clean: self._assets_auto_clean,
            _assets_loader_type: self._assets_loader_type,
        }
    }
}

pub trait LocaleMapFormatArgument {
    fn as_gender(&self) -> Option<Gender> { None }
    fn as_f64(&self) -> Option<f64> { None }
    fn as_i64(&self) -> Option<i64> { None }
    fn as_u64(&self) -> Option<u64> { None }
    fn as_i128(&self) -> Option<i128> { None }
    fn as_u128(&self) -> Option<u128> { None }
    fn as_string_map(&self) -> Option<HashMap<String, String>> { None }
}

impl LocaleMapFormatArgument for Gender {
    fn as_gender(&self) -> Option<Gender> { Some(*self) }
}

impl LocaleMapFormatArgument for f32 {
    fn as_f64(&self) -> Option<f64> { Some(f64::from(*self)) }
}

impl LocaleMapFormatArgument for f64 {
    fn as_f64(&self) -> Option<f64> { Some(*self) }
}

impl LocaleMapFormatArgument for i32 {
    fn as_i64(&self) -> Option<i64> { Some(i64::from(*self)) }
}

impl LocaleMapFormatArgument for u32 {
    fn as_u64(&self) -> Option<u64> { Some(u64::from(*self)) }
}

impl LocaleMapFormatArgument for i64 {
    fn as_i64(&self) -> Option<i64> { Some(*self) }
}

impl LocaleMapFormatArgument for u64 {
    fn as_u64(&self) -> Option<u64> { Some(*self) }
}

impl LocaleMapFormatArgument for i128 {
    fn as_i128(&self) -> Option<i128> { Some(*self) }
}

impl LocaleMapFormatArgument for u128 {
    fn as_u128(&self) -> Option<u128> { Some(*self) }
}

impl LocaleMapFormatArgument for HashMap<String, String> {
    fn as_string_map(&self) -> Option<HashMap<String, String>> { Some(self.clone()) }
}

pub struct LocaleMapOptions {
    _default_locale: RefCell<String>,
    _supported_locales: RefCell<Vec<String>>,
    _fallbacks: RefCell<HashMap<String, Vec<String>>>,
    _assets: RefCell<LocaleMapAssetOptions>,
}

impl LocaleMapOptions {
    pub fn new() -> Self {
        LocaleMapOptions {
            _default_locale: RefCell::new("en".to_string()),
            _supported_locales: RefCell::new(vec!["en".to_string()]),
            _fallbacks: RefCell::new(hashmap! {}),
            _assets: RefCell::new(LocaleMapAssetOptions::new()),
        }
    }

    pub fn default_locale<S: ToString>(&self, value: S) -> &Self {
        self._default_locale.replace(value.to_string());
        self
    }

    pub fn supported_locales<S: ToString>(&self, list: Vec<S>) -> &Self {
        self._supported_locales.replace(list.iter().map(|name| name.to_string()).collect());
        self
    }

    pub fn fallbacks<S: ToString>(&self, map: HashMap<S, Vec<S>>) -> &Self {
        self._fallbacks.replace(map.iter().map(|(k, v)| (
            k.to_string(),
            v.iter().map(|s| s.to_string()).collect()
        )).collect());
        self
    }

    pub fn assets(&self, options: &LocaleMapAssetOptions) -> &Self {
        self._assets.replace(options.clone());
        self
    }
}

pub struct LocaleMapAssetOptions {
    _src: RefCell<String>,
    _base_file_names: RefCell<Vec<String>>,
    _auto_clean: Cell<bool>,
    _loader_type: Cell<LocaleMapLoaderType>,
}

impl Clone for LocaleMapAssetOptions {
    fn clone(&self) -> Self {
        Self {
            _src: self._src.clone(),
            _base_file_names: self._base_file_names.clone(),
            _auto_clean: self._auto_clean.clone(),
            _loader_type: self._loader_type.clone(),
        }
    }
}

impl LocaleMapAssetOptions {
    pub fn new() -> Self {
        LocaleMapAssetOptions {
            _src: RefCell::new("res/lang".to_string()),
            _base_file_names: RefCell::new(vec![]),
            _auto_clean: Cell::new(true),
            _loader_type: Cell::new(LocaleMapLoaderType::Http),
        }
    }
    
    pub fn src<S: ToString>(&self, src: S) -> &Self {
        self._src.replace(src.to_string());
        self
    } 

    pub fn base_file_names<S: ToString>(&self, list: Vec<S>) -> &Self {
        self._base_file_names.replace(list.iter().map(|name| name.to_string()).collect());
        self
    }

    pub fn auto_clean(&self, value: bool) -> &Self {
        self._auto_clean.set(value);
        self
    }

    pub fn loader_type(&self, value: LocaleMapLoaderType) -> &Self {
        self._loader_type.set(value);
        self
    }
}

#[derive(Copy, Clone)]
pub enum LocaleMapLoaderType {
    FileSystem,
    Http,
}