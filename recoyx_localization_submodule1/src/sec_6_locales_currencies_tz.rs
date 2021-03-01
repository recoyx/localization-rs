use std::{borrow::Borrow, collections::HashMap};
use super::exp::*;
use maplit::hashmap;
use regex::Regex;
use lazy_static::lazy_static;
use lazy_regex::regex;

lazy_static! {
    // IANA Subtag Registry redundant tag and subtag maps
    static ref REDUNDANT_TAGS: HashMap<&'static str, &'static str> = hashmap! {
        "art-lojban" => "jbo",
        "i-ami" => "ami",
        "i-bnn" => "bnn",
        "i-hak" => "hak",
        "i-klingon" => "tlh",
        "i-lux" => "lb",
        "i-navajo" => "nv",
        "i-pwn" => "pwn",
        "i-tao" => "tao",
        "i-tay" => "tay",
        "i-tsu" => "tsu",
        "no-bok" => "nb",
        "no-nyn" => "nn",
        "sgn-BE-FR" => "sfb",
        "sgn-BE-NL" => "vgt",
        "sgn-CH-DE" => "sgg",
        "zh-guoyu" => "cmn",
        "zh-hakka" => "hak",
        "zh-min-nan" => "nan",
        "zh-xiang" => "hsn",
        "sgn-BR" => "bzs",
        "sgn-CO" => "csn",
        "sgn-DE" => "gsg",
        "sgn-DK" => "dsl",
        "sgn-ES" => "ssp",
        "sgn-FR" => "fsl",
        "sgn-GB" => "bfi",
        "sgn-GR" => "gss",
        "sgn-IE" => "isg",
        "sgn-IT" => "ise",
        "sgn-JP" => "jsl",
        "sgn-MX" => "mfs",
        "sgn-NI" => "ncs",
        "sgn-NL" => "dse",
        "sgn-NO" => "nsl",
        "sgn-PT" => "psr",
        "sgn-SE" => "swl",
        "sgn-US" => "ase",
        "sgn-ZA" => "sfs",
        "zh-cmn" => "cmn",
        "zh-cmn-Hans" => "cmn-Hans",
        "zh-cmn-Hant" => "cmn-Hant",
        "zh-gan" => "gan",
        "zh-wuu" => "wuu",
        "zh-yue" => "yue",
    };
    static ref REDUNDANT_SUBTAGS: HashMap<&'static str, &'static str> = hashmap! {
        "BU" => "MM",
        "DD" => "DE",
        "FX" => "FR",
        "TP" => "TL",
        "YD" => "YE",
        "ZR" => "CD",
        "heploc" => "alalc97",
        "in" => "id",
        "iw" => "he",
        "ji" => "yi",
        "jw" => "jv",
        "mo" => "ro",
        "ayx" => "nun",
        "bjd" => "drl",
        "ccq" => "rki",
        "cjr" => "mom",
        "cka" => "cmr",
        "cmk" => "xch",
        "drh" => "khk",
        "drw" => "prs",
        "gav" => "dev",
        "hrr" => "jal",
        "ibi" => "opa",
        "kgh" => "kml",
        "lcq" => "ppr",
        "mst" => "mry",
        "myt" => "mry",
        "sca" => "hle",
        "tie" => "ras",
        "tkk" => "twm",
        "tlw" => "weo",
        "tnf" => "prs",
        "ybd" => "rki",
        "yma" => "lrr",
    };
    static ref REDUNDANT_TAGS_EXT_LANG: HashMap<&'static str, Vec<&'static str>> = hashmap! {
        "aao" => vec!["aao", "ar"],
        "abh" => vec!["abh", "ar"],
        "abv" => vec!["abv", "ar"],
        "acm" => vec!["acm", "ar"],
        "acq" => vec!["acq", "ar"],
        "acw" => vec!["acw", "ar"],
        "acx" => vec!["acx", "ar"],
        "acy" => vec!["acy", "ar"],
        "adf" => vec!["adf", "ar"],
        "ads" => vec!["ads", "sgn"],
        "aeb" => vec!["aeb", "ar"],
        "aec" => vec!["aec", "ar"],
        "aed" => vec!["aed", "sgn"],
        "aen" => vec!["aen", "sgn"],
        "afb" => vec!["afb", "ar"],
        "afg" => vec!["afg", "sgn"],
        "ajp" => vec!["ajp", "ar"],
        "apc" => vec!["apc", "ar"],
        "apd" => vec!["apd", "ar"],
        "arb" => vec!["arb", "ar"],
        "arq" => vec!["arq", "ar"],
        "ars" => vec!["ars", "ar"],
        "ary" => vec!["ary", "ar"],
        "arz" => vec!["arz", "ar"],
        "ase" => vec!["ase", "sgn"],
        "asf" => vec!["asf", "sgn"],
        "asp" => vec!["asp", "sgn"],
        "asq" => vec!["asq", "sgn"],
        "asw" => vec!["asw", "sgn"],
        "auz" => vec!["auz", "ar"],
        "avl" => vec!["avl", "ar"],
        "ayh" => vec!["ayh", "ar"],
        "ayl" => vec!["ayl", "ar"],
        "ayn" => vec!["ayn", "ar"],
        "ayp" => vec!["ayp", "ar"],
        "bbz" => vec!["bbz", "ar"],
        "bfi" => vec!["bfi", "sgn"],
        "bfk" => vec!["bfk", "sgn"],
        "bjn" => vec!["bjn", "ms"],
        "bog" => vec!["bog", "sgn"],
        "bqn" => vec!["bqn", "sgn"],
        "bqy" => vec!["bqy", "sgn"],
        "btj" => vec!["btj", "ms"],
        "bve" => vec!["bve", "ms"],
        "bvl" => vec!["bvl", "sgn"],
        "bvu" => vec!["bvu", "ms"],
        "bzs" => vec!["bzs", "sgn"],
        "cdo" => vec!["cdo", "zh"],
        "cds" => vec!["cds", "sgn"],
        "cjy" => vec!["cjy", "zh"],
        "cmn" => vec!["cmn", "zh"],
        "coa" => vec!["coa", "ms"],
        "cpx" => vec!["cpx", "zh"],
        "csc" => vec!["csc", "sgn"],
        "csd" => vec!["csd", "sgn"],
        "cse" => vec!["cse", "sgn"],
        "csf" => vec!["csf", "sgn"],
        "csg" => vec!["csg", "sgn"],
        "csl" => vec!["csl", "sgn"],
        "csn" => vec!["csn", "sgn"],
        "csq" => vec!["csq", "sgn"],
        "csr" => vec!["csr", "sgn"],
        "czh" => vec!["czh", "zh"],
        "czo" => vec!["czo", "zh"],
        "doq" => vec!["doq", "sgn"],
        "dse" => vec!["dse", "sgn"],
        "dsl" => vec!["dsl", "sgn"],
        "dup" => vec!["dup", "ms"],
        "ecs" => vec!["ecs", "sgn"],
        "esl" => vec!["esl", "sgn"],
        "esn" => vec!["esn", "sgn"],
        "eso" => vec!["eso", "sgn"],
        "eth" => vec!["eth", "sgn"],
        "fcs" => vec!["fcs", "sgn"],
        "fse" => vec!["fse", "sgn"],
        "fsl" => vec!["fsl", "sgn"],
        "fss" => vec!["fss", "sgn"],
        "gan" => vec!["gan", "zh"],
        "gds" => vec!["gds", "sgn"],
        "gom" => vec!["gom", "kok"],
        "gse" => vec!["gse", "sgn"],
        "gsg" => vec!["gsg", "sgn"],
        "gsm" => vec!["gsm", "sgn"],
        "gss" => vec!["gss", "sgn"],
        "gus" => vec!["gus", "sgn"],
        "hab" => vec!["hab", "sgn"],
        "haf" => vec!["haf", "sgn"],
        "hak" => vec!["hak", "zh"],
        "hds" => vec!["hds", "sgn"],
        "hji" => vec!["hji", "ms"],
        "hks" => vec!["hks", "sgn"],
        "hos" => vec!["hos", "sgn"],
        "hps" => vec!["hps", "sgn"],
        "hsh" => vec!["hsh", "sgn"],
        "hsl" => vec!["hsl", "sgn"],
        "hsn" => vec!["hsn", "zh"],
        "icl" => vec!["icl", "sgn"],
        "ils" => vec!["ils", "sgn"],
        "inl" => vec!["inl", "sgn"],
        "ins" => vec!["ins", "sgn"],
        "ise" => vec!["ise", "sgn"],
        "isg" => vec!["isg", "sgn"],
        "isr" => vec!["isr", "sgn"],
        "jak" => vec!["jak", "ms"],
        "jax" => vec!["jax", "ms"],
        "jcs" => vec!["jcs", "sgn"],
        "jhs" => vec!["jhs", "sgn"],
        "jls" => vec!["jls", "sgn"],
        "jos" => vec!["jos", "sgn"],
        "jsl" => vec!["jsl", "sgn"],
        "jus" => vec!["jus", "sgn"],
        "kgi" => vec!["kgi", "sgn"],
        "knn" => vec!["knn", "kok"],
        "kvb" => vec!["kvb", "ms"],
        "kvk" => vec!["kvk", "sgn"],
        "kvr" => vec!["kvr", "ms"],
        "kxd" => vec!["kxd", "ms"],
        "lbs" => vec!["lbs", "sgn"],
        "lce" => vec!["lce", "ms"],
        "lcf" => vec!["lcf", "ms"],
        "liw" => vec!["liw", "ms"],
        "lls" => vec!["lls", "sgn"],
        "lsg" => vec!["lsg", "sgn"],
        "lsl" => vec!["lsl", "sgn"],
        "lso" => vec!["lso", "sgn"],
        "lsp" => vec!["lsp", "sgn"],
        "lst" => vec!["lst", "sgn"],
        "lsy" => vec!["lsy", "sgn"],
        "ltg" => vec!["ltg", "lv"],
        "lvs" => vec!["lvs", "lv"],
        "lzh" => vec!["lzh", "zh"],
        "max" => vec!["max", "ms"],
        "mdl" => vec!["mdl", "sgn"],
        "meo" => vec!["meo", "ms"],
        "mfa" => vec!["mfa", "ms"],
        "mfb" => vec!["mfb", "ms"],
        "mfs" => vec!["mfs", "sgn"],
        "min" => vec!["min", "ms"],
        "mnp" => vec!["mnp", "zh"],
        "mqg" => vec!["mqg", "ms"],
        "mre" => vec!["mre", "sgn"],
        "msd" => vec!["msd", "sgn"],
        "msi" => vec!["msi", "ms"],
        "msr" => vec!["msr", "sgn"],
        "mui" => vec!["mui", "ms"],
        "mzc" => vec!["mzc", "sgn"],
        "mzg" => vec!["mzg", "sgn"],
        "mzy" => vec!["mzy", "sgn"],
        "nan" => vec!["nan", "zh"],
        "nbs" => vec!["nbs", "sgn"],
        "ncs" => vec!["ncs", "sgn"],
        "nsi" => vec!["nsi", "sgn"],
        "nsl" => vec!["nsl", "sgn"],
        "nsp" => vec!["nsp", "sgn"],
        "nsr" => vec!["nsr", "sgn"],
        "nzs" => vec!["nzs", "sgn"],
        "okl" => vec!["okl", "sgn"],
        "orn" => vec!["orn", "ms"],
        "ors" => vec!["ors", "ms"],
        "pel" => vec!["pel", "ms"],
        "pga" => vec!["pga", "ar"],
        "pks" => vec!["pks", "sgn"],
        "prl" => vec!["prl", "sgn"],
        "prz" => vec!["prz", "sgn"],
        "psc" => vec!["psc", "sgn"],
        "psd" => vec!["psd", "sgn"],
        "pse" => vec!["pse", "ms"],
        "psg" => vec!["psg", "sgn"],
        "psl" => vec!["psl", "sgn"],
        "pso" => vec!["pso", "sgn"],
        "psp" => vec!["psp", "sgn"],
        "psr" => vec!["psr", "sgn"],
        "pys" => vec!["pys", "sgn"],
        "rms" => vec!["rms", "sgn"],
        "rsi" => vec!["rsi", "sgn"],
        "rsl" => vec!["rsl", "sgn"],
        "sdl" => vec!["sdl", "sgn"],
        "sfb" => vec!["sfb", "sgn"],
        "sfs" => vec!["sfs", "sgn"],
        "sgg" => vec!["sgg", "sgn"],
        "sgx" => vec!["sgx", "sgn"],
        "shu" => vec!["shu", "ar"],
        "slf" => vec!["slf", "sgn"],
        "sls" => vec!["sls", "sgn"],
        "sqk" => vec!["sqk", "sgn"],
        "sqs" => vec!["sqs", "sgn"],
        "ssh" => vec!["ssh", "ar"],
        "ssp" => vec!["ssp", "sgn"],
        "ssr" => vec!["ssr", "sgn"],
        "svk" => vec!["svk", "sgn"],
        "swc" => vec!["swc", "sw"],
        "swh" => vec!["swh", "sw"],
        "swl" => vec!["swl", "sgn"],
        "syy" => vec!["syy", "sgn"],
        "tmw" => vec!["tmw", "ms"],
        "tse" => vec!["tse", "sgn"],
        "tsm" => vec!["tsm", "sgn"],
        "tsq" => vec!["tsq", "sgn"],
        "tss" => vec!["tss", "sgn"],
        "tsy" => vec!["tsy", "sgn"],
        "tza" => vec!["tza", "sgn"],
        "ugn" => vec!["ugn", "sgn"],
        "ugy" => vec!["ugy", "sgn"],
        "ukl" => vec!["ukl", "sgn"],
        "uks" => vec!["uks", "sgn"],
        "urk" => vec!["urk", "ms"],
        "uzn" => vec!["uzn", "uz"],
        "uzs" => vec!["uzs", "uz"],
        "vgt" => vec!["vgt", "sgn"],
        "vkk" => vec!["vkk", "ms"],
        "vkt" => vec!["vkt", "ms"],
        "vsi" => vec!["vsi", "sgn"],
        "vsl" => vec!["vsl", "sgn"],
        "vsv" => vec!["vsv", "sgn"],
        "wuu" => vec!["wuu", "zh"],
        "xki" => vec!["xki", "sgn"],
        "xml" => vec!["xml", "sgn"],
        "xmm" => vec!["xmm", "ms"],
        "xms" => vec!["xms", "sgn"],
        "yds" => vec!["yds", "sgn"],
        "ysl" => vec!["ysl", "sgn"],
        "yue" => vec!["yue", "zh"],
        "zib" => vec!["zib", "sgn"],
        "zlm" => vec!["zlm", "ms"],
        "zmi" => vec!["zmi", "ms"],
        "zsl" => vec!["zsl", "sgn"],
        "zsm" => vec!["zsm", "ms"],
    };
}

