use crate::c_side;
use crate::c_side::get_text;
use crate::CString;
use core::ffi::{c_char, c_float, c_int, c_uint, c_void};
use std::f32::consts::PI;

//u32 is the radius, distance here.
macro_rules! average_point {
    ($p1:expr, $p2:expr) => {
        Point {
            x: ($p1.x + $p2.x) / 2.0,
            y: ($p1.y + $p2.y) / 2.0,
        }
    };
}

macro_rules! angle_between_points {
    ($p1:expr, $p2:expr) => {
        f32::atan2($p2.y - $p1.y, $p2.x - $p1.x)
    };
}
macro_rules! distance_between_points {
    ($p1:expr, $p2:expr) => {
        f32::sqrt(($p2.x - $p1.x).powi(2) + ($p2.y - $p1.y).powi(2))
    };
}

const DEFAULT_R: c_float = 1.0;
const DEFAULT_G: c_float = 1.0;
const DEFAULT_B: c_float = 1.0;
const HIGHLIGHT_R: c_float = 1.0;
const HIGHLIGHT_G: c_float = 0.0;
const HIGHLIGHT_B: c_float = 0.0;
const DEFAULT_THICKNESS: c_float = 7.0;
const LINE_HEIGHT: c_float = 3.0;
const TRIANGLE_WIDTH_RATIO_LINE_HEIGHT: c_float = 3.0 * LINE_HEIGHT; 
const TRIANGLE_HIGHT:c_float = 20.0;
const LINE_STOE_RATIO: c_float = 0.7;
const CENTERING_RATIO: c_float = 0.9;

#[allow(dead_code)]
pub struct Point{
    pub x: c_float,
    pub y: c_float,
}

#[allow(dead_code)]
pub enum CS{
    Circle(c_float),
    Square(c_float),
    Rectangle(c_float, c_float),
    Removed,
}

pub enum LineState{
    Bidirectional(c_uint, c_uint),//first starttoend, second endtostart
    StartToEnd(c_uint),
    EndToStart(c_uint),
    Nodirection,
    Removed,
}

pub enum Highlight{
    Yes,
    No,
}

#[allow(dead_code)]
pub struct Node{
    distance: CS,
    shape_id: c_uint,
    text_id: c_uint,
}
//location is the location of the shape and scale also the scale of the shape so access that

#[allow(dead_code)]
pub struct Line{
    state: LineState,
    line_id: c_uint,
    start: Point,
    end: Point,
}

impl Point {
    pub fn new(x: c_float, y: c_float) -> Self{
        Point{x,y,}
    }
}

