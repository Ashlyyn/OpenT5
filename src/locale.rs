#![allow(dead_code)]

#[allow(dead_code, clippy::upper_case_acronyms)]
enum Language {
    ENGLISH,
    FRENCH,
    FRENCHCANADIAN,
    GERMAN,
    AUSTRIAN,
    ITALIAN,
    SPANISH,
    BRITISH,
    RUSSIAN,
    POLISH,
    KOREAN,
    JAPANESE,
    CZECH,
}

fn init() -> Language {
    Language::ENGLISH
}
