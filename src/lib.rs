mod basic_language_info;
use basic_language_info::{
    global_basic_locale_data, BasicLanguageInfo,
};
pub use basic_language_info::Direction;

pub use isocountry::CountryCode;

mod locale;
pub use locale::{Locale, parse_locale};

mod country;
pub use country::{Country, parse_country};

mod locale_map;
pub use locale_map::{
    LocaleMap, LocaleMapOptions, LocaleMapAssetOptions,
    LocaleMapLoaderType,
};

#[cfg(test)]
mod tests;