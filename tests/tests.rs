use recoyx_localization::*;
use maplit::*;
use futures_await_test::async_test;

#[test]
fn locale_country() {
    let some_lang = parse_locale(&"pt-BR").unwrap();
    let some_country = some_lang.country();
    assert_eq!(some_lang.to_string(), String::from("Português (Brazil)"));
    assert_eq!(some_lang.standard_tag().to_string(), String::from("pt-BR"));
    assert!(some_country.is_some());
    assert_eq!(some_country.unwrap().standard_code().alpha3(), "BRA");
}

#[async_test]
async fn locale_map() {
    let mut locale_map = LocaleMap::new(
        LocaleMapOptions::new()
            .supported_locales(vec!["en-US"])
            .default_locale("en-US")
            .assets(LocaleMapAssetOptions::new()
                .src("tests/res")
                .base_file_names(vec!["common"])
                .auto_clean(true)
                .loader_type(LocaleMapLoaderType::FileSystem))
    ); // locale_map
    locale_map.load(None).await;
    assert!(locale_map.supports_locale(&parse_locale("en-US").unwrap()));
    assert_eq!(locale_map.format_relative_time(std::time::Duration::from_secs(10 * 60 * 60 * 24)), "1 week ago");
}