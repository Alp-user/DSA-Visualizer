use core::ffi::{c_char, c_float, c_int, c_uint, c_void};
use core::ffi::c_ulong;

#[allow(dead_code)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CircleSquare {
    pub circle_or_square: u8,          // unsigned char
    pub x: c_float,
    pub y: c_float,
    pub width: c_float,
    pub height: c_float,
    pub thickness: c_float,
    pub color: [c_float; 3],
    pub buffer_index: c_uint,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Text {
    pub text: *mut c_char,
    pub length: c_uint,
    pub start_vbo: c_uint,
    pub end_vbo: c_uint,
    pub x: c_float,
    pub y: c_float,
}


#[allow(dead_code)]
unsafe extern "C" {

    /* ---- sprite renderer ---- */
    pub fn new_sprite_renderer();
    pub fn destroy_sprite_renderer();

    pub fn new_line(
        x: c_float,
        y: c_float,
        width: c_float,
        height: c_float,
        angle: c_float,
        r: c_float,
        g: c_float,
        b: c_float,
    ) -> c_uint;

    pub fn new_circle(
        x: c_float,
        y: c_float,
        radius: c_float,
        thickness: c_float,
        r: c_float,
        g: c_float,
        b: c_float,
    ) -> c_uint;

    pub fn new_triangle(
        x: c_float,
        y: c_float,
        width: c_float,
        height: c_float,
        angle: c_float,
        r: c_float,
        g: c_float,
        b: c_float,
    ) -> c_uint;

    pub fn new_rectangle(
        x: c_float,
        y: c_float,
        width: c_float,
        height: c_float,
        thickness: c_float,
        r: c_float,
        g: c_float,
        b: c_float,
    ) -> c_uint;

    pub fn new_square(
        x: c_float,
        y: c_float,
        width: c_float,
        thickness: c_float,
        r: c_float,
        g: c_float,
        b: c_float,
    ) -> c_uint;

    pub fn move_sprite(sprite_id: c_uint, new_x: c_float, new_y: c_float);
    pub fn color_sprite(sprite_id: c_uint, r: c_float, g: c_float, b: c_float);
    pub fn scale_sprite(
        sprite_id: c_uint,
        width: c_float,
        height: c_float,
        thickness: c_float,
    );
    pub fn rotate_sprite(sprite_id: c_uint, thickness: c_float);
    pub fn remove_sprite(sprite_id: c_uint);

    pub fn sprite_cleanup();
    pub fn destroy_renderer();

    pub fn get_sprite(sprite_id: c_uint) -> *mut CircleSquare;
    pub fn draw_sprites();

    /* ---- font subsystem ---- */
    pub fn change_active_font(number: c_ulong);
    pub fn initialize_font(file_name: *const c_char);
    pub fn initialize_font_renderer();


    pub fn create_text(
        text: *const c_char,
        glo_x: c_float,
        glo_y: c_float,
    ) -> c_uint;

    pub fn create_centered_text(
        copy_text: *const c_char,
        cx: c_float,
        cy: c_float,
        width: c_float,
        height: c_float,
    ) -> c_uint;

    pub fn create_text_height(
        copy_text: *const c_char,
        cx: c_float,
        cy: c_float,
        pixel_line_height: c_float,
    ) -> c_uint;

    pub fn move_text(text_id: c_uint, glo_x: c_float, glo_y: c_float);
    pub fn modify_text(text_id: c_uint, new_text: *const c_char);
    pub fn delete_text(text_id: c_uint);

    pub fn load_all();
    pub fn load_new();
    pub fn render_text();
    pub fn free_font_resources();
    pub fn text_cleanup();
    
    pub fn scale_font(text_id: c_uint, pixel_line_height: c_uint);
    pub fn color_font(r: c_float, g: c_float, b: c_float);

    pub fn load_functions() -> c_int;
    pub fn get_text(text_id: c_uint) -> *mut Text; 
    pub fn override_sprite(sprite_id: c_uint, x: c_float, y: c_float, width: c_float, height: c_float, thick: c_float, r: c_float, g: c_float, b: c_float);
    //Initialize rendering
    pub fn initialize_render();
}

