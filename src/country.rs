use std::fmt::{Display, Formatter};

#[derive(PartialEq, Clone)]
pub struct Country {
    pub(crate) _standard_code: isocountry::CountryCode,
}

impl Country {
    pub fn standard_code(&self) -> isocountry::CountryCode {
        self._standard_code.clone()
    }

    pub fn universal_name(&self) -> &str {
        self._standard_code.name()
    }
}

impl Display for Country {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self._standard_code.to_string())
    }
}

pub fn parse_country<S: ToString>(src: S) -> Result<Country, isocountry::CountryCodeParseErr> {
    let src = src.to_string();
    let src: &str = src.as_ref();
    let r = if src.len() == 3 { isocountry::CountryCode::for_alpha3_caseless(src) } else { isocountry::CountryCode::for_alpha2_caseless(src) };
    if let Ok(r) = r { Ok(Country { _standard_code: r }) } else { Err(r.unwrap_err()) }
}