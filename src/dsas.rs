use crate::{c_side::CircleSquare, tree::{NodeColor, LineState, Node, Point, CS, Line }};
use std::{cmp, collections::{HashMap, HashSet, VecDeque}};
use crate::json_deserialize;
use std::f32::consts::PI;

macro_rules! normal_slope {
    ($slope:expr) => {
      -1.0/$slope
    };
}

#[macro_export]
macro_rules! angle_between_points {
  ($p1:expr, $p2:expr) => {
    -f32::atan2($p2.1 - $p1.1, $p2.0 - $p1.0)//y coordinate reversed 
  };
}

macro_rules! rotate_around {
  ($center:expr, $rotating:expr, $radian:expr) => {
    {
      let difference_x = $rotating.0 - $center.0;
      let difference_y = $rotating.1 - $center.1;
      (
        $center.0 + (difference_x * $radian.cos() + difference_y * $radian.sin()),
        $center.1 + (-difference_x * $radian.sin() + difference_y * $radian.cos()),
      )
    }
  };
}
macro_rules! distance_center {
    ($radian:expr, $parent_to_center :expr) => {
      {
        let distance = (2.0 * DIAGONAL + SPACE_NODES as f32) / ($radian);
        if distance - $parent_to_center < 2.0 * DIAGONAL + SPACE_NODES as f32 {
          $parent_to_center + 2.0 * DIAGONAL + SPACE_NODES as f32
        } 
        else {
          distance
        }
      }
    };
}

#[macro_export]
macro_rules! distance_between_points {
  ($p1:expr, $p2:expr) => {
    {
      f32::sqrt((($p2.0 - $p1.0).powi(2)) + (($p2.1 - $p1.1).powi(2)))
    }
  };
}
// helper macros for creating visual nodes and edges
macro_rules! create_visual_node_at_position {
  ($graph:expr, $node_wrapper:expr, $node_id:expr, $position:expr) => {
    {
      $node_wrapper.center = $position;
      $node_wrapper.visual_node = Some(
        Node::new(CS::Circle(DIAMETER as f32),
          $graph.node_labels[$node_id].as_str(),
          $position.0,
          $position.1,
          NodeColor::Default),
      );
    }
  };
}

macro_rules! create_edge_between_nodes {
  ($graph:expr, $from_id:expr, $to_id:expr, $from_pos:expr, $to_pos:expr, $weight_str:expr) => {
    {
      let angle = angle_between_points!($from_pos, $to_pos);
      if let Some(c_edge) = $graph.edges.get_mut(&($from_id, $to_id)) {
        *c_edge = Line::new(LineState::StartToEnd(0),
          Point::new($from_pos.0 + DIAMETER as f32 * angle.cos(), $from_pos.1 - DIAMETER as f32 * angle.sin()),
          Point::new($to_pos.0 - DIAMETER as f32 * angle.cos(), $to_pos.1 + DIAMETER as f32 * angle.sin()),
          $weight_str);
      }
    }
  };
}
pub const DIAMETER: i32 = 40;
pub const SPACE_NODES: i32 = 10;
pub const DIAGONAL: f32 = (DIAMETER as f32 * 1.414);//sqrt(2) approximately


#[derive(Debug)]
pub struct MLine{
  start: (f32, f32),
  slope: f32,
  intercept: f32,
}

impl MLine{
  pub fn new(start: (f32, f32), slope: f32) -> Self{
    let intercept = start.1 - slope * start.0;
    MLine { start, slope, intercept }
  }

  pub fn with_point(start: (f32, f32), other: (f32,f32)) -> Self{
    let slope = (other.1 - start.1) / (other.0 - start.0);
    Self::new(start, slope)
  }

  pub fn intersects(&self, other: &MLine) -> bool {
    if self.slope == other.slope {
        return false; // Lines are parallel
    }

    let x_intercept = (other.intercept - self.intercept) / (self.slope - other.slope);

    x_intercept >= self.start.0 && x_intercept >= other.start.0
  }
  
  //assume a convex region and the origins must be close not too far apart
  pub fn point_inside(first_line: Self, second_line: Self, point: (i32, i32)) -> bool{
    let fpoint = (point.0 as f32, point.1 as f32);

    let normal_slope_first = normal_slope!(first_line.slope);
    let normal_slope_second = normal_slope!(second_line.slope);
    
    let line_first = MLine::new(fpoint, normal_slope_first);
    let line_second = MLine::new(fpoint, normal_slope_second);
    
    let first_second = line_first.intersects(&second_line);
    let second_first = line_second.intersects(&first_line);

    !first_second && !second_first
  }
}
#[derive(Debug)]
pub struct Graph<'a>{
  node_labels: &'a Vec<String>,
  pub nodes: HashMap<usize,NodeWrapper>,
  pub edges: HashMap<(usize,usize), Line>,//index of first and then second node
  root: usize,
  note: String,
  initial_node_position: (i32,i32),
}

