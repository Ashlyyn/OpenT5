use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicIsize;
use core::sync::atomic::Ordering;

use crate::cmd;
use crate::com;
use crate::sys;

use super::add_flags;
use super::get_bool;
use super::get_enumeration;
use super::global_fns::exists;
use super::global_fns::find;
use super::name_is_valid;
use super::register_bool;
use super::register_color;
use super::register_float;
use super::register_int;
use super::set_bool_from_source;
use super::set_float_from_source;
use super::set_int64_from_source;
use super::set_int_from_source;
use super::set_string_from_source;
use super::value::DvarValue;
use super::Dvar;
use super::DvarFlags;
use super::SetSource;
use super::DVARS;

use lazy_static::lazy_static;

// Toggle current value of Dvar if possible
#[allow(
    clippy::todo,
    clippy::match_same_arms,
    clippy::panic_in_result_fn,
    clippy::too_many_lines
)]
fn toggle_simple(name: &str) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    let value = find(name).unwrap().current;
    match value {
        DvarValue::Bool(b) => {
            set_bool_from_source(name, !b, SetSource::External)
        }
        DvarValue::Float(f) => {
            let domain = find(name).unwrap().domain;
            if domain.as_float_limits().unwrap().min > 0.0
                || domain.as_float_limits().unwrap().max < 1.0
            {
                if (value.as_float().unwrap()
                    - domain.as_float_limits().unwrap().min)
                    .abs()
                    < f32::EPSILON
                {
                    set_float_from_source(
                        name,
                        domain.as_float_limits().unwrap().max,
                        SetSource::External,
                    )
                } else {
                    set_float_from_source(
                        name,
                        domain.as_float_limits().unwrap().min,
                        SetSource::External,
                    )
                }
            } else if f == 0.0 {
                set_float_from_source(name, 1.0, SetSource::External)
            } else {
                set_float_from_source(name, 0.0, SetSource::External)
            }
        }
        DvarValue::Int(i) => {
            let domain = find(name).unwrap().domain;
            if domain.as_int_limits().unwrap().max > 0
                && domain.as_int_limits().unwrap().min < 1
            {
                if i == 0 {
                    set_int_from_source(name, 1, SetSource::External)
                } else {
                    set_int_from_source(name, 0, SetSource::External)
                }
            } else if i == domain.as_int_limits().unwrap().min {
                set_int_from_source(
                    name,
                    domain.as_int_limits().unwrap().max,
                    SetSource::External,
                )
            } else {
                set_int_from_source(
                    name,
                    domain.as_int_limits().unwrap().min,
                    SetSource::External,
                )
            }
        }
        DvarValue::Int64(i) => {
            let domain = find(name).unwrap().domain;
            if domain.as_int64_limits().unwrap().max > 0
                && domain.as_int64_limits().unwrap().min < 1
            {
                if i == 0 {
                    set_int64_from_source(name, 1, SetSource::External)
                } else {
                    set_int64_from_source(name, 0, SetSource::External)
                }
            } else if i == domain.as_int64_limits().unwrap().min {
                set_int64_from_source(
                    name,
                    domain.as_int64_limits().unwrap().max,
                    SetSource::External,
                )
            } else {
                set_int64_from_source(
                    name,
                    domain.as_int64_limits().unwrap().min,
                    SetSource::External,
                )
            }
        }
        DvarValue::Vector2(_) => {
            com::println!(
                0.into(),
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name,
            );
            Err(())
        }
        DvarValue::Vector3(_) => {
            com::println!(
                0.into(),
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name,
            );
            Err(())
        }
        DvarValue::Vector4(_) => {
            com::println!(
                0.into(),
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name,
            );
            Err(())
        }
        DvarValue::String(_) => {
            com::println!(
                0.into(),
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name,
            );
            Err(())
        }
        DvarValue::Color(_) => {
            com::println!(
                0.into(),
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name,
            );
            Err(())
        }
        DvarValue::LinearColorRGB(_) => {
            com::println!(
                0.into(),
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name,
            );
            Err(())
        }
        DvarValue::ColorXYZ(_) => {
            com::println!(
                0.into(),
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name,
            );
            Err(())
        }
        DvarValue::Enumeration(_) => {
            todo!();
        }
    }
}

