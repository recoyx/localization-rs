use super::CountryCode;

pub fn parse_country<S>(src: &S) -> Result<CountryCode, isocountry::CountryCodeParseErr>
    where S: AsRef<str>
{
    let src: &str = src.as_ref();
    if src.len() == 3 { CountryCode::for_alpha3_caseless(src) } else { CountryCode::for_alpha2_caseless(src) }
}