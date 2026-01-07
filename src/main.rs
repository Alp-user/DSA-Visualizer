mod c_side;
mod dsas;
mod graph_draw;
mod hashgrid;
mod json_deserialize;
mod tree;

use crate::hashgrid::HashGrid;
use glfw::{
    Action, Context, GlfwReceiver, Key, OpenGlProfileHint, Window, WindowEvent, WindowHint,
    WindowMode, fail_on_errors,
};
use graph_draw::*;
use json_deserialize::{Root, deserialize_json};
use std::{f32::consts::PI, ffi::CString};
use tree::*;

pub static mut CAMERA_SHIFT: (f32, f32) = (0.0, 0.0);
pub static mut DIMENSIONS: (f32, f32) = (1920.0, 1080.0);
pub static mut STABLE_HAPPENED: bool = false;

static mut LEFT: bool = false;
static mut RIGHT: bool = false;
static mut UP: bool = false;
static mut DOWN: bool = false;
static mut ENTER: bool = false;
static mut ERASE: bool = false;
static mut NUM1: bool = false;
static mut NUM2: bool = false;
static mut NUM3: bool = false;
static mut NUM4: bool = false;
static mut NUM5: bool = false;
static mut NUM6: bool = false;
static mut NUM7: bool = false;
static mut NUM8: bool = false;
static mut NUM9: bool = false;

const SHIFT_AMOUNT: f32 = 8.0;

pub fn resize_camera(width: f32, height: f32, cam_horizontal: f32, cam_vertical: f32) {
    unsafe {
        CAMERA_SHIFT = (
            CAMERA_SHIFT.0 + cam_horizontal,
            CAMERA_SHIFT.1 + cam_vertical,
        );
        DIMENSIONS = (width, height);
        gl::Viewport(0, 0, DIMENSIONS.0 as i32, DIMENSIONS.1 as i32);
        c_side::set_uniform_matrix(DIMENSIONS.0, DIMENSIONS.1, CAMERA_SHIFT.0, CAMERA_SHIFT.1);
        c_side::sprite_uniform_matrix(DIMENSIONS.0, DIMENSIONS.1, CAMERA_SHIFT.0, CAMERA_SHIFT.1);
    }
}

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(4, 5));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::Samples(Some(4)));
    glfw.window_hint(glfw::WindowHint::AlphaBits(Some(8)));
    let (mut window, events) = glfw
        .create_window(1920, 1080, "RustGL", WindowMode::Windowed)
        .unwrap();
    window.make_current();
    window.set_key_polling(true);
    window.set_size(1920, 1080);
    window.set_framebuffer_size_callback(callback_resize);
    gl::load_with(|s| window.get_proc_address(s));

    let json_data: Root =
        deserialize_json("/home/alp/Desktop/code_files/c++/works/json_converter/ds.txt")
            .expect("Error");
    let graph_drawer = GraphDrawBuilder::new()
        .viewport((0, 0), (1920, 1080))
        .root(&json_data)
        .build();

    unsafe {
        c_side::initialize_render();
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::MULTISAMPLE);
        gl::Disable(gl::DEPTH_TEST); // If applicable
    }

    unsafe {
        c_side::new_sprite_renderer();
        let path = "/usr/share/fonts/TTF/CaskaydiaCoveNerdFontMono-Regular.ttf";
        c_side::initialize_font_renderer(CString::new(path).expect("Error cstr").as_ptr());
        c_side::load_all_text_vbo();
    }

    let mut graph_draw = GraphDrawBuilder::new()
        .viewport((0, 0), (1920, 1080))
        .root(&json_data)
        .listener_id(0)
        .build()
        .expect("Error building graph drawer");

    for i in 0..(json_data.total_listeners as usize) {
        println!("Adding graph for listener id {}", i);
        graph_draw.add_new_graph(i);
    }

    while !window.should_close() {
        window.swap_buffers();
        glfw.poll_events();
        if !unsafe { STABLE_HAPPENED } {
            unsafe {
                STABLE_HAPPENED = graph_draw.simulation_step();
            }
        }
        handle_window_event(&events, &mut window);
        // Update animations if enabled
        if unsafe { LEFT } {
            left_pressed();
        }
        if unsafe { RIGHT } {
            right_pressed();
        }
        if unsafe { UP } {
            up_pressed();
        }
        if unsafe { DOWN } {
            down_pressed();
        }

        if unsafe { ENTER } {
            graph_draw.forward_diff();
            unsafe {
                ENTER = false;
            }
        }
        if unsafe { ERASE } {
            graph_draw.backward_diff();
            unsafe {
                ERASE = false;
            }
        }
        if unsafe { NUM1 } {
            graph_draw.change_listener_id(1);
            unsafe {
                NUM1 = false;
            }
        }
        if unsafe { NUM2 } {
            graph_draw.change_listener_id(2);
            unsafe {
                NUM2 = false;
            }
        }
        if unsafe { NUM3 } {
            graph_draw.change_listener_id(3);
            unsafe {
                NUM3 = false;
            }
        }
        if unsafe { NUM4 } {
            graph_draw.change_listener_id(4);
            unsafe {
                NUM4 = false;
            }
        }
        if unsafe { NUM5 } {
            graph_draw.change_listener_id(5);
            unsafe {
                NUM5 = false;
            }
        }
        if unsafe { NUM6 } {
            graph_draw.change_listener_id(6);
            unsafe {
                NUM6 = false;
            }
        }
        if unsafe { NUM7 } {
            graph_draw.change_listener_id(7);
            unsafe {
                NUM7 = false;
            }
        }
        if unsafe { NUM8 } {
            graph_draw.change_listener_id(8);
            unsafe {
                NUM8 = false;
            }
        }
        if unsafe { NUM9 } {
            graph_draw.change_listener_id(9);
            unsafe {
                NUM9 = false;
            }
        }

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.5);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            c_side::render_text();
            c_side::draw_sprites();
        }
    }
}

