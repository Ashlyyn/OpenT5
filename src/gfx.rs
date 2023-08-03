#![allow(dead_code)]

use crate::{
    platform::{os::target::MonitorHandle, WindowHandle},
    render::{MIN_HORIZONTAL_RESOLUTION, MIN_VERTICAL_RESOLUTION},
    *,
};
use arrayvec::ArrayVec;
use common::*;
use num::complex::Complex;

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default)]
struct DrawSurf {
    object_id: u16,             // 16 bits
    fade: u8,                   // 4 bits
    custom_index: u8,           // 5 bits
    reflection_probe_index: u8, // 3 bits
    hdr_bits: bool,             // 1 bit
    glight_render: bool,        // 1 bit
    dlight_render: bool,        // 1 bit
    material_sorted_index: u16, // 12 bits
    primary_light_index: u8,    // 8 bits
    surf_type: u8,              // 4 bits
    prepass: u8,                // 2 bits
    no_dynamic_shadow: bool,    // 1 bit
    primary_sort_key: u8,       // 6 bits
}

#[derive(Clone, Default)]
struct MaterialInfo {
    name: String,
    game_flags: u32,
    sort_key: u8,
    texture_atlas_row_count: u8,
    texture_atlas_column_count: u8,
    draw_surf: DrawSurf,
    surface_type_bits: u32,
    layered_surface_types: u32,
}

#[derive(Copy, Clone, Default)]
struct MaterialStreamRouting {
    source: u8,
    dest: u8,
}

#[derive(Copy, Clone, Default)]
struct VertexDeclaration;

#[derive(Clone, Default)]
struct MaterialVertexStreamRouting {
    data: ArrayVec<MaterialStreamRouting, 16>,
    decl: ArrayVec<VertexDeclaration, 18>,
}

#[derive(Clone, Default)]
struct MaterialVertexDeclaration {
    stream_count: u8,
    has_optional_source: bool,
    is_loaded: bool,
    routing: MaterialVertexStreamRouting,
}

#[derive(Copy, Clone, Default)]
struct VertexShaderProgram;

#[derive(Copy, Clone, Default)]
struct VertexShaderLoadDef {
    program: VertexShaderProgram,
    program_size: u16,
}

#[derive(Copy, Clone, Default)]
struct VertexShader;

#[derive(Clone, Default)]
struct MaterialVertexShaderProgram {
    vs: VertexShader,
    load_def: VertexShaderLoadDef,
}

#[derive(Clone, Default)]
struct MaterialVertexShader {
    name: String,
    prog: MaterialVertexShaderProgram,
}

struct PixelShaderProgram;

struct PixelShaderLoadDef {
    program: PixelShaderProgram,
    program_size: u16,
}

struct PixelShader;

struct MaterialVertexPixelProgram {
    ps: VertexShader,
    load_def: PixelShaderLoadDef,
}

#[derive(Clone, Default)]
struct MaterialPixelShader {
    name: String,
    prog: MaterialVertexShaderProgram,
}

#[derive(Copy, Clone, Default)]
struct MaterialArgumentCodeConst {
    index: u16,
    first_row: u8,
    row_count: u8,
}

#[derive(Clone)]
enum MaterialArgumentDef {
    LiteralConst(Vec4f32),
    CodeConst(MaterialArgumentCodeConst),
    CodeSample(u32),
    NameHash(u32),
}

#[derive(Clone)]
struct MaterialShaderArgument {
    dest: u16,
    material_argument_def: MaterialArgumentDef,
}

#[derive(Clone, Default)]
struct MaterialPass {
    vertex_decl: MaterialVertexDeclaration,
    vertex_shader: MaterialVertexShader,
    pixel_shader: MaterialPixelShader,
    per_prim_arg_count: u8,
    per_obj_arg_count: u8,
    stable_arg_count: u8,
    custom_sampler_flags: u8,
    args: Vec<MaterialShaderArgument>,
}

