use super::{
    BasicLanguageInfo, Direction, Country,
    basic_locale_data,
};
use std::{borrow::Borrow, collections::HashMap, fmt::{Display, Formatter}, hash::{Hash, Hasher}, rc::Rc, str::FromStr, sync::Once};
use language_tag::LangTag;

static START: Once = Once::new();
static mut COUNTRY_CODES: Option<HashMap<String, Country>> = None;

pub fn init_static_data() {
    START.call_once(|| unsafe {
        let locale_country_codes_0: HashMap<String, String> =
            serde_json::from_str(&String::from_utf8_lossy(include_bytes!("locale_country_data.json"))).unwrap();
        let locale_country_codes_1: &mut HashMap<String, Country> = &mut HashMap::new();
        for (locale_tag, country_code) in locale_country_codes_0 {
            let country_code = isocountry::CountryCode::for_alpha3(country_code.as_ref());
            if country_code.is_err() {
                continue;
            }
            locale_country_codes_1.insert(locale_tag, Country { _standard_code: country_code.unwrap() });
        }
        COUNTRY_CODES = Some(locale_country_codes_1.clone());
    });
}

fn country_codes() -> &'static HashMap<String, Country> {
    unsafe {
        init_static_data();
        COUNTRY_CODES.as_ref().unwrap().borrow()
    }
}

pub fn parse_locale<S: ToString>(src: S) -> Result<Locale, String> {
    let src = src.to_string();
    let src: &str = src.as_ref();
    let tag = LangTag::from_str(src);
    if tag.is_err() {
        return Err(tag.unwrap_err());
    }
    let mut tag = tag.unwrap();
    if tag.get_region().is_none() {
        let src = src.to_lowercase();
        if src == "br" { tag = LangTag::from_str("pt_BR").unwrap(); }
        if src == "us" { tag = LangTag::from_str("en_US").unwrap(); }
        if src == "jp" { tag = LangTag::from_str("ja_JP").unwrap(); }
    }
    if basic_locale_data().get(&tag.get_language().to_string().replace("-", "")).is_none() {
        return Err(String::from("Invalid locale code."));
    }
    Ok(Locale {
        _tag: Rc::new(tag),
    })
}

#[derive(Clone, Eq)]
pub struct Locale {
    pub(crate) _tag: Rc<LangTag>,
}

impl Locale {
    fn _get_basic_info(&self) -> Option<&BasicLanguageInfo> {
        let langscript = self._tag.get_language().to_string().replace("-", "");
        let langscript: &str = langscript.as_ref();
        basic_locale_data().get(langscript)
    }

    pub fn direction(&self) -> Direction {
        let data = self._get_basic_info();
        if let Some(data) = data { data.direction } else { Direction::LeftToRight }
    }

    pub fn universal_name(&self) -> &str {
        let data = self._get_basic_info();
        if let Some(data) = data { &data.universal_name } else { "" }
    }

    pub fn native_name(&self) -> &str {
        let data = self._get_basic_info();
        if let Some(data) = data { &data.native_name } else { "" }
    }

    pub fn country(&self) -> Option<Country> {
        let tagsrc =
            if self._tag.get_region().is_some() {
                format!("{}{}", self._tag.get_language().to_string(), self._tag.get_region().unwrap().to_string())
            } else { self._tag.get_language().to_string() };
        let r = country_codes().get(&tagsrc);
        if let Some(r) = r { Some(r.clone()) } else { None }
    }

    pub fn standard_tag(&self) -> &LangTag {
        self._tag.as_ref()
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let country = self.country();
        if let Some(country) = country {
            write!(f, "{} ({})", self.native_name(), country.universal_name())
        } else { write!(f, "{}", self.native_name()) }
    }
}

impl PartialEq for Locale {
    fn eq(&self, rhs: &Locale) -> bool {
        self._tag == rhs._tag
    }
}

impl Hash for Locale {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self._tag.to_string().hash(state);
    }
}