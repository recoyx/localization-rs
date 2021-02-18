use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};
use std::cell::Cell;
use super::*;
use maplit::{hashmap, hashset};

pub struct LocaleMap {
    _current_locale: Option<Locale>,
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
    pub fn new<'a>(options: &LocaleMapOptions<'a>) -> Self {
        let mut locale_path_components = HashMap::<Locale, String>::new();
        let mut supported_locales = HashSet::<Locale>::new();
        for code in options._supported_locales.borrow().iter() {
            let locale_parse = parse_locale(code).unwrap();
            locale_path_components.insert(locale_parse.clone(), String::from(*code));
            supported_locales.insert(locale_parse);
        }
        let mut fallbacks = HashMap::<Locale, Vec<Locale>>::new();
        for (k, v) in options._fallbacks.borrow().iter() {
            fallbacks.insert(parse_locale(k).unwrap(), v.iter().map(|s| parse_locale(s).unwrap()).collect());
        }
        let default_locale = String::from(options._default_locale.get());
        Self {
            _current_locale: None,
            _locale_path_components: Rc::new(locale_path_components),
            _supported_locales: Rc::new(supported_locales),
            _default_locale: parse_locale(&default_locale).unwrap(),
            _fallbacks: Rc::new(fallbacks),
            _assets: Rc::new(RefCell::new(HashMap::new())),
            _assets_src: String::from(options._assets.borrow()._src.get()),
            _assets_base_file_names: options._assets.borrow()._base_file_names.borrow().iter().map(|&s| String::from(s)).collect(),
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

    pub async fn update_locale(&mut self, new_locale: Locale) {
        self.load(Some(new_locale)).await;
    }

    pub async fn load(&mut self, mut new_locale: Option<Locale>) {
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
            new_assets.insert(locale.clone(), self.load_single_locale(&locale).await);
        }

        if self._assets_auto_clean {
            self._assets.borrow_mut().clear();
        }

        let mut assets_output = self._assets.borrow_mut();
        for (locale, root) in new_assets {
            assets_output.insert(locale, root);
        }
        self._current_locale = Some(new_locale);
    }

    async fn load_single_locale(&self, locale: &Locale) -> serde_json::Value {
        let mut r = serde_json::Value::Object(serde_json::Map::new());
        match self._assets_loader_type {
            LocaleMapLoaderType::FileSystem => {
                for base_name in self._assets_base_file_names.iter() {
                    let res_path = format!("{}/{}/{}.json", self._assets_src, self._locale_path_components.get(locale).unwrap(), base_name);
                    let content = std::fs::read(res_path.clone());
                    if content.is_err() {
                        println!("Failed to load resource at {}.", res_path);
                        continue;
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
                        continue;
                    }
                    let content = if content.is_ok() { Some(content.unwrap().text().await) } else { None };
                    LocaleMap::apply_deep(base_name, serde_json::from_str(content.unwrap().unwrap().as_ref()).unwrap(), &mut r);
                }
            },
        }
        r
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
}

impl Clone for LocaleMap {
    fn clone(&self) -> Self {
        Self {
            _current_locale: self._current_locale.clone(),
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

pub struct LocaleMapOptions<'a> {
    _default_locale: Cell<&'a str>,
    _supported_locales: RefCell<Vec<&'a str>>,
    _fallbacks: RefCell<HashMap<&'a str, Vec<&'a str>>>,
    _assets: RefCell<LocaleMapAssetOptions<'a>>,
}

impl<'a> LocaleMapOptions<'a> {
    pub fn new() -> Self {
        LocaleMapOptions {
            _default_locale: Cell::new("en"),
            _supported_locales: RefCell::new(vec!["en", "en-US"]),
            _fallbacks: RefCell::new(hashmap! {
                "en-US" => vec!["en"]
            }),
            _assets: RefCell::new(LocaleMapAssetOptions::new()),
        }
    }

    pub fn default_locale<S>(&self, value: &'a S) -> &Self where S: AsRef<str> {
        self._default_locale.set(value.as_ref());
        self
    }

    pub fn supported_locales<S>(&self, list: Vec<&'a S>) -> &Self where S: AsRef<str> {
        self._supported_locales.replace(list.iter().map(|&name| name.as_ref()).collect());
        self
    }

    pub fn fallbacks<S>(&self, map: HashMap<&'a S, Vec<&'a S>>) -> &Self where S: AsRef<str> {
        self._fallbacks.replace(map.iter().map(|(&k, v)| (k.as_ref(), v.iter().map(|&s| s.as_ref()).collect())).collect());
        self
    }

    pub fn assets(&self, options: &LocaleMapAssetOptions<'a>) -> &Self {
        self._assets.replace(options.clone());
        self
    }
}

pub struct LocaleMapAssetOptions<'a> {
    _src: Cell<&'a str>,
    _base_file_names: RefCell<Vec<&'a str>>,
    _auto_clean: Cell<bool>,
    _loader_type: Cell<LocaleMapLoaderType>,
}

impl<'a> Clone for LocaleMapAssetOptions<'a> {
    fn clone(&self) -> Self {
        Self {
            _src: self._src.clone(),
            _base_file_names: self._base_file_names.clone(),
            _auto_clean: self._auto_clean.clone(),
            _loader_type: self._loader_type.clone(),
        }
    }
}

impl<'a> LocaleMapAssetOptions<'a> {
    pub fn new() -> Self {
        LocaleMapAssetOptions {
            _src: Cell::new("res/lang"),
            _base_file_names: RefCell::new(vec![]),
            _auto_clean: Cell::new(true),
            _loader_type: Cell::new(LocaleMapLoaderType::Http),
        }
    }
    
    pub fn src<S>(&self, src: &'a S) -> &Self where S: AsRef<str> {
        self._src.set(src.as_ref());
        self
    } 

    pub fn base_file_names<S>(&self, list: Vec<&'a S>) -> &Self where S: AsRef<str> {
        self._base_file_names.replace(list.iter().map(|&name| name.as_ref()).collect());
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