#[derive(Clone, Default)]
struct MaterialTechnique {
    name: String,
    flags: u16,
    pass_array: Vec<MaterialPass>,
}

#[derive(Clone, Default)]
struct MaterialTechniqueSet {
    name: String,
    world_vert_format: u8,
    techset_flags: u16,
    techniques: ArrayVec<MaterialTechnique, 130>,
}

#[derive(Clone)]
struct BaseTexture;

#[derive(Clone)]
struct TextureDef;

#[derive(Clone)]
struct VolumeTexture;

#[derive(Clone)]
struct CubeTexture;

#[derive(Clone)]
struct ImageLoadDef {
    level_count: u8,
    flags: u8,
    format: i32,
    data: Vec<u8>,
}

#[derive(Clone)]
enum Texture {
    BaseMap(BaseTexture),
    Map(TextureDef),
    VolumeMap(VolumeTexture),
    CubeMap(CubeTexture),
    LoadDef(ImageLoadDef),
}

#[derive(Clone)]
struct Image {
    texture: Texture,
    map_type: u8,
    semantic: u8,
    category: u8,
    delay_load_pixels: bool,
    picmip: Option<Picmip>,
    track: u8,
    card_memory: CardMemory,
    width: u16,
    height: u16,
    depth: u16,
    level_count: u8,
    streaming: u8,
    base_size: u32,
    pixels: Vec<u8>,
    loaded_size: u32,
    skipped_mip_levels: u8,
    name: String,
}

#[derive(Copy, Clone, Default)]
struct WaterWritable {
    float_time: f32,
}

#[derive(Clone)]
struct Water {
    writable: WaterWritable,
    h0: Complex<f32>,
    w_term: f32,
    m: i32,
    n: i32,
    l_x: f32,
    l_y: f32,
    gravity: f32,
    windvel: f32,
    winddir: Vec2f32,
    amplitude: f32,
    code_constant: Vec4f32,
    image: Image,
}

#[derive(Clone)]
enum MaterialTextureDefInfo {
    Image(Image),
    Water(Water),
}

#[derive(Clone)]
struct MaterialTextureDef {
    name_start: u8,
    name_end: u8,
    sampler_state: u8,
    semantic: u8,
    is_mature_content: bool,
    info: MaterialTextureDefInfo,
}

#[derive(Copy, Clone, Default)]
struct MaterialConstantDef {
    name: [char; 12],
    literal: Vec4f32,
}

#[derive(Copy, Clone, Default)]
struct StateBits {
    load_bits: [u32; 2],
}

#[derive(Clone, Default)]
pub struct Material {
    info: MaterialInfo,
    state_bits_entry: ArrayVec<u8, 130>,
    texture_count: u8,
    constant_count: u8,
    state_bits_count: u8,
    state_flags: u8,
    camera_region: u8,
    max_streamed_mips: u8,
    technique_set: MaterialTechniqueSet,
    texture_table: Vec<MaterialTextureDef>,
    constant_table: Vec<MaterialConstantDef>,
    state_bits_table: Vec<StateBits>,
}

#[derive(Clone, Default)]
struct AnimParamsDef {
    name: String,
    rect_client: RectDef,
    border_size: f32,
    fore_color: Vec4f32,
    back_color: Vec4f32,
    border_color: Vec4f32,
    outline_color: Vec4f32,
    text_scale: f32,
    rotation: f32,
    on_event: GenericEventHandler,
}

