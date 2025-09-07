use gl::DebugMessageInsert;

use crate::c_side::{self, load_all_text_vbo};
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
    -f32::atan2($p2.y - $p1.y, $p2.x - $p1.x)//y coordinate reversed 
  };
}
macro_rules! distance_between_points {
  ($p1:expr, $p2:expr) => {
    f32::sqrt(($p2.x - $p1.x).powi(2) + ($p2.y - $p1.y).powi(2))
  };
}

macro_rules! perpendicular_point{
  ($center:expr, $distance:expr, $radian:expr) => {
    {
      let perpendicular_angle = $radian + PI /2.0;
      Point{x: $center.x + $distance as f32 * perpendicular_angle.cos(),
            y: $center.y - $distance as f32 * perpendicular_angle.sin()}
    }
  }
}

macro_rules! rotate_around {
  ($center:expr, $rotating:expr, $radian:expr) => {
    {
      let difference: Point = Point{x: $rotating.x - $center.x, y: $rotating.y - $center.y};
      Point {
        x: $center.x + (difference.x * $radian.cos() - difference.y * $radian.sin()),
        y: $center.y + (difference.x * $radian.sin() + difference.y * $radian.cos()),
      }
    }
  };
}

const DEFAULT_R: c_float = 1.0;
const DEFAULT_G: c_float = 1.0;
const DEFAULT_B: c_float = 1.0;
const HIGHLIGHT_R: c_float = 1.0;
const HIGHLIGHT_G: c_float = 0.0;
const HIGHLIGHT_B: c_float = 0.0;
const DEFAULT_THICKNESS: c_float = 5.0;
const LINE_HEIGHT: c_float = 2.0;
const TRIANGLE_WIDTH_RATIO_LINE_HEIGHT: c_float = 5.0 * LINE_HEIGHT; 
const TRIANGLE_HIGHT:c_float = 10.0;
const LINE_STOE_RATIO: c_float = 0.3;
const CENTERING_RATIO: c_float = 0.9;
const WEIGHT_SIZE: c_int = 18;
const WBOTTOM_DISTANCE:c_int = 10;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Point{
  pub x: c_float,
  pub y: c_float,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct IntPoint{
  pub x: c_int,
  pub y: c_int,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CS{
  Circle(c_float),//radius
  Square(c_float),//width
  Rectangle(c_float, c_float),//width, height
  Removed,
}

#[derive(Debug)]
pub enum LineState{
  StartToEnd (c_uint),
  Nodirection,
  Novisual,
  Removed,
}

const RED: (c_float, c_float, c_float) = (1.0, 0.0, 0.0);
const GREEN: (c_float, c_float, c_float) = (0.0, 1.0, 0.0);
const BLUE: (c_float, c_float, c_float) = (0.0, 0.0, 1.0);
const YELLOW: (c_float, c_float, c_float) = (1.0, 1.0, 0.0);
const ORANGE: (c_float, c_float, c_float) = (1.0, 0.5, 0.0);
const PURPLE: (c_float, c_float, c_float) = (0.5, 0.0, 1.0);
const DEFAULT_COLOR: (c_float, c_float, c_float) = (DEFAULT_R, DEFAULT_G, DEFAULT_B);

#[derive(Debug, Copy, Clone)]
pub enum NodeColor {
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Purple,
    Default,
}

impl NodeColor {
    fn to_rgb(&self) -> (c_float, c_float, c_float) {
        match self {
            NodeColor::Red => RED,
            NodeColor::Green => GREEN,
            NodeColor::Blue => BLUE,
            NodeColor::Yellow => YELLOW,
            NodeColor::Orange => ORANGE,
            NodeColor::Purple => PURPLE,
            NodeColor::Default => DEFAULT_COLOR,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Node{
  distance: CS,
  shape_id: c_uint,
  text_id: c_uint,
  weight_id: Option<c_uint>,
}
//location is the location of the shape and scale also the scale of the shape so access that

#[derive(Debug)]
#[allow(dead_code)]
pub struct Line{
  state: LineState,
  line_id: c_uint,
  weight_id: c_uint,
  pub start: Point,
  pub end: Point,
}

impl Point {
  pub fn new(x: c_float, y: c_float) -> Self{
    Point{x,y,}
  }
}

impl Line{

    pub fn new(way: LineState, start: Point, end: Point, weight_str: &str) -> Self {
      let angle = angle_between_points!(start, end);
      let middle: Point = average_point!(start, end);
      let distance = distance_between_points!(start, end) / 2.0;
      let weight_center: Point = perpendicular_point!(middle,(WEIGHT_SIZE/2 + WBOTTOM_DISTANCE), angle);//perpendicular to line
    unsafe {
      let new_line: Line;
      match way {
        LineState::StartToEnd(_) => {
          new_line = Line {
            state: LineState::StartToEnd(
              c_side::new_triangle(
                middle.x + (distance * LINE_STOE_RATIO * angle.cos()),
                middle.y + (distance * LINE_STOE_RATIO * -angle.sin()),
                TRIANGLE_WIDTH_RATIO_LINE_HEIGHT,
                TRIANGLE_HIGHT,
                angle + PI / 2.0,
                DEFAULT_R,
                DEFAULT_G,
                DEFAULT_B
              )
            ),
            start,
            end,
            weight_id: c_side::create_text_centered(CString::new(weight_str).expect("Error cstr").as_ptr(),
              weight_center.x as i32, weight_center.y as i32,
              distance as i32, WEIGHT_SIZE, 0.0),
            line_id: c_side::new_line(middle.x, middle.y, distance, LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B),
          };
        }
        LineState::Nodirection => {
          new_line = Line {
            state: LineState::Nodirection,
            start,
            end,
            weight_id: c_side::create_text_centered(CString::new(weight_str).expect("Error cstr").as_ptr(),
              weight_center.x as i32, weight_center.y as i32,
              distance as i32, WEIGHT_SIZE, 0.0),
            line_id: c_side::new_line(middle.x, middle.y, distance, LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B),
          };
        }
        LineState::Novisual => {
            new_line = Line {
                state: LineState::Novisual,
                start,
                end,
                line_id: 0,
                weight_id: 0,
            };
        }
        LineState::Removed => {
          panic!("Invalid");
        }
      };
      if !matches!(way, LineState::Novisual) {
        load_all_text_vbo();
      }
      new_line
    }
  }

  //This does not change the weight text
  pub fn override_line(&mut self,start: Point, end: Point){
    let angle = angle_between_points!(start, end);
    let middle: Point = average_point!(start,end);
    let distance = distance_between_points!(start,end) / 2.0;

    let weight_center: Point = perpendicular_point!(middle,(WEIGHT_SIZE/2 + WBOTTOM_DISTANCE), angle);//perpendicular to line

    unsafe{
      match self.state {
        LineState::StartToEnd(id) => {
          // Overriding triangle sprite (arrow head)
          c_side::override_sprite(id, middle.x + (distance*LINE_STOE_RATIO*f32::cos(angle)),
          middle.y + (distance * LINE_STOE_RATIO*f32::sin(-angle)),
          TRIANGLE_WIDTH_RATIO_LINE_HEIGHT, TRIANGLE_HIGHT, angle + PI/2.0, DEFAULT_R, DEFAULT_G, DEFAULT_B);
          self.start = start;
          self.end = end;
          // Overriding line sprite
        }
        LineState::Nodirection => {
          self.start = start;
          self.end = end;
          // Overriding line sprite
        }
        LineState::Novisual => {
            self.start = start;
            self.end = end;
        }
        LineState::Removed => {
          panic!("Invalid");
        }
      }
      if !matches!(self.state, LineState::Novisual) {
        c_side::override_sprite(self.line_id, middle.x, middle.y, distance, 
          LINE_HEIGHT, angle, DEFAULT_R, DEFAULT_G, DEFAULT_B);
        c_side::move_text(self.weight_id, weight_center.x as i32, weight_center.y as i32);
        c_side::rotate_text(self.weight_id, angle);
        c_side::load_all_text_vbo();
      }
    }
  }

  pub fn remove_line(&mut self){
    unsafe{
      if let LineState::Removed = self.state {
      panic!("Invalid op");
      }
      let is_visual = !matches!(self.state, LineState::Novisual);
      match self.state {
        LineState::StartToEnd(id) => {
          c_side::remove_sprite(id);
        }
        LineState::Nodirection => {
        }
        LineState::Novisual => {
        }
        LineState::Removed => unreachable!(),
      }
      self.state = LineState::Removed;
      if is_visual {
        c_side::remove_sprite(self.line_id);
        c_side::remove_text(self.weight_id);
        c_side::load_all_text_vbo();
      }
    }
  }
  pub fn weight_line(&mut self, weight_str: &str){
    match self.state {
      LineState::Removed => unreachable!(),
      _ => {
        unsafe{
          c_side::remove_text(self.weight_id);
          let angle = angle_between_points!(self.start, self.end);
          let middle: Point = average_point!(self.start,self.end);
          let distance = distance_between_points!(self.start,self.end) / 2.0;
          let weight_center: Point = perpendicular_point!(middle,(WEIGHT_SIZE/2 + WBOTTOM_DISTANCE), angle);//perpendicular to line
          self.weight_id = c_side::create_text_centered(CString::new(weight_str).expect("Error cstr").as_ptr(),
            weight_center.x as i32, weight_center.y as i32,
            distance as i32, WEIGHT_SIZE, 0.0);
          c_side::load_all_text_vbo();
        }
      }
    }

  }
}



impl Node {
  pub fn new(shape_distance: CS, text: &str, center_x: c_float, center_y: c_float, color: NodeColor) -> Self {
    let mut bounding_width: c_float;
    let mut bounding_height: c_float;
    let (r, g, b) = color.to_rgb();
    let send_text = CString::new(text).expect("Error");

    let node = match shape_distance {
      CS::Circle(radius) => {
        bounding_width = 3.0 * radius * f32::cos(PI / 4.0);
        bounding_width -= 3.0 * DEFAULT_THICKNESS;
        bounding_width *= CENTERING_RATIO;
        bounding_height = bounding_width;

        unsafe {
          Node {
            distance: shape_distance,
            text_id: c_side::create_text_centered(
              send_text.as_ptr(),
              center_x as i32,
              center_y as i32,
              bounding_width as i32,
              bounding_height as i32,
              0.0,
            ),
            shape_id: c_side::new_circle(center_x, center_y, radius, DEFAULT_THICKNESS, r, g, b),
            weight_id: None,
          }
        }
      }
      CS::Square(edge_length) => {
        bounding_width = 3.0 * (edge_length - DEFAULT_THICKNESS);
        bounding_width *= CENTERING_RATIO;
        bounding_height = bounding_width;

        unsafe {
          Node {
            distance: shape_distance,
            text_id: c_side::create_text_centered(
              send_text.as_ptr(),
              center_x as i32,
              center_y as i32,
              bounding_width as i32,
              bounding_height as i32,
              0.0,
            ),
            shape_id: c_side::new_square(center_x, center_y, edge_length, DEFAULT_THICKNESS, r, g, b),
            weight_id: None,
          }
        }
      }
      CS::Rectangle(width, height) => {
        bounding_width = 3.0 * (width - DEFAULT_THICKNESS);
        bounding_height = 3.0 * (height - DEFAULT_THICKNESS);
        bounding_width *= CENTERING_RATIO;
        bounding_height *= CENTERING_RATIO;

        unsafe {
          Node {
            distance: shape_distance,
            text_id: c_side::create_text_centered(
              send_text.as_ptr(),
              center_x as i32,
              center_y as i32,
              bounding_width as i32,
              bounding_height as i32,
              0.0,
            ),
            shape_id: c_side::new_rectangle(center_x, center_y, width, height, DEFAULT_THICKNESS, r, g, b),
            weight_id: None,
          }
        }
      }
      CS::Removed => {
        panic!("Invalid");
      }
    };
    unsafe { c_side::load_all_text_vbo();}
    node
  }
  pub fn move_node(&self, x: c_float, y: c_float){
    unsafe{
      if let CS::Removed = self.distance {
        panic!("Invalid op");
      }
      let text_obj = c_side::get_text(self.text_id);
      let sprite_obj = c_side::get_sprite(self.shape_id);
      c_side::move_text(self.text_id,
        x as i32, y as i32);
        // (*sprite_obj).x  as i32,
        // (*sprite_obj).y as i32);
      if let Some(weight_id) = self.weight_id {
        let weight_obj = c_side::get_text(weight_id);
        c_side::move_text(weight_id, x as i32,
          (y - (*sprite_obj).height / 2.0 - WEIGHT_SIZE as f32 - WBOTTOM_DISTANCE as f32) as i32);
      }
      c_side::move_sprite(self.shape_id, x, y); 
      c_side::load_all_text_vbo();
    }
  }

  pub fn color_node(&mut self, color: NodeColor) {
    let (r, g, b) = color.to_rgb();
    unsafe {
        if let CS::Removed = self.distance {
            panic!("Invalid op");
        }
        let sprite_obj = c_side::get_sprite(self.shape_id);
        c_side::override_sprite(
            self.shape_id,
            (*sprite_obj).x,
            (*sprite_obj).y,
            (*sprite_obj).width,
            (*sprite_obj).height,
            (*sprite_obj).thickness,
            r,
            g,
            b,
        );
    }
  }
  
  pub fn label_node(&mut self, new_text: &str) {
    unsafe {
      if let CS::Removed = self.distance {
        panic!("Invalid op");
      }
      let sprite_obj = c_side::get_sprite(self.shape_id);
      c_side::remove_text(self.text_id);
      self.text_id = c_side::create_text_centered(
        CString::new(new_text).expect("Error cstr").as_ptr(),
        (*sprite_obj).x as i32,
        (*sprite_obj).y as i32,
        (*sprite_obj).width as i32,
        (*sprite_obj).height as i32,
        0.0,
      );
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
          self.text_id = c_side::create_text_centered((*text_obj).text, 
            (*sprite_obj).x as i32, (*sprite_obj).y as i32,
            bounding_width as i32, bounding_height as i32, 0.0);
        }
        CS::Square(_) =>{
          self.distance = CS::Square(width);
          self.text_id = c_side::create_text_centered((*text_obj).text,
            (*sprite_obj).x as i32, (*sprite_obj).y as i32,
            (width*2.0) as i32, (height*2.0) as i32, 0.0);
        }
        CS::Rectangle(_, _) =>{
          self.distance = CS::Rectangle(width, height);
          self.text_id = c_side::create_text_centered((*text_obj).text,
            (*sprite_obj).x as i32, (*sprite_obj).y as i32,
            (width*2.0) as i32, (height*2.0) as i32, 0.0);
        }
        CS::Removed =>{
          panic!("Removed");
        }
      }
      c_side::remove_text(old_text_id);
      // Update or recreate weight text if present
      if let Some(wid) = self.weight_id {
        let weight_obj = c_side::get_text(wid);
        let old_weight_id = wid;
        // For simplicity, just recreate at the same position with new size
        self.weight_id = Some(c_side::create_text_centered(
          (*weight_obj).text,
          (*sprite_obj).x as i32,
          ((*sprite_obj).y - (*sprite_obj).height / 2.0 - WEIGHT_SIZE as f32 - WBOTTOM_DISTANCE as f32) as i32,
          (width*2.0) as i32, (height*2.0) as i32, 0.0));
        c_side::remove_text(old_weight_id);
      }
      c_side::load_all_text_vbo();
    }
  }

  pub fn remove_node(&mut self){
    unsafe{
      if let CS::Removed = self.distance {
        panic!("Invalid op");
      }
      c_side::remove_sprite(self.shape_id);
      c_side::remove_text(self.text_id);
      if let Some(wid) = self.weight_id {
        c_side::remove_text(wid);
        self.weight_id = None;
      }
      c_side::load_all_text_vbo();
      self.distance = CS::Removed;
    }
  }
  
  pub fn weight_node(&mut self, weight: &str) {
    unsafe {
        if let CS::Removed = self.distance {
            panic!("Invalid op");
        }
        let sprite_obj = c_side::get_sprite(self.shape_id);
        if let Some(old_weight_id) = self.weight_id {
            c_side::remove_text(old_weight_id);
        }
        self.weight_id = Some(c_side::create_text_centered(
            CString::new(weight).expect("Error cstr").as_ptr(),
            (*sprite_obj).x as i32,
            ((*sprite_obj).y - (*sprite_obj).height / 2.0 - WEIGHT_SIZE as f32 - WBOTTOM_DISTANCE as f32) as i32,
            (*sprite_obj).width as i32,
            WEIGHT_SIZE,
            0.0,
        ));
        c_side::load_all_text_vbo();
    }
  }
}
