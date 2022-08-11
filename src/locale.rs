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

fn get_localization_lang() -> Language {
    let s = match std::fs::read("localization.txt") {
        Err(_) => return Language::ENGLISH,
        Ok(v) => match String::from_utf8(v) {
            Err(_) => return Language::ENGLISH,
            Ok(s) => s,
        },
    };

    let lang_str = s.split('\n').collect::<Vec<&str>>()[0];
    match lang_str {
        "english" => Language::ENGLISH,
        "french" => Language::FRENCH,
        "frenchcanadian" => Language::FRENCHCANADIAN,
        "german" => Language::GERMAN,
        "austrian" => Language::AUSTRIAN,
        "italian" => Language::ITALIAN,
        "spanish" => Language::SPANISH,
        "british" => Language::BRITISH,
        "russian" => Language::RUSSIAN,
        "polish" => Language::POLISH,
        "korean" => Language::KOREAN,
        "japanese" => Language::JAPANESE,
        "czech" => Language::CZECH,
        _ => Language::ENGLISH,
    }
}