fn index_string_to_enum_string(
    name: &str,
    index_string: &str,
) -> Option<String> {
    let Some(dvar) = find(name) else { return None };

    if dvar
        .domain
        .as_enumeration_limits()
        .unwrap()
        .strings
        .is_empty()
    {
        return None;
    }

    if index_string.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    match index_string.parse::<usize>() {
        Ok(i) => {
            if i == 0
                || i >= dvar
                    .domain
                    .as_enumeration_limits()
                    .unwrap()
                    .strings
                    .len()
            {
                None
            } else {
                Some(
                    dvar.domain
                        .as_enumeration_limits()
                        .unwrap()
                        .strings
                        .iter()
                        .nth(i)
                        .unwrap()
                        .clone(),
                )
            }
        }
        Err(_) => None,
    }
}

lazy_static! {
    static ref IS_LOADING_AUTO_EXEC_GLOBAL_FLAG: AtomicBool =
        AtomicBool::new(false);
}

fn set_command(name: &str, value: &str) {
    if set_string_from_source(name, value, SetSource::External).is_err() {
        return;
    }

    if IS_LOADING_AUTO_EXEC_GLOBAL_FLAG.load(Ordering::SeqCst) == true {
        if let Some(d) = DVARS.write().unwrap().get_mut(name) {
            d.add_flags(DvarFlags::AUTOEXEC);
            d.reset = d.current.clone();
        }
    }
}

// Get a single string from a command's argv entries
fn get_combined_string(start_idx: usize) -> String {
    let argc = cmd::argc();
    let mut str = String::new();
    for i in start_idx..argc {
        let argv_i = cmd::argv(i);
        str.push_str(&argv_i);
        if argc != i - 1 {
            str.push(' ');
        }
    }
    str
}

lazy_static! {
    static ref DVAR_COUNT_LOCAL: AtomicIsize = AtomicIsize::new(0);
}

#[allow(clippy::many_single_char_names)]
fn list_single(dvar: &Dvar, name: &str) {
    if !dvar.flags.contains(DvarFlags::CON_ACCESS)
        && get_bool("con_access_restricted").unwrap_or(false) == true
    {
        return;
    }

    if !name.is_empty() && com::filter(name, &dvar.name, false) == false {
        return;
    }

    let s: char = if dvar
        .flags
        .contains(DvarFlags::UNKNOWN_00000400 | DvarFlags::SERVER_INFO)
    {
        'S'
    } else {
        ' '
    };
    let u: char = if dvar.flags.contains(DvarFlags::USER_INFO) {
        'U'
    } else {
        ' '
    };
    let r: char = if dvar.flags.contains(DvarFlags::READ_ONLY) {
        'R'
    } else {
        ' '
    };
    let i: char = if dvar.flags.contains(DvarFlags::WRITE_PROTECTED) {
        'I'
    } else {
        ' '
    };
    let a: char = if dvar.flags.contains(DvarFlags::ARCHIVE) {
        'A'
    } else {
        ' '
    };
    let l: char = if dvar.flags.contains(DvarFlags::LATCHED) {
        'L'
    } else {
        ' '
    };
    let c: char = if dvar.flags.contains(DvarFlags::CHEAT_PROTECTED) {
        'C'
    } else {
        ' '
    };
    let y: char = if dvar.flags.contains(DvarFlags::SYSTEM_INFO) {
        'Y'
    } else {
        ' '
    };
    let d: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00000100_D) {
        'D'
    } else {
        ' '
    };
    let x: char = if dvar.flags.contains(DvarFlags::AUTOEXEC) {
        'X'
    } else {
        ' '
    };
    let e: char = if dvar.flags.contains(DvarFlags::EXTERNAL) {
        'E'
    } else {
        ' '
    };
    let v: char = if dvar.flags.contains(DvarFlags::SAVED) {
        'Y'
    } else {
        ' '
    };

    com::println!(
        0.into(),
        "{}{}{}{}{}{}{}{}{}{}{}{} {} \"{}\"",
        s,
        u,
        r,
        i,
        a,
        l,
        c,
        y,
        d,
        x,
        e,
        v,
        dvar.name,
        dvar.current,
    );
    DVAR_COUNT_LOCAL.fetch_add(1, Ordering::SeqCst);
}

