#![allow(dead_code)]

use core::fmt::Display;
use std::{collections::HashMap, sync::RwLock};

use core::fmt::Write;
extern crate alloc;

use lazy_static::lazy_static;
use num::Integer;
use num_derive::FromPrimitive;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, FromPrimitive)]
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

impl Display for Language {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match *self {
            // try to match the language
            Self::ENGLISH => "english",
            Self::FRENCH => "french",
            Self::FRENCHCANADIAN => "frenchcanadian",
            Self::GERMAN => "german",
            Self::AUSTRIAN => "austrian",
            Self::ITALIAN => "italian",
            Self::SPANISH => "spanish",
            Self::BRITISH => "british",
            Self::RUSSIAN => "russian",
            Self::POLISH => "polish",
            Self::KOREAN => "korean",
            Self::JAPANESE => "japanese",
            Self::CZECH => "czech",
            // if it can't be matched, default to ""
            _ => "",
        };
        write!(f, "{}", s)
    }
}

impl Language {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn try_from_u8(value: u8) -> Option<Self> {
        num::FromPrimitive::from_u8(value)
    }
}

#[derive(Clone, Default)]
struct Localization {
    language: Language,
    strings: HashMap<String, String>,
}

lazy_static! {
    static ref LOCALIZATION: RwLock<Localization> =
        RwLock::new(Localization::default());
}

/// Gets the localization language from localization.txt
pub fn get_lang() -> Language {
    LOCALIZATION.read().unwrap().language
}

#[allow(clippy::match_same_arms)]
pub fn lang_from_str(lang: &str) -> Option<Language> {
    match lang {
        // try to match the language string
        "english" => Some(Language::ENGLISH),
        "french" => Some(Language::FRENCH),
        "frenchcanadian" => Some(Language::FRENCHCANADIAN),
        "german" => Some(Language::GERMAN),
        "austrian" => Some(Language::AUSTRIAN),
        "italian" => Some(Language::ITALIAN),
        "spanish" => Some(Language::SPANISH),
        "british" => Some(Language::BRITISH),
        "russian" => Some(Language::RUSSIAN),
        "polish" => Some(Language::POLISH),
        "korean" => Some(Language::KOREAN),
        "japanese" => Some(Language::JAPANESE),
        "czech" => Some(Language::CZECH),
        // if it can't be matched, default to English
        _ => None,
    }
}

pub fn init() -> Language {
    // try to read localization.txt
    // if read fails, default to English
    // if it succeeds, try to copy it into a String
    let (lang, strings) = std::fs::read("localization.txt").map_or_else(
        |_| (Language::ENGLISH, Vec::<String>::new()),
        |v| {
            let s = String::from_utf8_lossy(&v);
            // the language string should be at the beginning
            // of the file, a single word followed by a newline
            let strings = s.trim().split('\n').collect::<Vec<&str>>();
            let lang = lang_from_str(strings.first().unwrap());
            // collect the rest of the strings for LOCALIZATION.strings
            // trim the whitespace from the file,
            // then split it by quotation marks
            // and collect the strings
            let mut t = String::new();
            let file_strings: Vec<&str> = strings.get(1..).unwrap().to_vec();
            file_strings
                .iter()
                .for_each(|&s| writeln!(t, "{}", s).unwrap());

            let strings = t
                .split('"')
                .collect::<Vec<&str>>()
                .iter()
                .map(|&s| s.to_owned().trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>();
            (lang.unwrap_or(Language::ENGLISH), strings)
        },
    );

    let keys: Vec<String> = strings
        .iter()
        .enumerate()
        .filter(|(i, _)| i.is_even())
        .map(|(_, s)| s.clone())
        .collect();
    let values: Vec<String> = strings
        .iter()
        .enumerate()
        .filter(|(i, _)| i.is_odd())
        .map(|(_, s)| s.clone())
        .collect();

    let mut map: HashMap<String, String> = HashMap::new();
    for i in 0..keys.len() {
        map.insert(
            keys.get(i).unwrap().clone(),
            values.get(i).unwrap().clone(),
        );
    }

    *LOCALIZATION.write().unwrap() = Localization {
        language: lang,
        strings: map,
    };
    lang
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn localize_ref(s: &str) -> String {
    LOCALIZATION
        .read()
        .unwrap()
        .strings
        .clone()
        .get(&s.to_owned())
        .map_or_else(|| s.to_owned(), |s| s.clone())
}
