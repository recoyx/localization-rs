//! Defines regular expressions for various operations related to the BCP 47 syntax,
//! as defined at http://tools.ietf.org/html/bcp47#section-2.1

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // extlang       = 3ALPHA              ; selected ISO 639 codes
    //                 *2("-" 3ALPHA)      ; permanently reserved
    static ref EXTLANG: String = "[a-z]{3}(?:-[a-z]{3}){0,2}".to_string();

    // language      = 2*3ALPHA            ; shortest ISO 639 code
    //                 ["-" extlang]       ; sometimes followed by
    //                                     ; extended language subtags
    //               / 4ALPHA              ; or reserved for future use
    //               / 5*8ALPHA            ; or registered language subtag
    static ref LANGUAGE: String = "(?:[a-z]{2,3}(?:-".to_string() + &EXTLANG + ")?|[a-z]{4}|[a-z]{5,8})";

    // script        = 4ALPHA              ; ISO 15924 code
    static ref SCRIPT: String = "[a-z]{4}".to_string();

    // region        = 2ALPHA              ; ISO 3166-1 code
    //               / 3DIGIT              ; UN M.49 code
    static ref REGION: String = "(?:[a-z]{2}|\\d{3})".to_string();

    // variant       = 5*8alphanum         ; registered variants
    //               / (DIGIT 3alphanum)
    static ref VARIANT: String = "(?:[a-z0-9]{5,8}|\\d[a-z0-9]{3})".to_string();

    //                                     ; Single alphanumerics
    //                                     ; "x" reserved for private use
    // singleton     = DIGIT               ; 0 - 9
    //               / %x41-57             ; A - W
    //               / %x59-5A             ; Y - Z
    //               / %x61-77             ; a - w
    //               / %x79-7A             ; y - z
    static ref SINGLETON: String = "[0-9a-wy-z]".to_string();

    // extension     = singleton 1*("-" (2*8alphanum))
    static ref EXTENSION: String = SINGLETON.clone() + "(?:-[a-z0-9]{2,8})+";

    // privateuse    = "x" 1*("-" (1*8alphanum))
    static ref PRIVATEUSE: String = "x(?:-[a-z0-9]{1,8})+".to_string();

    // irregular     = "en-GB-oed"         ; irregular tags do not match
    //               / "i-ami"             ; the 'langtag' production and
    //               / "i-bnn"             ; would not otherwise be
    //               / "i-default"         ; considered 'well-formed'
    //               / "i-enochian"        ; These tags are all valid,
    //               / "i-hak"             ; but most are deprecated
    //               / "i-klingon"         ; in favor of more modern
    //               / "i-lux"             ; subtags or subtag
    //               / "i-mingo"           ; combination
    //               / "i-navajo"
    //               / "i-pwn"
    //               / "i-tao"
    //               / "i-tay"
    //               / "i-tsu"
    //               / "sgn-BE-FR"
    //               / "sgn-BE-NL"
    //               / "sgn-CH-DE"
    static ref IRREGULAR: String = "(?:en-GB-oed".to_string()
            + "|i-(?:ami|bnn|default|enochian|hak|klingon|lux|mingo|navajo|pwn|tao|tay|tsu)"
            + "|sgn-(?:BE-FR|BE-NL|CH-DE))";

    // regular       = "art-lojban"        ; these tags match the 'langtag'
    //               / "cel-gaulish"       ; production, but their subtags
    //               / "no-bok"            ; are not extended language
    //               / "no-nyn"            ; or variant subtags: their meaning
    //               / "zh-guoyu"          ; is defined by their registration
    //               / "zh-hakka"          ; and all of these are deprecated
    //               / "zh-min"            ; in favor of a more modern
    //               / "zh-min-nan"        ; subtag or sequence of subtags
    //               / "zh-xiang"
    static ref REGULAR: String = "(?:art-lojban|cel-gaulish|no-bok|no-nyn".to_string()
            + "|zh-(?:guoyu|hakka|min|min-nan|xiang))";

    // grandfathered = irregular           ; non-redundant tags registered
    //               / regular             ; during the RFC 3066 era
    static ref GRANDFATHERED: String = "(?:".to_string() + &IRREGULAR + "|" + &REGULAR + ")";

    // langtag       = language
    //                 ["-" script]
    //                 ["-" region]
    //                 *("-" variant)
    //                 *("-" extension)
    //                 ["-" privateuse]
    static ref LANGTAG: String = format!("{}(?:-{})?(?:-{})?(?:-{})*(?:-{})*(?:-{})?",
        LANGUAGE.clone(), SCRIPT.clone(), REGION.clone(), VARIANT.clone(), EXTENSION.clone(), PRIVATEUSE.clone());

    // Language-Tag  = langtag             ; normal language tags
    //               / privateuse          ; private use tag
    //               / grandfathered       ; grandfathered tags
    pub static ref EXP_BCP47_SYNTAX: Regex = Regex::new(("(?i)^(?:".to_string()+&LANGTAG+"|"+&PRIVATEUSE+"|"+&GRANDFATHERED+")$").as_ref()).unwrap();

    // Match duplicate variants in a language tag
    pub static ref EXP_VARIANT_DUPES: Regex = Regex::new(("(?i)^(?!x).*?-(".to_string()+&VARIANT+")-(?:\\w{4,8}-(?!x-))*\\1\\b").as_ref()).unwrap();

    // Match duplicate singletons in a language tag (except in private use)
    pub static ref EXP_SINGLETON_DUPES: Regex = Regex::new(("(?i)^(?!x).*?-(".to_string()+&SINGLETON+")-(?:\\w+-(?!x-))*\\1\\b").as_ref()).unwrap();

    // Match all extension sequences (use regex.replace_all())
    pub static ref EXP_EXT_SEQUENCES: Regex = Regex::new(("(?i)-".to_string()+&EXTENSION).as_ref()).unwrap();
}