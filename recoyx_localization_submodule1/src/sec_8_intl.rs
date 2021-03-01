use super::errors::LocaleError;
use super::sec_9_negotiation::{canonicalize_locale_list};

/// 8.2.1
pub fn get_canonical_locales<S: ToString>(locales: Vec<S>) -> Result<Vec<String>, LocaleError> {
    let locales = locales.iter().map(|e| e.to_string()).collect();

    // 1. Let ll be ? CanonicalizeLocaleList(locales).
    let ll = canonicalize_locale_list(locales);
    if ll.is_err() {
        return ll;
    }
    let ll = ll.unwrap();
    // 2. Return CreateArrayFromList(ll).
    {
        let mut result = vec![];
        let len = ll.len();
        let mut k = 0;
        while k < len {
            result[k] = ll[k].clone();
            k += 1;
        }
        Ok(result)
    }
}