fn callback_resize(_: &mut glfw::Window, width: i32, height: i32) {
    unsafe {
        DIMENSIONS = (width as f32, height as f32);
        gl::Viewport(0, 0, width, height);
        c_side::set_uniform_matrix(DIMENSIONS.0, DIMENSIONS.1, CAMERA_SHIFT.0, CAMERA_SHIFT.1);
        c_side::sprite_uniform_matrix(DIMENSIONS.0, DIMENSIONS.1, CAMERA_SHIFT.0, CAMERA_SHIFT.1);
    }
}

fn handle_window_event(events: &GlfwReceiver<(f64, WindowEvent)>, window: &mut glfw::PWindow) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => unsafe {
                LEFT = true;
            },
            glfw::WindowEvent::Key(Key::Left, _, Action::Release, _) => unsafe {
                LEFT = false;
            },
            glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => unsafe {
                RIGHT = true;
            },
            glfw::WindowEvent::Key(Key::Right, _, Action::Release, _) => unsafe {
                RIGHT = false;
            },
            glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => unsafe {
                UP = true;
            },
            glfw::WindowEvent::Key(Key::Up, _, Action::Release, _) => unsafe {
                UP = false;
            },
            glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => unsafe {
                DOWN = true;
            },
            glfw::WindowEvent::Key(Key::Down, _, Action::Release, _) => unsafe {
                DOWN = false;
            },
            glfw::WindowEvent::Key(Key::Enter, _, Action::Press, _) => unsafe {
                ENTER = true;
            },
            glfw::WindowEvent::Key(Key::Enter, _, Action::Release, _) => unsafe {
                ENTER = false;
            },
            glfw::WindowEvent::Key(Key::Backspace, _, Action::Press, _) => unsafe {
                ERASE = true;
            },
            glfw::WindowEvent::Key(Key::Backspace, _, Action::Release, _) => unsafe {
                ERASE = false;
            },
            glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => unsafe {
                NUM1 = true;
            },
            glfw::WindowEvent::Key(Key::Num1, _, Action::Release, _) => unsafe {
                NUM1 = false;
            },
            glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => unsafe {
                NUM2 = true;
            },
            glfw::WindowEvent::Key(Key::Num2, _, Action::Release, _) => unsafe {
                NUM2 = false;
            },
            glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => unsafe {
                NUM3 = true;
            },
            glfw::WindowEvent::Key(Key::Num3, _, Action::Release, _) => unsafe {
                NUM3 = false;
            },
            glfw::WindowEvent::Key(Key::Num4, _, Action::Press, _) => unsafe {
                NUM4 = true;
            },
            glfw::WindowEvent::Key(Key::Num4, _, Action::Release, _) => unsafe {
                NUM4 = false;
            },
            glfw::WindowEvent::Key(Key::Num5, _, Action::Press, _) => unsafe {
                NUM5 = true;
            },
            glfw::WindowEvent::Key(Key::Num5, _, Action::Release, _) => unsafe {
                NUM5 = false;
            },
            glfw::WindowEvent::Key(Key::Num6, _, Action::Press, _) => unsafe {
                NUM6 = true;
            },
            glfw::WindowEvent::Key(Key::Num6, _, Action::Release, _) => unsafe {
                NUM6 = false;
            },
            glfw::WindowEvent::Key(Key::Num7, _, Action::Press, _) => unsafe {
                NUM7 = true;
            },
            glfw::WindowEvent::Key(Key::Num7, _, Action::Release, _) => unsafe {
                NUM7 = false;
            },
            glfw::WindowEvent::Key(Key::Num8, _, Action::Press, _) => unsafe {
                NUM8 = true;
            },
            glfw::WindowEvent::Key(Key::Num8, _, Action::Release, _) => unsafe {
                NUM8 = false;
            },
            glfw::WindowEvent::Key(Key::Num9, _, Action::Press, _) => unsafe {
                NUM9 = true;
            },
            glfw::WindowEvent::Key(Key::Num9, _, Action::Release, _) => unsafe {
                NUM9 = false;
            },
            _ => {}
        }
    }
}

