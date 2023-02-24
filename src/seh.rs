#![allow(dead_code)]

use crate::{locale::Language, *};

use lazy_static::lazy_static;

lazy_static! {
    static ref G_CURRENT_ASIAN: AtomicBool = AtomicBool::new(false);
}

pub fn update_current_language() {
    let lang: Language =
        num::FromPrimitive::from_i32(dvar::get_int("loc_language").unwrap())
            .unwrap();
    match lang {
        Language::RUSSIAN => {
            dvar::set_bool("cg_subtitles", false).unwrap();
        }
        Language::KOREAN | Language::JAPANESE => {
            G_CURRENT_ASIAN.store(true, Ordering::SeqCst);
        }
        _ => {}
    };
    dvar::set_string("language", locale::get_str_from_lang(lang)).unwrap();
}

#[allow(clippy::as_conversions)]
pub fn init_language() {
    dvar::register_int(
        "loc_language",
        Language::ENGLISH as _,
        Some(Language::ENGLISH as _),
        Some(Language::MAX as _),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Language"),
    )
    .unwrap();

    dvar::register_bool(
        "loc_forceEnglish",
        false,
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Force english localized strings"),
    )
    .unwrap();

    dvar::register_bool(
        "loc_translate",
        true,
        dvar::DvarFlags::LATCHED,
        Some("Enable translations"),
    )
    .unwrap();

    dvar::register_bool(
        "loc_warnings",
        false,
        dvar::DvarFlags::empty(),
        Some("Enable localization warnings"),
    )
    .unwrap();

    dvar::register_bool(
        "loc_warningsAsErrors",
        false,
        dvar::DvarFlags::empty(),
        Some("Throw an error for any unlocalized string"),
    )
    .unwrap();

    update_current_language();
}
