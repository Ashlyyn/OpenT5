#![allow(dead_code)]

use crate::*;
use arrayvec::ArrayVec;
use common::*;
use num::complex::Complex;

#[derive(Clone, Default)]
struct GfxDrawSurf {
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
    draw_surf: GfxDrawSurf,
    surface_type_bits: u32,
    layered_surface_types: u32,
}

#[derive(Copy, Clone, Default)]
struct MaterialStreamRouting {
    source: u8,
    dest: u8,
}

#[derive(Copy, Clone, Default)]
struct VertexDeclaration {}

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
struct VertexShaderProgram {}

#[derive(Copy, Clone, Default)]
struct GfxVertexShaderLoadDef {
    program: VertexShaderProgram,
    program_size: u16,
}

#[derive(Copy, Clone, Default)]
struct VertexShader {}

#[derive(Clone, Default)]
struct MaterialVertexShaderProgram {
    vs: VertexShader,
    load_def: GfxVertexShaderLoadDef,
}

#[derive(Clone, Default)]
struct MaterialVertexShader {
    name: String,
    prog: MaterialVertexShaderProgram,
}

struct PixelShaderProgram {}

struct GfxPixelShaderLoadDef {
    program: PixelShaderProgram,
    program_size: u16,
}

struct PixelShader {}

struct MaterialVertexPixelProgram {
    ps: VertexShader,
    load_def: GfxPixelShaderLoadDef,
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
struct BaseTexture {}

#[derive(Clone)]
struct Texture {}

#[derive(Clone)]
struct VolumeTexture {}

#[derive(Clone)]
struct CubeTexture {}

#[derive(Clone)]
struct GfxImageLoadDef {
    level_count: u8,
    flags: u8,
    format: i32,
    data: Vec<u8>,
}

#[derive(Clone)]
enum GfxTexture {
    BaseMap(BaseTexture),
    Map(Texture),
    VolumeMap(VolumeTexture),
    CubeMap(CubeTexture),
    LoadDef(GfxImageLoadDef),
}

#[derive(Clone)]
struct GfxImage {
    texture: GfxTexture,
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
    image: GfxImage,
}

#[derive(Clone)]
enum MaterialTextureDefInfo {
    Image(GfxImage),
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
struct GfxStateBits {
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
    state_bits_table: Vec<GfxStateBits>,
}

#[derive(Clone, Default)]
struct AnimParamsDef {
    name: String,
    rect_client: RectDef,
    border_size: f32,
    foreColor: Vec4f32,
    backColor: Vec4f32,
    borderColor: Vec4f32,
    outlineColor: Vec4f32,
    text_scale: f32,
    rotation: f32,
    on_event: GenericEventHandler,
}

impl AnimParamsDef {
    pub fn new() -> Self {
        AnimParamsDef {
            name: "".to_string(),
            rect_client: RectDef::new(),
            border_size: 0.0,
            foreColor: (0.0, 0.0, 0.0, 0.0),
            backColor: (0.0, 0.0, 0.0, 0.0),
            borderColor: (0.0, 0.0, 0.0, 0.0),
            outlineColor: (0.0, 0.0, 0.0, 0.0),
            text_scale: 0.0,
            rotation: 0.0,
            on_event: GenericEventHandler::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct UIAnimInfo {
    animStateCount: i32,
    animStates: Vec<AnimParamsDef>,
    currentAnimState: AnimParamsDef,
    nextAnimState: AnimParamsDef,
    animating: i32,
    animStartTime: i32,
    animDuration: i32,
}

impl UIAnimInfo {
    pub fn new() -> Self {
        UIAnimInfo {
            animStateCount: 0,
            animStates: Vec::new(),
            currentAnimState: AnimParamsDef::new(),
            nextAnimState: AnimParamsDef::new(),
            animating: 0,
            animStartTime: 0,
            animDuration: 0,
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
