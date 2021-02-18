use super::*;
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
            // Specify supported locale codes.
            // The form in which the locale code appears here
            // is a post-component for the assets "src" path. 
            // For example: "path/to/res/lang/en-US"
            .supported_locales(vec!["en", "en-US", "pt-BR"])
            .default_locale("en-US")
            .fallbacks(hashmap! {
                "en-US" => vec!["en"],
                "pt-BR" => vec!["en-US"],
            })
            .assets(LocaleMapAssetOptions::new()
                .src("src/test_res")
                .base_file_names(vec!["common"])
                // "auto_clean" indicates whether to clean previous unused locale data. 
                .auto_clean(true)
                // Specify LocaleMapLoaderType::FileSystem or LocaleMapLoaderType::Http
                .loader_type(LocaleMapLoaderType::FileSystem))
    ); // locale_map
    assert!(locale_map.supports_locale(&parse_locale("en-US").unwrap()));
    locale_map.load(None).await;
    println!("{}", locale_map.get("common.message_id"));
    println!("{}", locale_map.get_formatted("common.parameterized", vec![ &localization_vars!{
        "x" => "foo"
    } ]));
    println!("{}", locale_map.get_formatted("common.contextual", vec![ &Gender::Female ]));
    for i in 0..3 {
        println!("{}", locale_map.get_formatted("common.qty", vec![ &i ]));
    }
}