pub fn left_pressed() {
    unsafe {
        resize_camera(DIMENSIONS.0, DIMENSIONS.1, -SHIFT_AMOUNT, 0.0);
    }
}

pub fn right_pressed() {
    unsafe {
        resize_camera(DIMENSIONS.0, DIMENSIONS.1, SHIFT_AMOUNT, 0.0);
    }
}

pub fn up_pressed() {
    unsafe {
        resize_camera(DIMENSIONS.0, DIMENSIONS.1, 0.0, SHIFT_AMOUNT);
    }
}

pub fn down_pressed() {
    unsafe {
        resize_camera(DIMENSIONS.0, DIMENSIONS.1, 0.0, -SHIFT_AMOUNT);
    }
}
// Simple smoke test for HashGrid
fn run_hashgrid_smoke() {
    use std::collections::HashSet;
    println!("Running HashGrid smoke test...");
    let screen = ((0.0f32, 0.0f32), (100.0f32, 100.0f32));
    let mut hg: crate::hashgrid::HashGrid<u32> = HashGrid::new(screen, 10);

    println!("Initial grid:\n{:#?}", hg);

    // Insert rectangle centered at (15,15) with size 10x10 and key 1
    let res = hg.insert_rectangle((15.0, 15.0), (10.0, 10.0), &1u32);
    println!("After insert result: {:?}\n{:#?}", res, hg);

    // Try moving the rectangle to (45,45)
    let res_move = hg.move_rectangle((15.0, 15.0), (10.0, 10.0), (45.0, 45.0));
    println!("After move result: {:?}\n{:#?}", res_move, hg);

    // Remove rectangle at new location
    let res_rem = hg.remove_rectangle((45.0, 45.0), (10.0, 10.0), &1u32);
    println!("After remove result: {:?}\n{:#?}", res_rem, hg);
}
