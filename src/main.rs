//use tree_sitter::{InputEdit, Language, Parser, Point};
mod c_side;
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
        println!("Blending enabled: {}", gl::IsEnabled(gl::BLEND) == gl::TRUE);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        println!("Blend function set to SRC_ALPHA, ONE_MINUS_SRC_ALPHA");
        gl::Enable(gl::MULTISAMPLE);
        println!("Multisampling enabled: {}", gl::IsEnabled(gl::MULTISAMPLE) == gl::TRUE);
        gl::Disable(gl::DEPTH_TEST);  // If applicable
        println!("Depth test disabled: {}", gl::IsEnabled(gl::DEPTH_TEST) == gl::FALSE);
    }
    unsafe{
        c_side::new_sprite_renderer();
        c_side::initialize_font_renderer();
        c_side::initialize_font(CString::new("include/font/Unnamed.fnt").expect("Error").as_ptr());
        c_side::create_text(CString::new("Something").expect("Error").as_ptr(), 600.0, 700.0);

        let line_id = c_side::new_line(200.0, 200.0, 40.0, 3.0, 1.0, 0.0, 0.0, 0.0);
        let circle_id = c_side::new_circle(400.0, 200.0, 30.0, 3.0, 0.0, 0.0, 0.0);
        let triangle_id = c_side::new_triangle(600.0, 200.0, 50.0, 50.0, 0.0, 0.0, 1.0, 0.0);
        let rectangle_id = c_side::new_rectangle(800.0, 200.0, 60.0, 40.0, 2.0, 0.0, 0.0, 1.0);
        let square_id = c_side::new_square(1000.0, 200.0, 50.0, 2.0, 1.0, 1.0, 0.0);

        c_side::move_sprite(line_id, 250.0, 350.0);
        c_side::move_sprite(circle_id, 450.0, 350.0);
        c_side::move_sprite(triangle_id, 650.0, 350.0);
        c_side::move_sprite(rectangle_id, 850.0, 350.0);
        c_side::move_sprite(square_id, 1050.0, 350.0);

        // Test changing colors
        c_side::remove_sprite(square_id);
        c_side::sprite_cleanup();
        c_side::color_sprite(line_id, 1.0, 0.0, 1.0); // magenta line
        c_side::color_sprite(circle_id, 0.0, 1.0, 1.0); // cyan circle
        c_side::color_sprite(triangle_id, 1.0, 1.0, 0.0); // yellow triangle
        c_side::color_sprite(rectangle_id, 0.5, 0.5, 0.5); // gray rectangle

        // Test scaling
        c_side::scale_sprite(line_id, 60.0, 5.0, 7.0); // make line longer and thicker
        c_side::scale_sprite(circle_id, 40.0, 40.0, 5.0); // bigger circle with thicker border
        c_side::scale_sprite(triangle_id, 70.0, 70.0, 0.0); // larger triangle
        c_side::scale_sprite(rectangle_id, 80.0, 60.0, 4.0); // larger rectangle with thicker border
        //
        // // Test rotation (where applicable)
        c_side::rotate_sprite(line_id, 45.0); // rotate line 45 degrees
        c_side::rotate_sprite(triangle_id,PI ); // flip triangle upside down
        // c_side::rotate_sprite(rectangle_id, 90.0); // rotate rectangle 90 degrees
        //
        // // Verify operations
        println!("Post-test GL error: {}", gl::GetError());

        println!("{0}", gl::GetError());
    }
    while !window.should_close() {
       window.swap_buffers();
    // poll for and process events
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
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
        unsafe{
            gl::ClearColor(0.3, 0.8, 0.2, 0.5);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            c_side::render_text();
            c_side::draw_sprites();
        }
    }
    unsafe {
        c_side::destroy_renderer();
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