#[derive(Debug)]
pub struct NodeWrapper{
  pub visual_node: Option<Node>,
  pub center: (f32,f32),
  pub neighbors: Vec<usize>,
  pub new_neighbors: u32,
  pub force: (f32, f32),
  pub velocity: (f32, f32),
}

impl<'a> Graph <'a>{
  pub fn new(node_labels: &'a Vec<String>, initial_node_position: (i32,i32), root: usize) -> Self{
    Graph { node_labels,
      nodes: HashMap::new(),
      edges: HashMap::new(),
      root,
      note: String::from(""),
      initial_node_position,
    }
  }

  //bigbang algorithm used here
  pub fn build_base(&mut self, base_info: &'a json_deserialize::BaseInfo) -> &Self{
    
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut visited: HashSet<usize> = HashSet::new();

    let mut new_nodes: HashMap<usize,NodeWrapper > = HashMap::new();
    let mut new_edges: HashMap<(usize,usize),Line>  = HashMap::new();

    self.root = self.root as usize;

    visited.insert(self.root as usize);
    queue.push_back(self.root as usize);

    let mut layer_ending_id = self.root;


    while !queue.is_empty(){
      let c_id = queue.pop_front().unwrap();
      let neighbors = base_info.edges.get(&(c_id as u32)).unwrap();
      let mut c_neighbors: Vec<usize> = Vec::new();//neighbors of current node

      // Get mutable reference to current node to update new_neighbors
      let mut current_node = NodeWrapper { 
        visual_node: None,
        center: (0.0,0.0),
        neighbors: Vec::new(), 
        new_neighbors: 0, // Root is the first node
        force: (0.0, 0.0),
        velocity: (0.0, 0.0),
      };

      for (neighbor_id, weight_str) in neighbors {
        c_neighbors.push((*neighbor_id) as usize);
        if !visited.contains(&(*neighbor_id as usize)){
          queue.push_back(*neighbor_id as usize);
          visited.insert(*neighbor_id as usize);
          // Increment new_neighbors for the current active node
          current_node.new_neighbors += 1;
        }
        new_edges.insert((c_id, (*neighbor_id) as usize), Line::new(LineState::Novisual,
           Point::new(0.0,0.0),
           Point::new(0.0,0.0),
           ""));
      }

      current_node.neighbors = c_neighbors;
      new_nodes.insert(c_id as usize, current_node);

      if c_id == (layer_ending_id as usize) {
        if let Some(last_id) = queue.back() {
          layer_ending_id = *last_id;
        }
      }
    }
    self.nodes = new_nodes;
    self.edges = new_edges;
    self
  } 

  pub fn bigbang_base(&mut self, weight_strings: &HashMap<u32, HashMap<u32, String>>) -> &Self{
    let self_nodes: *mut HashMap<usize,NodeWrapper> = &mut self.nodes;
    
  // Initialize root node and get global parameters
    let root_node = unsafe { (*self_nodes).get_mut(&self.root).unwrap() };
    root_node.center = (self.initial_node_position.0 as f32, self.initial_node_position.1 as f32);
    root_node.visual_node = Some(
      Node::new(CS::Circle(DIAMETER as f32),
        self.node_labels[self.root].as_str(),
        root_node.center.0,
        root_node.center.1,
        NodeColor::Default)
    );
    let global_center = root_node.center;
    println!("[bigbang] root={} center={:?} new_neighbors={}", self.root, global_center, root_node.new_neighbors);
    
    // Setup BFS queue and visited set
    let mut queue: VecDeque<(usize, f32)> = VecDeque::new();
    let mut visited: HashSet<usize> = HashSet::new();
    visited.insert(self.root);
    
    // Setup root's direct neighbors
    self.setup_root_neighbors(self_nodes, global_center, &mut queue, weight_strings);

    let mut position: (f32,f32);
    while let Some((current_id, c_arch_radians)) = queue.pop_front() {
      let current_node = unsafe { (*self_nodes).get_mut(&current_id).unwrap() };
      println!("[bigbang] pop: id={} arch_rad={} new_neighbors={} center={:?}", current_id, c_arch_radians, current_node.new_neighbors, current_node.center);
      
      if current_node.new_neighbors == 0 {
        continue;
      }

      let (neighbor_angles, initial_position) = self.calculate_neighbor_positioning(
        current_node, c_arch_radians, global_center
      );

      let current_node = unsafe { (*self_nodes).get_mut(&current_id).unwrap() };
      position = initial_position;
      
      for &neighbor_id in &current_node.neighbors.clone() {
        if current_id == neighbor_id {
          current_node.visual_node.as_mut().unwrap().color_node(NodeColor::Blue);
          current_node.visual_node.as_mut().unwrap().weight_node(weight_strings.get(&(current_id as u32))
            .unwrap().get(&(neighbor_id as u32)).unwrap().as_str());
          continue;
        }

        let c_neighbor_node = unsafe { (*self_nodes).get_mut(&neighbor_id).unwrap() };
        if !visited.contains(&neighbor_id) {
          visited.insert(neighbor_id);
          queue.push_back((neighbor_id, neighbor_angles));
          create_visual_node_at_position!(self, c_neighbor_node, neighbor_id, position);
        }

        let weight = weight_strings.get(&(current_id as u32)).and_then(|m| m.get(&(neighbor_id as u32)));
        create_edge_between_nodes!(self, current_id, neighbor_id, current_node.center, c_neighbor_node.center,
           weight.unwrap().as_str()
        );
        position = rotate_around!(global_center, position, -neighbor_angles);
      }

    }
    
       self
  }