/// Converts only a-z to uppercase as per section 6.1 of the spec.
pub fn to_latin_uppercase(s: &str) -> String {
    regex!(r"[a-z]").replace_all(s, |cap: &regex::Captures| cap[0].to_uppercase()).into()
}

/// The IsStructurallyValidLanguageTag abstract operation verifies that the locale
/// argument (which must be a String value)
///
/// - represents a well-formed BCP 47 language tag as specified in RFC 5646 section
///   2.1, or successor,
/// - does not include duplicate variant subtags, and
/// - does not include duplicate singleton subtags.
///
/// The abstract operation returns true if locale can be generated from the ABNF
/// grammar in section 2.1 of the RFC, starting with Language-Tag, and does not
/// contain duplicate variant or singleton subtags (other than as a private use
/// subtag). It returns false otherwise. Terminal value characters in the grammar are
/// interpreted as the Unicode equivalents of the ASCII octet values given.
pub fn is_structurally_valid_language_tag(locale: String) -> bool {
    // represents a well-formed BCP 47 language tag as specified in RFC 5646
    if !EXP_BCP47_SYNTAX.is_match(locale.as_ref()) {
        false
    // does not include duplicate variant subtags, and
    } else if EXP_VARIANT_DUPES.is_match(locale.as_ref()) {
        false
    // // does not include duplicate singleton subtags.
    } else if EXP_SINGLETON_DUPES.is_match(locale.as_ref()) {
        false
    } else {
        true
    }
}