fn toggle_internal() -> Result<(), ()> {
    let argc = cmd::argc();

    let name = if argc < 1 {
        String::new()
    } else {
        cmd::argv(1)
    };

    if cmd::argc() < 2 {
        com::println!(
            0.into(),
            "USAGE: {} <variable> <optional value sequence>",
            name,
        );
        return Err(());
    }

    let argv_1 = cmd::argv(1);

    if !exists(&name) {
        com::println!(0.into(), "toggle failed: dvar \'{}\' not found.", name,);
        return Err(());
    }

    if cmd::argc() == 2 {
        return toggle_simple(&name);
    }

    for i in 2..argc {
        let mut argv_i = cmd::argv(i);
        if let DvarValue::Enumeration(_) =
            DvarValue::Enumeration(get_enumeration(&name).unwrap())
        {
            if let Some(s) = index_string_to_enum_string(&name, &argv_i) {
                if s.len() != 1 {
                    argv_i = s;
                }
            }
        }
        if get_enumeration(&name).unwrap() == argv_i {
            set_command(&cmd::argv(1), &cmd::argv(i + 1));
            return Ok(());
        }
    }

    let mut argv_2 = cmd::argv(2);
    if let DvarValue::Enumeration(_) = find(&name).unwrap().current {
        if let Some(s) = index_string_to_enum_string(&name, &argv_2) {
            if s.len() != 1 {
                argv_2 = s;
            }
        }
    }
    set_command(&argv_1, &argv_2);
    Ok(())
}

fn toggle_f() {
    #[allow(unused_must_use)]
    {
        toggle_internal();
    }
}

fn toggle_print_f() {
    if toggle_internal().is_err() {
        return;
    }

    let name = cmd::argv(1);
    com::println!(
        0.into(),
        "{} toggled to {}",
        name,
        find(&name).unwrap().current,
    );
}

fn set_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println!(0.into(), "USAGE: set <variable> <value>");
        return;
    }

    let name = cmd::argv(1);
    if !name_is_valid(&name) {
        com::println!(0.into(), "invalid variable name: {}", name);
        return;
    }

    let string = get_combined_string(2);
    set_command(&name, &string);
}

fn sets_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println!(0.into(), "USAGE: sets <variable> <value>\n");
    }

    set_f();
    let name = cmd::argv(1);

    let mut writer = DVARS.write().unwrap();

    if let Some(d) = writer.get_mut(&name) {
        d.add_flags(DvarFlags::SERVER_INFO);
    }
}

fn seta_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println!(0.into(), "USAGE: seta <variable> <value>\n");
    }

    set_f();
    let name = cmd::argv(1);

    let mut writer = DVARS.write().unwrap();

    if let Some(d) = writer.get_mut(&name) {
        d.add_flags(DvarFlags::ARCHIVE);
    }
}

fn set_admin_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println!(0.into(), "USAGE: setadminvar <variable> <value>\n");
    }

    let name = cmd::argv(1);

    if let Some(d) = DVARS.write().unwrap().get_mut(&name) {
        if d.flags.contains(DvarFlags::CON_ACCESS) {
            d.add_flags(DvarFlags::ARCHIVE);
        }
        set_f();
    } else {
        let name = cmd::argv(1);
        com::println!(
            0.into(),
            "setadmindvar failed: dvar \'{}\' not found.",
            name,
        );
    }
}

#[allow(clippy::todo)]
fn set_mod_dvar_f() {
    todo!()
}

fn set_from_dvar_f() {
    let argc = cmd::argc();
    if argc != 3 {
        com::println!(0.into(), "USAGE: setfromdvar <dest_dvar> <source_dvar>");
        return;
    }

    let dest_dvar_name = cmd::argv(1);
    let source_dvar_name = cmd::argv(2);

    let mut writer = DVARS.write().unwrap();
    if let Some(d) = writer.get_mut(&source_dvar_name) {
        set_command(&dest_dvar_name, &d.current.to_string());
    } else {
        com::println!(
            0.into(),
            "dvar \'{}\' doesn\'t exist\n",
            source_dvar_name,
        );
    }
}

#[allow(clippy::todo)]
fn set_from_localized_string_f() {
    todo!()
}

fn set_to_time_f() {
    let argc = cmd::argc();

    if argc < 2 {
        com::println!(0.into(), "USAGE: set <variable>");
        return;
    }

    let name = cmd::argv(1);
    if !name_is_valid(&name) {
        let name = cmd::argv(1);
        com::println!(0.into(), "invalid variable name: {}\n", name);
        return;
    }

    let time = sys::milliseconds();
    let name = cmd::argv(1);
    set_command(&name, &format!("{}", time));
}

fn reset_f() {
    let argc = cmd::argc();
    if argc != 2 {
        com::println!(0.into(), "USAGE: reset <variable>");
        return;
    }

    let name = cmd::argv(1);

    if exists(&name) {
        let mut writer = DVARS.write().unwrap();
        writer.get_mut(&name).unwrap().reset(SetSource::External);
    }
}

