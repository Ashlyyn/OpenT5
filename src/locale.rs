#![allow(dead_code)]

use std::{sync::{Arc, RwLock}, collections::HashMap};

use lazy_static::lazy_static;
use num::Integer;
use num_derive::FromPrimitive;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Default, Debug, FromPrimitive)]
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
    strings: HashMap<String, String>,
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
        Ok(v) => {
            let s = String::from_utf8_lossy(&v);
            // the language string should be at the beginning
            // of the file, a single word followed by a newline
            let strings = s.trim().split('\n').collect::<Vec<&str>>();
            let lang = get_lang_from_str(strings[0]);
            // collect the rest of the strings for LOCALIZATION.strings
            // trim the whitespace from the file, 
            // then split it by quotation marks
            // and collect the strings
            let mut t = String::new();
            let file_strings: Vec<&str> = strings[1..].to_vec();
            file_strings.iter().for_each(|&s| t.push_str(&format!("{}\n", s)));

            let strings = t.split('"').collect::<Vec<&str>>()
                .iter()
                .map(|&s| s.to_string().trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>();
            (lang, strings)
        },
    };

    let keys: Vec<String> = strings.iter().enumerate().filter(|(i, _)| i.is_even()).map(|(_, s)| s.clone()).collect();
    let values: Vec<String> = strings.iter().enumerate().filter(|(i, _)| i.is_odd()).map(|(_, s)| s.clone()).collect();

    let mut map: HashMap<String, String> = HashMap::new();
    for i in 0..keys.len() {
        map.insert(keys[i].clone(), values[i].clone());
    }

    *LOCALIZATION.clone().try_write().expect("") = Localization {
        language: lang,
        strings: map,
    };
    lang
}

pub fn localize_ref(s: &str) -> String {
    let lock = LOCALIZATION.clone();
    let reader = match lock.read() {
        Ok(r) => r,
        Err(_) => return s.to_string(),
    };

    let strings = reader.strings.clone();
    match strings.get(&s.to_string()) {
        Some(s) => s.clone(),
        None => s.to_string()
    }
}