impl Line{
    pub fn new(way: LineState,start: Point, end: Point) -> Self{
        let angle = angle_between_points!(start, end);
        let middle: Point = average_point!(start,end);
        let distance = distance_between_points!(start,end) / 2.0;
        unsafe{
            match way {
                LineState::Bidirectional(_,_) => {
                    Line{
                        state: LineState::Bidirectional(
                                   c_side::new_triangle(middle.x + (distance*LINE_STOE_RATIO*f32::cos(angle)),
                                   middle.y + (distance * LINE_STOE_RATIO*f32::sin(angle)),
                                    TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT,angle + PI + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B),
                                   c_side::new_triangle(middle.x - (distance*LINE_STOE_RATIO*f32::cos(angle)),
                                   middle.y - (distance * LINE_STOE_RATIO*f32::sin(angle)),
                                   TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT, angle + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B),
                                   ),
                        start,
                        end,
                        line_id: c_side::new_line(middle.x, middle.y,distance , LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B),
                    }
                }
                LineState::StartToEnd(_) => {
                    Line{
                        state: LineState::StartToEnd(
                                   c_side::new_triangle(middle.x + (distance*LINE_STOE_RATIO*f32::cos(angle)),
                                   middle.y + (distance * LINE_STOE_RATIO*f32::sin(angle)),
                                    TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT,angle + PI + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B)),
                        start,
                        end,
                        line_id: c_side::new_line(middle.x, middle.y,distance , LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B),
                    }
                }
                LineState::EndToStart(_) => {
                    Line{
                        state: LineState::EndToStart(
                                   c_side::new_triangle(middle.x - (distance*LINE_STOE_RATIO*f32::cos(angle)),
                                   middle.y - (distance * LINE_STOE_RATIO*f32::sin(angle)),
                                   TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT, angle + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B)),
                        start,
                        end,
                        line_id: c_side::new_line(middle.x, middle.y,distance , LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B),
                    }
                }
                LineState::Nodirection => {
                    Line{
                        state: LineState::Nodirection,
                        start,
                        end,
                        line_id: c_side::new_line(middle.x, middle.y,distance , LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B),
                    }
                }
                LineState::Removed => {
                    panic!("Invalid");
                }
            }

        }
    }
    pub fn override_line(&mut self,start: Point, end: Point){
        let angle = angle_between_points!(start, end);
        let middle: Point = average_point!(start,end);
        let distance = distance_between_points!(start,end) / 2.0;
        unsafe{
            match self.state {
                LineState::Bidirectional(first_id,second_id) => {
                        // Overriding triangle sprite (first arrow head)
                        c_side::override_sprite(first_id,middle.x + (distance*LINE_STOE_RATIO*f32::cos(angle)),
                        middle.y + (distance * LINE_STOE_RATIO*f32::sin(angle)),
                        TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT,angle + PI + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                        // Overriding triangle sprite (second arrow head)
                        c_side::override_sprite(second_id,middle.x - (distance*LINE_STOE_RATIO*f32::cos(angle)),
                        middle.y - (distance * LINE_STOE_RATIO*f32::sin(angle)),
                        TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT, angle + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                        self.start = start; self.end = end;
                        // Overriding line sprite
                        c_side::override_sprite(self.line_id,middle.x, middle.y,distance , LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                    }
                LineState::StartToEnd(id) => {
                    // Overriding triangle sprite (arrow head)
                    c_side::override_sprite(id, middle.x + (distance*LINE_STOE_RATIO*f32::cos(angle)),
                    middle.y + (distance * LINE_STOE_RATIO*f32::sin(angle)),
                    TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT, angle + PI + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                    self.start = start;
                    self.end = end;
                    // Overriding line sprite
                    c_side::override_sprite(self.line_id, middle.x, middle.y, distance, LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                }
                LineState::EndToStart(id) => {
                    // Overriding triangle sprite (arrow head)
                    c_side::override_sprite(id, middle.x - (distance*LINE_STOE_RATIO*f32::cos(angle)),
                    middle.y - (distance * LINE_STOE_RATIO*f32::sin(angle)),
                    TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT, angle + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                    self.start = start;
                    self.end = end;
                    // Overriding line sprite
                    c_side::override_sprite(self.line_id, middle.x, middle.y, distance, LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                }
                LineState::Nodirection => {
                    self.start = start;
                    self.end = end;
                    // Overriding line sprite
                    c_side::override_sprite(self.line_id, middle.x, middle.y, distance, LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B);
                }
                LineState::Removed => {
                    panic!("Invalid");
                }
            }
        }
    }
    pub fn remove_line(&mut self){
        unsafe{
            if let LineState::Removed = self.state {
                panic!("Invalid op");
            }
            match self.state {
                LineState::Bidirectional(first_id, second_id) => {
                    c_side::remove_sprite(first_id);
                    c_side::remove_sprite(second_id);
                    c_side::remove_sprite(self.line_id);
                }
                LineState::StartToEnd(id) => {
                    c_side::remove_sprite(id);
                    c_side::remove_sprite(self.line_id);
                }
                LineState::EndToStart(id) => {
                    c_side::remove_sprite(id);
                    c_side::remove_sprite(self.line_id);
                }
                LineState::Nodirection => {
                    c_side::remove_sprite(self.line_id);
                }
                LineState::Removed => unreachable!(),
            }
            self.state = LineState::Removed;
        }
    }
}
    


impl Node{
    pub fn new(shape_distance: CS,text: &str, center_x: c_float, center_y: c_float, highlight: Highlight) -> Self {
        let mut bounding_width: c_float;
        let mut bounding_height: c_float;
        let r: c_float;
        let g: c_float;
        let b: c_float;
        let send_text = CString::new(text).expect("Error");

        if let highlight = Highlight::Yes {
            r = HIGHLIGHT_R;
            g = HIGHLIGHT_G;
            b = HIGHLIGHT_B;
        }
        else{
            r = DEFAULT_R;
            g = DEFAULT_G;
            b = DEFAULT_B;
        }

        match shape_distance{
            CS::Circle(radius) =>{
                bounding_width = 2.0 * radius * f32::cos(PI / 4.0);
                bounding_width -= 2.0 * DEFAULT_THICKNESS;
                bounding_width *= CENTERING_RATIO;
                bounding_height = bounding_width;

                unsafe{
                    Node{distance: shape_distance,
                        text_id: c_side::create_centered_text( send_text.as_ptr(),
                        center_x, center_y,bounding_width,bounding_height),
                        shape_id: c_side::new_circle(center_x, center_y, radius , DEFAULT_THICKNESS, r, g, b)
                    }
                }
            }
            CS::Square(edge_length) =>{
                bounding_width = 2.0 * (edge_length - DEFAULT_THICKNESS);
                bounding_width *= CENTERING_RATIO;
                bounding_height = bounding_width;

                unsafe{
                    Node{distance: shape_distance,
                        text_id: c_side::create_centered_text( send_text.as_ptr(),
                        center_x, center_y,bounding_width,bounding_height),
                        shape_id: c_side::new_square(center_x, center_y, edge_length , DEFAULT_THICKNESS, r, g, b)
                    }
                }
            }
            CS::Rectangle(width,height) => {
                bounding_width = 2.0 * (width - DEFAULT_THICKNESS);
                bounding_height = 2.0 * (height - DEFAULT_THICKNESS);
                bounding_width *= CENTERING_RATIO;
                bounding_height *= CENTERING_RATIO;

                unsafe{
                    Node{distance: shape_distance,
                        text_id: c_side::create_centered_text( send_text.as_ptr(),
                        center_x, center_y,bounding_width,bounding_height),
                        shape_id: c_side::new_rectangle(center_x, center_y, width,height , DEFAULT_THICKNESS, r, g, b)
                    }
                }
            }
            CS::Removed => {
                panic!("Invalid");
            }
        }

    }

    pub fn move_node(&self, x: c_float, y: c_float){
        unsafe{
            if let CS::Removed = self.distance {
                panic!("Invalid op");
            }
            let text_obj = c_side::get_text(self.text_id);
            let sprite_obj = c_side::get_sprite(self.shape_id);
            c_side::move_text(self.text_id,
                x - ((*sprite_obj).x - (*text_obj).x),
                y - ((*sprite_obj).y - (*text_obj).y));
            c_side::move_sprite(self.shape_id, x, y); 
        }
    }

    pub fn scale_node(&mut self, width: c_float,  height: c_float){
        unsafe{
            let sprite_obj = c_side::get_sprite(self.shape_id);
            let text_obj = c_side::get_text(self.text_id);
            let old_text_id = self.text_id;
            c_side::scale_sprite(self.shape_id, width, height, (*sprite_obj).thickness); 
            match self.distance{
                CS::Circle(_) =>{
                    let bounding_width = 2.0*width*f32::cos(PI / 4.0);
                    let bounding_height = bounding_width;

                    self.distance = CS::Circle(width);
                    self.text_id = c_side::create_centered_text((*text_obj).text, 
                        (*sprite_obj).x, (*sprite_obj).y,
                        bounding_width, bounding_height);
                }
                CS::Square(_) =>{
                    self.distance = CS::Square(width);
                    self.text_id = c_side::create_centered_text((*text_obj).text,
                        (*sprite_obj).x, (*sprite_obj).y,
                        width*2.0, height*2.0);

                }
                CS::Rectangle(_, _) =>{
                    self.distance = CS::Rectangle(width, height);
                    self.text_id = c_side::create_centered_text((*text_obj).text,
                        (*sprite_obj).x,(*sprite_obj).y,
                        width*2.0, height*2.0);
                }
                CS::Removed =>{
                    panic!("Removed");
                }
            }
            c_side::delete_text(old_text_id);
        }
    }

    pub fn remove_node(&mut self){
        unsafe{
            if let CS::Removed = self.distance {
                panic!("Invalid op");
            }
            c_side::remove_sprite(self.shape_id);
            c_side::delete_text(self.text_id);
            self.distance = CS::Removed;
        }
    }
}
