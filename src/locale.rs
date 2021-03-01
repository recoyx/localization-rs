use super::{
    LocaleBasicData, Direction, Country,
    LOCALE_BASIC_DATA,
};
use std::{fmt::{Display, Formatter}, hash::{Hash, Hasher}, rc::Rc, str::FromStr};
use language_tag::LangTag;

/// Parses a locale code. If the given string is a valid language tag but its
/// language subtag is not a known language, an error is returned instead.
///
/// Some region codes are specially translated into the correct language identifier,
/// such as from `jp` to `ja` and `br` to `pt-BR`.
//
///
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
        if src == "jp" { tag = LangTag::from_str("ja").unwrap(); }
    }
    if LOCALE_BASIC_DATA.get(&tag.get_language().to_string().replace("-", "")).is_none() {
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
    fn _get_basic_info(&self) -> Option<&LocaleBasicData> {
        let langscript = self._tag.get_language().to_string().replace("-", "");
        let langscript: &str = langscript.as_ref();
        LOCALE_BASIC_DATA.get(langscript)
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
        if let Some(r) = self.standard_tag().get_region() {
            let r = isocountry::CountryCode::for_alpha2_caseless((&r.to_string()).as_ref());
            if let Ok(r) = r {
                return Some(Country { _standard_code: r });
            }
        }
        let s = self.standard_tag().to_string();
        if s == "fr" { return Some(Country { _standard_code: isocountry::CountryCode::for_alpha3_caseless(&"FRA").unwrap() }); }
        if s == "ja" { return Some(Country { _standard_code: isocountry::CountryCode::for_alpha3_caseless(&"JPN").unwrap() }); }
        if s == "ru" { return Some(Country { _standard_code: isocountry::CountryCode::for_alpha3_caseless(&"RUS").unwrap() }); }
        None
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