/// The CanonicalizeLanguageTag abstract operation returns the canonical and case-
/// regularized form of the locale argument (which must be a String value that is
/// a structurally valid BCP 47 language tag as verified by the
/// IsStructurallyValidLanguageTag abstract operation). It takes the steps
/// specified in RFC 5646 section 4.5, or successor, to bring the language tag
/// into canonical form, and to regularize the case of the subtags, but does not
/// take the steps to bring a language tag into “extlang form” and to reorder
/// variant subtags.
/// The specifications for extensions to BCP 47 language tags, such as RFC 6067,
/// may include canonicalization rules for the extension subtag sequences they
/// define that go beyond the canonicalization rules of RFC 5646 section 4.5.
/// Implementations are allowed, but not required, to apply these additional rules.
pub fn canonicalize_language_tag(locale: String) -> String {
    // A language tag is in 'canonical form' when the tag is well-formed
    // according to the rules in Sections 2.1 and 2.2

    // Section 2.1 says all subtags use lowercase...
    let locale = locale.to_lowercase();

    // ...with 2 exceptions: 'two-letter and four-letter subtags that neither
    // appear at the start of the tag nor occur after singletons.  Such two-letter
    // subtags are all uppercase (as in the tags "en-CA-x-ca" or "sgn-BE-FR") and
    // four-letter subtags are titlecase (as in the tag "az-Latn-x-latn").
    let mut parts: Vec<String> = locale.split("-").map(|s| s.to_string()).collect();
    for i in 1..parts.len() {
        // Two-letter subtags are all uppercase
        if parts[i].len() == 2 {
            parts[i] = parts[i].to_uppercase();
        }
        // Four-letter subtags are titlecase
        else if parts[i].len() == 4 {
            let chars = parts[i].chars().collect::<Vec<char>>();
            let p1 = String::from(chars[0]).to_uppercase();
            let p2 = chars[1..].iter().map(|&ch| String::from(ch)).collect::<Vec<String>>().join("");
            parts[i] = p1 + p2.as_ref();
        }
        // Is it a singleton?
        else if parts[i].len() == 1 && parts[i] != "x" {
            break;
        }
    }
    let mut locale = parts.join("-");

    // The steps laid out in RFC 5646 section 4.5 are as follows:

    // 1.  Extension sequences are ordered into case-insensitive ASCII order
    //     by singleton subtag.
    let mut _match: Vec<&str> = vec![];
    for caps in EXP_EXT_SEQUENCES.captures_iter(&locale) {
        for s in caps.iter() {
            _match.push(s.unwrap().as_str());
        }
    }
    if _match.len() > 1 {
        // The built-in sort() sorts by ASCII order, so use that
        _match.sort();

        // Replace all extensions with the joined, sorted array
        locale = Regex::new((r"(?i)(?:".to_string() + EXP_EXT_SEQUENCES.to_string().as_ref() + ")+").as_ref()).unwrap().replace::<&str>(&locale, _match.join("").as_ref()).into();
    }

    // 2.  Redundant or grandfathered tags are replaced by their 'Preferred-
    //     Value', if there is one.
    if REDUNDANT_TAGS.contains_key::<str>(locale.as_ref()) {
        locale = REDUNDANT_TAGS[locale.as_ref() as &str].to_string();
    }

    // 3.  Subtags are replaced by their 'Preferred-Value', if there is one.
    //     For extlangs, the original primary language subtag is also
    //     replaced if there is a primary language subtag in the 'Preferred-
    //     Value'.
    let mut parts: Vec<&str> = locale.split("-").collect();
    let mut i = 0;
    let mut max = parts.len();
    while i < max {
        if REDUNDANT_SUBTAGS.contains_key(parts[i]) {
            parts[i] = REDUNDANT_SUBTAGS[parts[i]];
        }
        else if REDUNDANT_TAGS_EXT_LANG.contains_key(parts[i]) {
            parts[i] = REDUNDANT_TAGS_EXT_LANG[parts[i]][0];

            // For extlang tags, the prefix needs to be removed if it is redundant
            if i == 1 && REDUNDANT_TAGS_EXT_LANG[parts[1]][1] == parts[0] {
                parts = parts[i..].iter().map(|&v| v).collect();
                i += 1;
                max -= 1;
            }
        }
        i += 1;
    }

    parts.join("-").to_string()
}

