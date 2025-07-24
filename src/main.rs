//use tree_sitter::{InputEdit, Language, Parser, Point};
mod c_side;
mod tree;
use glfw::{fail_on_errors, Action, Context, Key, WindowEvent,WindowHint, OpenGlProfileHint, WindowMode};
use std::{f32::consts::PI, ffi::CString};

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(4, 5));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::Samples(Some(4)));
    glfw.window_hint(glfw::WindowHint::AlphaBits(Some(8)));

    let (mut window, events) = glfw.create_window(1920, 1080, "RustGL", WindowMode::Windowed).unwrap();
        //.expect("Failed to create window!");
    window.make_current();
    window.set_key_polling(true);
    // the supplied function must be of the type:
    // `&fn(symbol: &'static str) -> *const std::os::raw::c_void`
    gl::load_with(|s| window.get_proc_address(s));
    unsafe {
        c_side::initialize_render();
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::MULTISAMPLE);
        gl::Disable(gl::DEPTH_TEST);  // If applicable
    }
    unsafe{
        c_side::new_sprite_renderer();
        c_side::initialize_font_renderer();
        c_side::initialize_font(CString::new("include/font/Unnamed.fnt").expect("Error").as_ptr());
    }

    unsafe {

        // Test Node creation
        let mut square_node = tree::Node::new( tree::CS::Square(60.0), 
            "1", 
            500.0, 
            300.0, 
            tree::Highlight::No
        );
        let mut rect_node = tree::Node::new(
            tree::CS::Rectangle(70.0, 50.0), 
            "2", 
            700.0, 
            300.0, 
            tree::Highlight::Yes
        );
        let mut circle_node = tree::Node::new(
            tree::CS::Circle(50.0), 
            "3", 
            300.0, 
            300.0, 
            tree::Highlight::No
        );
        let mut circle_node1 = tree::Node::new(
            tree::CS::Circle(50.0), 
            "4", 
            700.0, 
            300.0, 
            tree::Highlight::No
        );
        let mut circle_node2 = tree::Node::new(
            tree::CS::Circle(50.0), 
            "5", 
            0.0, 
            300.0, 
            tree::Highlight::No
        );

        // Test Line creation
        let mut bidirectional_line = tree::Line::new(
            tree::LineState::Bidirectional(0, 0),
            tree::Point::new(300.0, 400.0),
            tree::Point::new(500.0, 400.0)
        );
        let mut start_to_end_line = tree::Line::new(
            tree::LineState::StartToEnd(0),
            tree::Point::new(300.0, 450.0),
            tree::Point::new(500.0, 450.0)
        );
        let mut end_to_start_line = tree::Line::new(
            tree::LineState::EndToStart(0),
            tree::Point::new(300.0, 500.0),
            tree::Point::new(500.0, 500.0)
        );
        let mut no_dir_line = tree::Line::new(
            tree::LineState::Nodirection,
            tree::Point::new(300.0, 550.0),
            tree::Point::new(500.0, 550.0)
        );

        // Test node movement
        circle_node.move_node(350.0, 350.0);
        square_node.move_node(550.0, 350.0);
        rect_node.move_node(750.0, 350.0);

        // Test line movement
        bidirectional_line.override_line(
            tree::Point::new(350.0, 400.0),
            tree::Point::new(550.0, 400.0)
        );
        start_to_end_line.override_line(
            tree::Point::new(350.0, 450.0),
            tree::Point::new(550.0, 450.0)
        );
        end_to_start_line.override_line(
            tree::Point::new(350.0, 500.0),
            tree::Point::new(550.0, 500.0)
        );
        no_dir_line.override_line(
            tree::Point::new(350.0, 550.0),
            tree::Point::new(550.0, 550.0)
        );
        let mut cleanup_done = false;
        while !window.should_close() {
            window.swap_buffers();
            glfw.poll_events();

            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true)
                    },
                    glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                        if !cleanup_done {
                            println!("Running cleanup tests...");
                            
                            // Test removal
                            circle_node.remove_node();
                            square_node.remove_node();
                            rect_node.remove_node();
                            
                            bidirectional_line.remove_line();
                            start_to_end_line.remove_line();
                            end_to_start_line.remove_line();
                            no_dir_line.remove_line();
                            
                            println!("Cleanup tests complete. All objects should be removed.");
                            cleanup_done = true;
                        }
                    }
                    _ => {},
                }
            }

            unsafe {
                gl::ClearColor(0.3, 0.8, 0.2, 0.5);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                c_side::render_text();
                c_side::draw_sprites();
            }
        }
    }
}

fn handle_window_event(window:&mut glfw::Window, event: glfw::WindowEvent){
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        },
        glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
            println!("Good Job\n");
        }
        _ => {},
    }
}