#[allow(clippy::semicolon_outside_block)]
fn list_f() {
    DVAR_COUNT_LOCAL.store(0, Ordering::SeqCst);
    let argv_1 = cmd::argv(1);
    DVARS
        .read()
        .unwrap()
        .values()
        .for_each(|d| list_single(d, &argv_1));
    com::println!(
        0.into(),
        "\n{} total dvars",
        DVAR_COUNT_LOCAL.load(Ordering::SeqCst),
    );
}

fn dump_f() {
    com::dvar_dump(0, &cmd::argv(1));
}

fn register_bool_f() {
    let argc = cmd::argc();
    if argc != 3 {
        let cmd = cmd::argv(0);
        com::println!(0.into(), "USAGE: {} <name> <default>", cmd);
    }

    let name = cmd::argv(1);
    let value = cmd::argv(2).parse::<bool>().unwrap();
    let dvar = find(&name);

    match dvar {
        None => {}
        Some(d) => match d.current {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_bool(
                        &name,
                        value,
                        DvarFlags::EXTERNAL,
                        Some("External Dvar"),
                    )
                    .unwrap();
                }
            }
            _ => {
                com::println!(
                    0.into(),
                    "dvar \'{}\' is not a boolean dvar",
                    name,
                );
            }
        },
    }
}

#[allow(clippy::match_same_arms)]
fn register_int_f() {
    let argc = cmd::argc();
    if argc != 5 {
        let cmd = cmd::argv(0);
        com::println!(0.into(), "USAGE: {} <name> <default> <min> <max>", cmd,);
        return;
    }

    let name = cmd::argv(1);
    let value = cmd::argv(2).parse::<i32>().ok();
    let min = cmd::argv(3).parse::<i32>().ok();
    let max = cmd::argv(4).parse::<i32>().ok();

    if min > max {
        com::println!(
            0.into(),
            "dvar {}: min {} should not be greater than max {}i\n",
            name,
            min.unwrap_or(0),
            max.unwrap_or(0),
        );
        return;
    }

    let dvar = find(&name);
    match dvar {
        None => {
            register_int(
                &name,
                value.unwrap_or(0),
                min,
                max,
                DvarFlags::EXTERNAL,
                Some("External Dvar"),
            )
            .unwrap();
        }
        Some(d) => match d.current {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_int(
                        &name,
                        value.unwrap_or(0),
                        min,
                        max,
                        DvarFlags::EXTERNAL,
                        Some("External Dvar"),
                    )
                    .unwrap();
                }
            }
            DvarValue::Int(_) => {}
            DvarValue::Enumeration(_) => {}
            _ => {
                com::println!(
                    0.into(),
                    "dvar \'{}\' is not an integer dvar",
                    d.name,
                );
            }
        },
    }
}

fn register_float_f() {
    let argc = cmd::argc();
    if argc != 5 {
        let cmd = cmd::argv(0);
        com::println!(0.into(), "USAGE: {} <name> <default> <min> <max>", cmd,);
        return;
    }

    let name = cmd::argv(1);
    let value = cmd::argv(2).parse::<f32>().ok();
    let min = cmd::argv(3).parse::<f32>().ok();
    let max = cmd::argv(4).parse::<f32>().ok();

    if min > max {
        com::println!(
            0.into(),
            "dvar {}: min {} should not be greater than max {}i\n",
            name,
            min.unwrap_or(0.0),
            max.unwrap_or(0.0),
        );
        return;
    }

    let dvar = find(&name);
    match dvar {
        None => {
            #[allow(unused_must_use)]
            {
                register_float(
                    &name,
                    value.unwrap_or(0.0),
                    min,
                    max,
                    DvarFlags::EXTERNAL,
                    Some("External Dvar"),
                );
            }
        }
        Some(d) => match d.current {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_float(
                        &name,
                        value.unwrap_or(0.0),
                        min,
                        max,
                        DvarFlags::EXTERNAL,
                        Some("External Dvar"),
                    )
                    .unwrap();
                }
            }
            DvarValue::Float(_) => {}
            _ => {
                com::println!(
                    0.into(),
                    "dvar {} is not an integer dvar",
                    d.name,
                );
            }
        },
    }
}

