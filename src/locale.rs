#![allow(dead_code)]

use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use num_derive::FromPrimitive;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Default, FromPrimitive)]
#[repr(u8)]
pub enum Language {
    #[default]
    ENGLISH = 0,
    FRENCH = 1,
    FRENCHCANADIAN = 2,
    GERMAN = 3,
    AUSTRIAN = 4,
    ITALIAN = 5,
    SPANISH = 6,
    BRITISH = 7,
    RUSSIAN = 8,
    POLISH = 9,
    KOREAN = 10,
    JAPANESE = 11,
    CZECH = 12,
    MAX = 13,
}

#[derive(Clone, Default)]
struct Localization {
    language: Language,
    strings: Vec<String>,
}

lazy_static! {
    static ref LOCALIZATION: Arc<RwLock<Localization>> =
        Arc::new(RwLock::new(Default::default()));
}

/// Gets the localization language from localization.txt
pub fn get_lang() -> Language {
    LOCALIZATION.try_read().expect("").language
}

pub fn get_lang_from_str(lang: &str) -> Language {
    match lang {
        // try to match the language string
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
        // if it can't be matched, default to English
        _ => Language::ENGLISH,
    }
}

pub fn get_str_from_lang(lang: Language) -> &'static str {
    match lang {
        // try to match the language
        Language::ENGLISH => "english",
        Language::FRENCH => "french",
        Language::FRENCHCANADIAN => "frenchcanadian",
        Language::GERMAN => "german",
        Language::AUSTRIAN => "austrian",
        Language::ITALIAN => "italian",
        Language::SPANISH => "spanish",
        Language::BRITISH => "british",
        Language::RUSSIAN => "russian",
        Language::POLISH => "polish",
        Language::KOREAN => "korean",
        Language::JAPANESE => "japanese",
        Language::CZECH => "czech",
        // if it can't be matched, default to ""
        _ => "",
    }
}

pub fn init() -> Language {
    // try to read localization.txt
    let (lang, strings) = match std::fs::read("localization.txt") {
        // if read fails, default to English
        Err(_) => (Language::ENGLISH, Vec::<String>::new()),
        // if it succeeds, try to copy it into a String
        Ok(v) => match String::from_utf8(v) {
            // if copy fails, default to English
            Err(_) => (Language::ENGLISH, Vec::<String>::new()),
            // if it succeeds, get the string
            Ok(s) => {
                // trim the whitespace from the file, then split it by newline
                // and collect the strings
                let file_strings = s.trim().split('\n').collect::<Vec<&str>>();
                // the language string should be at the beginning
                // of the file, a single word followed by a newline
                let lang = get_lang_from_str(file_strings[0]);
                // collect the rest of the strings for LOCALIZATION.strings
                let strings = file_strings[1..]
                    .to_vec()
                    .iter()
                    .map(|&s| s.to_string())
                    .collect::<Vec<String>>();
                (lang, strings)
            }
        },
    };
    *LOCALIZATION.clone().try_write().expect("") = Localization {
        language: lang,
        strings,
    };
    lang
}

pub fn localize_ref(s: &str) -> String {
    s.to_string()
}
