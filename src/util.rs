use caseless::Caseless;
use unicode_normalization::UnicodeNormalization;

pub fn compatibility_case_fold(s: &str) -> String {
    s.nfd()
        .default_case_fold()
        .nfkd()
        .default_case_fold()
        .nfkd()
        .collect()
}