#[allow(clippy::many_single_char_names)]
fn register_color_f() {
    let argc = cmd::argc();
    // The command will be argv[0]. The name of the Dvar will be argv[1].
    // The R, G, B, and A components will be argv[2]-argv[6].
    // However, the A componenet is optional. Thus, argc may be 5 (if the A
    // component is not included), or 6 (if it is).

    // If argc isn't 5 or 6, the command is malformed. Print the correct usage
    // and return
    if argc != 5 && argc != 6 {
        let cmd = cmd::argv(0);
        com::println!(0.into(), "USAGE: {} <name> <r> <g> <b> [a]", cmd,);
        return;
    }

    let name = cmd::argv(1);
    // Default the R, G, and B components to 0.0 if they're malformed
    let r = cmd::argv(2).parse::<f32>().unwrap_or(0.0);
    let g = cmd::argv(3).parse::<f32>().unwrap_or(0.0);
    let b = cmd::argv(4).parse::<f32>().unwrap_or(0.0);
    // Default the A component to 1.0 if it's missing or malformed.
    let a = cmd::argv(5).parse::<f32>().unwrap_or(1.0);
    let dvar = find(&name);
    match dvar {
        None => {
            // If the Dvar doesn't exist, register it.
            register_color(
                &name,
                r,
                g,
                b,
                a,
                DvarFlags::EXTERNAL,
                Some("External Dvar"),
            )
            .unwrap();
        }
        Some(d) => {
            // Else if it does exist, the type is String, and the External flag is
            // set, register it
            if let DvarValue::String(_) = d.current {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_color(
                        &name,
                        r,
                        g,
                        b,
                        a,
                        DvarFlags::EXTERNAL,
                        Some("External Dvar"),
                    )
                    .unwrap();
                }
            }
        }
    }
    // Otherwise do nothing and continue
}

fn setu_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println!(0.into(), "USAGE: setu <variable> <value>");
        return;
    }

    set_f();
    let name = cmd::argv(1);
    add_flags(&name, DvarFlags::USER_INFO).unwrap();
}

#[allow(clippy::todo)]
fn set_all_client_dvars_f() {
    todo!()
}

fn restore_dvars() {
    if find("sv_restoreDvars").unwrap().current.as_bool().unwrap() == false {
        return;
    }

    DVARS.write().unwrap().values_mut().for_each(|d| {
        if d.loaded_from_save_game == true {
            d.loaded_from_save_game = false;
            d.set_variant(d.saved.clone(), SetSource::Internal);
        }
    });
}

fn display_dvar(dvar: &Dvar, i: &mut i32) {
    if dvar.flags.contains(DvarFlags::SAVED) {
        *i += 1;
        com::println!(0.into(), " {} \"{}\"", dvar.name, dvar);
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn list_saved_dvars() {
    let mut i = 0;
    DVARS
        .read()
        .unwrap()
        .values()
        .enumerate()
        .for_each(|(j, d)| {
            display_dvar(d, &mut (j as _));
            i = j;
        });

    com::println!(0.into(), "\n{} total SAVED dvars", i);
}

/// Adds commands for Dvar module
pub fn add_commands() {
    cmd::add_internal("toggle", toggle_f).unwrap();
    cmd::add_internal("togglep", toggle_print_f).unwrap();
    cmd::add_internal("set", set_f).unwrap();
    cmd::add_internal("sets", sets_f).unwrap();
    cmd::add_internal("seta", seta_f).unwrap();
    cmd::add_internal("setadminvar", set_admin_f).unwrap();
    cmd::add_internal("setmoddvar", set_mod_dvar_f).unwrap();
    cmd::add_internal("setfromdvar", set_from_dvar_f).unwrap();
    cmd::add_internal("setfromlocString", set_from_localized_string_f).unwrap();
    cmd::add_internal("reset", reset_f).unwrap();
    cmd::add_internal("dvarlist", list_f).unwrap();
    cmd::add_internal("dvardump", dump_f).unwrap();
    cmd::add_internal("dvar_bool", register_bool_f).unwrap();
    cmd::add_internal("dvar_int", register_int_f).unwrap();
    cmd::add_internal("dvar_float", register_float_f).unwrap();
    cmd::add_internal("dvar_color", register_color_f).unwrap();
    cmd::add_internal("setu", setu_f).unwrap();
    cmd::add_internal("setAllClientDvars", set_all_client_dvars_f).unwrap();
    cmd::add_internal("restoreDvars", restore_dvars).unwrap();
    cmd::add_internal("dvarlist_saved", list_saved_dvars).unwrap();
}