static mut DEFAULT_LOCALE: Option<String> = None;

/**
 * The DefaultLocale abstract operation returns a String value representing the
 * structurally valid (6.2.2) and canonicalized (6.2.3) BCP 47 language tag for the
 * host environment’s current locale.
 */
pub fn /* 6.2.4 */ default_locale() -> Option<String> {
    unsafe { DEFAULT_LOCALE.clone() }
}

pub fn set_default_locale(locale: Option<String>) {
    unsafe { DEFAULT_LOCALE = locale; };
}

// Sect 6.3 Currency Codes
// =======================

lazy_static! {
    static ref EXP_CURRENCY_CODE: Regex = Regex::new(r"^[A-Z]{3}$").unwrap();
}

/// The IsWellFormedCurrencyCode abstract operation verifies that the currency argument
/// (after conversion to a String value) represents a well-formed 3-letter ISO currency
/// code. The following steps are taken:
pub fn is_well_formed_currency_code<S: ToString>(currency: S) -> bool {
    // 1. Let `c` be ToString(currency)
    let c = currency.to_string();

    // 2. Let `normalized` be the result of mapping c to upper case as described
    //    in 6.1.
    let normalized = to_latin_uppercase(c.as_ref());

    // 3. If the string length of normalized is not 3, return false.
    // 4. If normalized contains any character that is not in the range "A" to "Z"
    //    (U+0041 to U+005A), return false.
    if !EXP_CURRENCY_CODE.is_match(normalized.as_ref()) {
        return false;
    }

    // 5. Return true
    true
}