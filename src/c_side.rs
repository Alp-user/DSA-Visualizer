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
  pub text: *const c_char, // u32string in C++ - represented as const char* for simplicity
  pub scale_constant: c_float,
  pub rotation: c_float,
  
  pub coordinates: [c_int; 2], // glm::ivec2
  pub center_coordinates: [c_int; 2], // glm::ivec2
  pub buffer_indices: [c_int; 2], // glm::ivec2
  pub box_dimensions: [c_int; 2], // glm::ivec2
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

  /* ---- font renderer ---- */
  pub fn initialize_font_renderer(font_path: *const c_char);
  pub fn render_text();
  pub fn create_text(text: *const c_char, x: c_int, y: c_int, pixel_height: c_int) -> c_uint;
  pub fn create_text_centered(
    text: *const c_char, 
    center_x: c_int, 
    center_y: c_int, 
    max_width: c_int, 
    max_height: c_int, 
    rotation: c_float
  ) -> c_uint;
  pub fn load_text_vbo(text_id: c_uint);
  pub fn load_all_text_vbo();

  pub fn move_text(text_id: c_uint, center_x: c_int, center_y: c_int); // glm::ivec2 new_center
  pub fn rotate_text(text_id: c_uint, angle: c_float);
  pub fn remove_text(text_id: c_uint);
  pub fn scale_text(text_id: c_uint, pixel_height: c_int);
  pub fn get_text(text_id: c_uint) -> *mut Text;
  pub fn cleanup_text(); 
  pub fn override_sprite(sprite_id: c_uint, x: c_float, y: c_float, width: c_float, height: c_float, thick: c_float, r: c_float, g: c_float, b: c_float);
  //Initialize rendering
  pub fn initialize_render();
  pub fn sprite_uniform_matrix(width: c_float, height: c_float, cam_horizontal: c_float, cam_vertical: c_float);
  pub fn set_uniform_matrix(width: c_float, height: c_float, cam_horizontal: c_float, cam_vertical: c_float);
}
