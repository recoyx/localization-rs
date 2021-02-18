# Localization

Flexible localization for Rust.

Features:

- `LocaleMap`
  - Load assets from HTTP and File System.
- General language code and country code manipulation.
  - `Locale` and `parse_locale(str)`
  - `Country` and `parse_country(str)`

## Getting started

Add the following dependencies to Cargo.toml:

```toml
[dependencies]
recoyx_localization = "0.1"
maplit = "1.0"
```

```rust
use recoyx_localization::{
    LocaleMap, LocaleMapOptions, LocaleMapAssetOptions,
    LocaleMapLoaderType,
    localization_vars,
};
use maplit::hashmap;

fn main() {
    let locale_map = LocaleMap::new(
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
                .src("path/to/res/lang")
                .base_file_names(vec!["common", "validation"])
                // "auto_clean" indicates whether to clean previous unused locale data. 
                .auto_clean(true)
                // Specify LocaleMapLoaderType::FileSystem or LocaleMapLoaderType::Http
                .loader_type(LocaleMapLoaderType::FileSystem))
    ); // locale_map
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
```