impl AnimParamsDef {
    pub const fn new() -> Self {
        Self {
            name: String::new(),
            rect_client: RectDef::new(),
            border_size: 0.0,
            fore_color: (0.0, 0.0, 0.0, 0.0),
            back_color: (0.0, 0.0, 0.0, 0.0),
            border_color: (0.0, 0.0, 0.0, 0.0),
            outline_color: (0.0, 0.0, 0.0, 0.0),
            text_scale: 0.0,
            rotation: 0.0,
            on_event: GenericEventHandler::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct UIAnimInfo {
    anim_state_count: i32,
    anim_states: Vec<AnimParamsDef>,
    current_anim_state: AnimParamsDef,
    next_anim_state: AnimParamsDef,
    animating: i32,
    anim_start_time: i32,
    anim_duration: i32,
}

impl UIAnimInfo {
    pub const fn new() -> Self {
        Self {
            anim_state_count: 0,
            anim_states: Vec::new(),
            current_anim_state: AnimParamsDef::new(),
            next_anim_state: AnimParamsDef::new(),
            animating: 0,
            anim_start_time: 0,
            anim_duration: 0,
        }
    }
}

pub struct WindowDef {
    name: String,
    rect: RectDef,
    rect_client: RectDef,
    group: String,
    style: u8,
    border: u8,
    modal: u8,
    frame_sides: u8,
    frame_tex_size: f32,
    frame_size: f32,
    owner_draw: i32,
    owner_draw_flags: i32,
    border_size: f32,
    static_flags: i32,
    dynamic_flags: i32,
    next_time: i32,
    fore_color: Vec4f32,
    back_color: Vec4f32,
    border_color: Vec4f32,
    outline_color: Vec4f32,
    rotation: f32,
    background: Material,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default)]
pub struct ListBoxDef {
    mouse_pos: i32,
    cursor_pos: i32,
    start_pos: i32,
    end_pos: i32,
    draw_padding: i32,
    element_width: f32,
    element_height: f32,
    num_columns: i32,
    special: f32,
    column_info: ArrayVec<ColumnInfo, 16>,
    not_selectable: bool,
    no_scroll_bars: bool,
    use_paging: bool,
    select_border: Vec4f32,
    disable_color: Vec4f32,
    focus_color: Vec4f32,
    element_highlight_color: Vec4f32,
    element_background_color: Vec4f32,
    select_icon: Material,
    background_item_listbox: Material,
    highlight_texture: Material,
    no_blinking_highlight: bool,
    rows: Vec<MenuRow>,
    max_rows: i32,
    row_count: i32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct WindowParms {
    pub window_handle: Option<WindowHandle>,
    pub monitor_handle: Option<MonitorHandle>,
    pub hz: f32,
    pub fullscreen: bool,
    pub x: u16,
    pub y: u16,
    pub scene_width: u32,
    pub scene_height: u32,
    pub display_width: u32,
    pub display_height: u32,
    pub aa_samples: u32,
}

impl WindowParms {
    pub fn new() -> Self {
        Self {
            hz: 60.0,
            fullscreen: false,
            x: 0,
            y: 0,
            scene_width: MIN_HORIZONTAL_RESOLUTION,
            scene_height: MIN_VERTICAL_RESOLUTION,
            display_width: MIN_HORIZONTAL_RESOLUTION,
            display_height: MIN_VERTICAL_RESOLUTION,
            aa_samples: 0,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WindowTarget {
    pub width: u32,
    pub height: u32,
    pub handle: Option<WindowHandle>,
}

impl WindowTarget {
    pub const fn new() -> Self {
        Self {
            width: MIN_HORIZONTAL_RESOLUTION,
            height: MIN_VERTICAL_RESOLUTION,
            handle: None,
        }
    }
}

impl Default for WindowTarget {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Globals {
    pub started_render_thread: bool,
    pub is_multiplayer: bool,
    pub end_frame_fence: i32,
    pub is_rendering_remote_update: bool,
    pub screen_update_notify: bool,
    pub remote_screen_update_nesting: i32,
    pub remote_screen_update_in_game: i32,
    pub remote_screen_last_scene_resolve_target: u8,
    pub back_end_frame_count: i32,
    pub frame_buffer: u8,
    pub display_buffer: u8,
    pub ui_3d_use_frame_buffer: u8,
    pub ui_3d_render_target: u8,
}