  fn setup_root_neighbors(&mut self, self_nodes: *mut HashMap<usize,NodeWrapper>, 
                         global_center: (f32, f32), queue: &mut VecDeque<(usize, f32)>,
                          weight_strings: &HashMap<u32, HashMap<u32, String>>) {
    let root_node = unsafe { (*self_nodes).get_mut(&self.root).unwrap() };
    
    let mut c_rotation = 2.0 * PI / root_node.new_neighbors as f32;
    c_rotation = if c_rotation > PI { 3.0 * PI / 4.0 } else { c_rotation };
    let distance_from_root = distance_center!(2.0 * PI, 0.0);
    
    let mut position = (global_center.0 + distance_from_root, global_center.1);
    let mut neighbor_id_ptr = root_node.neighbors.as_ptr();

    for i in 0..root_node.neighbors.len() {
      let neighbor_id = unsafe { *neighbor_id_ptr };
      
      if self.root == neighbor_id {
        unsafe { neighbor_id_ptr = neighbor_id_ptr.add(1) };
        root_node.visual_node.as_mut().unwrap().color_node(NodeColor::Blue);
        root_node.visual_node.as_mut().unwrap().weight_node(weight_strings.get(&(self.root as u32))
          .unwrap().get(&(neighbor_id as u32)).unwrap().as_str());
        continue;
      }

      position = rotate_around!(global_center, position, i as f32 * c_rotation);
      let c_node = unsafe { (*self_nodes).get_mut(&neighbor_id).unwrap() };

      queue.push_back((neighbor_id, c_rotation));
      println!("[bigbang root] neighbor {} pos={:?} rotation={}", neighbor_id, position, c_rotation);
      create_visual_node_at_position!(self, c_node, neighbor_id, position);
      let w = weight_strings.get(&(self.root as u32)).and_then(|m| m.get(&(neighbor_id as u32))); 
      println!("[bigbang root] create edge root->{} weight={:?}", neighbor_id, w);
      create_edge_between_nodes!(self, self.root, neighbor_id, root_node.center, c_node.center,
      w.unwrap().as_str());

      unsafe { neighbor_id_ptr = neighbor_id_ptr.add(1) };
    }
  }


  fn calculate_neighbor_positioning(&self, current_node: &NodeWrapper, c_arch_radians: f32, 
                                  global_center: (f32, f32)) -> (f32, (f32, f32)) {
    let neighbor_angles = c_arch_radians / current_node.new_neighbors as f32;
    let angle_root_parent = angle_between_points!(global_center, current_node.center);
    let center_global_distance = distance_between_points!(current_node.center, global_center);
    
    let initial_additional_rotation = {
      let mut half_neighbors = current_node.new_neighbors / 2;
      if current_node.new_neighbors % 2 == 0 {
        half_neighbors -= 1;
        neighbor_angles / 2.0 + half_neighbors as f32 * neighbor_angles
      } else {
        half_neighbors as f32 * neighbor_angles
      }
    };

    let distance_from_root = distance_center!(c_arch_radians, center_global_distance);
    let mut position = (global_center.0 + distance_from_root, global_center.1);
    position = rotate_around!(global_center, position, angle_root_parent + initial_additional_rotation);
    
    (neighbor_angles, position)
  }
  
  // visual node and edge helpers replaced by macros above

  fn add_node(&mut self, key: usize, node: NodeWrapper){
    self.nodes.insert(key, node);
  }

  //fn add_edge(&mut self, key: (usize,usize), )
}
