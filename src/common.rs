#![allow(dead_code)]
use arrayvec::ArrayVec;

use crate::*;

use gfx::{ListBoxDef, UIAnimInfo, WindowDef};

pub type Vec2f32 = (f32, f32);
pub type Vec3f32 = (f32, f32, f32);
pub type Vec4f32 = (f32, f32, f32, f32);

#[derive(Copy, Clone, Default)]
pub struct RectDef {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    horz_align: i32,
    vert_align: i32,
}

impl RectDef {
    pub fn new() -> Self {
        RectDef {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            horz_align: 0,
            vert_align: 0,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct CardMemory {
    platform: [i32; 2],
}

#[derive(Copy, Clone, Default)]
pub struct Picmip {
    platform: [u8; 2],
}

#[derive(Clone)]
enum Operand {
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Clone)]
enum ExpressionRpn {
    Constant(Operand),
    Cmd(Vec<u8>),
    CmdIdx(i32),
}

#[derive(Clone, Default)]
struct ExpressionStatement {
    file_name: String,
    line: i32,
    numRpn: i32,
    rpn: Vec<ExpressionRpn>,
}

impl ExpressionStatement {
    fn new() -> Self {
        ExpressionStatement {
            file_name: "".to_string(),
            line: 0,
            numRpn: 0,
            rpn: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
struct TextExp {
    text_exp: ExpressionStatement,
}

impl TextExp {
    fn new() -> Self {
        TextExp {
            text_exp: ExpressionStatement::new(),
        }
    }
}

#[derive(Copy, Clone, Default)]
struct ScriptCondition {
    fire_on_true: bool,
    construct_id: i32,
    block_id: i32,
}

#[derive(Clone, Default)]
struct GenericEventScript {
    prerequisites: Vec<ScriptCondition>,
    condition: ExpressionStatement,
    fire_on_true: bool,
    action: String,
    block_id: i32,
    construct_id: i32,
}

impl GenericEventScript {
    fn new() -> Self {
        GenericEventScript {
            prerequisites: Vec::new(),
            condition: ExpressionStatement::new(),
            fire_on_true: false,
            action: "".to_string(),
            block_id: 0,
            construct_id: 0,
        }
    }
}

#[derive(Clone, Default)]
struct ItemKeyHandler {
    key: i32,
    key_script: GenericEventScript,
}

#[derive(Copy, Clone, Default)]
pub struct ColumnInfo {
    element_style: i32,
    max_chars: i32,
    rect: RectDef,
}

#[derive(Clone, Default)]
struct MenuCell {
    cell_type: i32,
    max_chars: i32,
    string_value: String,
}

#[derive(Clone, Default)]
pub struct MenuRow {
    cells: Vec<MenuCell>,
    event_name: String,
    on_focus_event_name: String,
    disable_arg: bool,
    status: i32,
    name: i32,
}

#[derive(Clone, Default)]
struct MultiDef {
    dvar_list: ArrayVec<String, 32>,
    dvar_str: ArrayVec<String, 32>,
    dvar_value: ArrayVec<f32, 32>,
    count: i32,
    action_on_enter_press_only: bool,
    str_def: i32,
}

#[derive(Copy, Clone, Default)]
struct EditFieldDef {
    cursor_pos: i32,
    min_val: f32,
    max_val: f32,
    def_val: f32,
    range: f32,
    max_chars: i32,
    max_chars_goto_next: i32,
    max_paint_chars: i32,
    paint_offset: i32,
}

#[derive(Clone, Default)]
struct EnumDvarDef {
    enum_dvar_name: String,
}

#[derive(Clone)]
enum FocusDefData {
    ListBox(Box<ListBoxDef>),
    Multi(Box<MultiDef>),
    EditField(EditFieldDef),
    EnumDvar(EnumDvarDef),
    Data(Vec<u8>),
}

#[derive(Clone)]
struct FocusItemDef {
    mouse_enter_text: String,
    mouse_exit_text: String,
    mouse_enter: String,
    mouse_exit: String,
    on_key: ItemKeyHandler,
    focus_type_data: FocusDefData,
}

#[derive(Copy, Clone, Default)]
struct GameMsgDef {
    game_msg_window_index: i32,
    game_msg_window_mode: i32,
}

#[derive(Clone)]
enum TextDefData {
    FocusItemDef(Box<FocusItemDef>),
    GameMsgDef(GameMsgDef),
    Data(Vec<u8>),
}

#[derive(Clone)]
struct TextDef {
    text_rect: RectDef,
    alignment: i32,
    font_enum: i32,
    item_flags: i32,
    text_align_mode: i32,
    text_align_x: f32,
    text_align_y: f32,
    text_scale: f32,
    text_style: i32,
    text: String,
    text_exp_data: TextExp,
    text_type_data: TextDefData,
}

#[derive(Clone, Default)]
struct ImageDef {
    material_exp: ExpressionStatement,
}

#[derive(Clone, Default)]
struct OwnerDrawDef {
    data_exp: ExpressionStatement,
}

#[derive(Clone)]
enum ItemDefData {
    TextDef(TextDef),
    ImageDef(ImageDef),
    FocusItemDef(FocusItemDef),
    OwnerDrawDef(OwnerDrawDef),
    Data(Vec<u8>),
}

#[derive(Clone, Default)]
pub struct GenericEventHandler {
    name: String,
    event_script: GenericEventScript,
}

impl GenericEventHandler {
    pub fn new() -> Self {
        GenericEventHandler {
            name: "".to_string(),
            event_script: GenericEventScript::new(),
        }
    }
}

struct MenuDef {
    window: WindowDef,
    font: String,
    fullscreen: bool,
    ui_3d_window_id: i32,
    item_count: i32,
    font_index: i32,
    cursor_item: i32,
    fade_cycle: i32,
    priority: i32,
    fade_clamp: f32,
    fade_amount: f32,
    fade_in_amount: f32,
    blur_radius: f32,
    open_slide_speed: i32,
    close_slide_speed: i32,
    open_slide_direction: i32,
    close_slide_direction: i32,
    initial_rect_info: RectDef,
    open_fading_time: i32,
    close_fading_time: i32,
    fade_time_counter: i32,
    slide_time_counter: i32,
    on_event: GenericEventHandler,
    on_key: ItemKeyHandler,
    visible_exp: ExpressionStatement,
    show_bits: u64,
    hide_bits: u64,
    allowed_binding: String,
    sound_name: String,
    image_track: i32,
    control: i32,
    focus_color: Vec4f32,
    disable_color: Vec4f32,
    rect_x_exp: ExpressionStatement,
    rect_y_exp: ExpressionStatement,
    items: Vec<ItemDef>,
}

#[derive(Clone, Default)]
struct RectData {
    rect_x_exp: ExpressionStatement,
    rect_y_exp: ExpressionStatement,
    rect_w_exp: ExpressionStatement,
    rect_h_exp: ExpressionStatement,
}

impl RectData {
    fn new() -> Self {
        RectData {
            rect_x_exp: ExpressionStatement::new(),
            rect_y_exp: ExpressionStatement::new(),
            rect_w_exp: ExpressionStatement::new(),
            rect_h_exp: ExpressionStatement::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ItemDef {
    def_type: i32,
    data_type: i32,
    image_track: i32,
    dvar: String,
    dvar_test: String,
    enable_dvar: String,
    dvar_flags: i32,
    type_data: Option<ItemDefData>,
    rect_exp_data: RectData,
    visible_exp: ExpressionStatement,
    show_bits: u64,
    hide_bits: u64,
    forecolor_a_exp: ExpressionStatement,
    ui_3d_window_id: i32,
    on_event: GenericEventHandler,
    animInfo: UIAnimInfo,
}

impl ItemDef {
    pub fn new() -> Self {
        ItemDef {
            def_type: 0,
            data_type: 0,
            image_track: 0,
            dvar: "".to_string(),
            dvar_test: "".to_string(),
            enable_dvar: "".to_string(),
            dvar_flags: 0,
            type_data: None,
            rect_exp_data: RectData::new(),
            visible_exp: ExpressionStatement::new(),
            show_bits: 0,
            hide_bits: 0,
            forecolor_a_exp: ExpressionStatement::new(),
            ui_3d_window_id: 0,
            on_event: GenericEventHandler::new(),
            animInfo: UIAnimInfo::new(),
        }
    }
}
