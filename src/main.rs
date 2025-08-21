//use tree_sitter::{InputEdit, Language, Parser, Point};
mod c_side;
mod tree;
mod json_deserialize;
mod graph_draw;
mod dsas;
mod hashgrid;
use glfw::{fail_on_errors, Action, Context, Key, WindowEvent,WindowHint, OpenGlProfileHint, WindowMode};
use std::{f32::consts::PI, ffi::CString};
use tree::{*};
use json_deserialize::{deserialize_json, Root};
use graph_draw::{*};

// Animation state variables
static mut MOVE_ANIMATION: bool = false;
static mut SCALE_ANIMATION: bool = false;
static mut LINE_ANIMATION: bool = false;

fn main() {
  let mut glfw = glfw::init(fail_on_errors!()).unwrap();

  glfw.window_hint(WindowHint::ContextVersion(4, 5));
  glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
  glfw.window_hint(WindowHint::Samples(Some(4)));
  glfw.window_hint(glfw::WindowHint::AlphaBits(Some(8)));
  let (mut window, events) = glfw.create_window(1920, 1080, "RustGL", WindowMode::Windowed).unwrap();
  window.make_current();
  window.set_key_polling(true);
  gl::load_with(|s| window.get_proc_address(s));

  let json_data: Root = deserialize_json("/home/alp/code_files/c++/works/json_converter/ds.txt").expect("Error");
  let graph_drawer = GraphDrawBuilder::new()
    .viewport((500,500), (900,900))
    .root(&json_data)
    .build();

  println!("{:#?}", graph_drawer);

  unsafe {
    c_side::initialize_render();
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    gl::Enable(gl::MULTISAMPLE);
    gl::Disable(gl::DEPTH_TEST);  // If applicable
  }
  unsafe{
    c_side::new_sprite_renderer();
    let path = "/usr/share/fonts/TTF/CaskaydiaCoveNerdFontMono-Regular.ttf";
    c_side::initialize_font_renderer(CString::new(path).expect("Error cstr").as_ptr());
  }
  
  while !window.should_close() {
    window.swap_buffers();
    glfw.poll_events();

    for (_, event) in glfw::flush_messages(&events) {
      match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
          window.set_should_close(true)
        },
        glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
        },
        glfw::WindowEvent::Key(Key::M, _, Action::Press, _) => {
        },
        glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
        },
        glfw::WindowEvent::Key(Key::L, _, Action::Press, _) => {
        },
        _ => {},
      }
    }

    // Update animations if enabled

    unsafe {
      gl::ClearColor(0.3, 0.8, 0.2, 0.5);
      gl::Clear(gl::COLOR_BUFFER_BIT);
      c_side::render_text();
      c_side::draw_sprites();
    }
  }
}

fn handle_window_event(window:&mut glfw::Window, event: glfw::WindowEvent){
  match event {
    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
      window.set_should_close(true)
    },
    glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
      println!("Good Job
");
    }
    _ => {},
  }
}
