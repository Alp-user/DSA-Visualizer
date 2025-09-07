use crate::{c_side::CircleSquare, tree::{NodeColor, LineState, Node, Point, CS, Line }};
use std::{cmp, collections::{HashMap, HashSet, VecDeque}, hash::Hash};
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

#[macro_export]
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

#[macro_export]
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
      // println!("---- EDGE ---- from={} to={} angle={:.6} from_pos=({:.3},{:.3}) to_pos=({:.3},{:.3}) weight=\"{}\"",
        // $from_id, $to_id, angle, $from_pos.0, $from_pos.1, $to_pos.0, $to_pos.1, $weight_str);
      if let Some(c_edge) = $graph.edges.get_mut(&($from_id, $to_id)) {
        *c_edge = Line::new(LineState::StartToEnd(0),
          Point::new($from_pos.0 + DIAMETER as f32 * angle.cos(), $from_pos.1 - DIAMETER as f32 * angle.sin()),
          Point::new($to_pos.0 - DIAMETER as f32 * angle.cos(), $to_pos.1 + DIAMETER as f32 * angle.sin()),
          $weight_str);
        // println!("    -> edge endpoints set: from=({:.3},{:.3}) to=({:.3},{:.3})",
          // $from_pos.0 + DIAMETER as f32 * angle.cos(), $from_pos.1 - DIAMETER as f32 * angle.sin(),
          // $to_pos.0 - DIAMETER as f32 * angle.cos(), $to_pos.1 + DIAMETER as f32 * angle.sin());
      }
    }
  };
}
pub const DIAMETER: i32 = 40;
pub const SPACE_NODES: i32 = 70;
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
  weight_strings: &'a HashMap<u32, HashMap<u32, String>>,
  pub nodes: HashMap<usize,NodeWrapper>,
  pub edges: HashMap<(usize,usize), Line>,//index of first and then second node
  pub root: usize,
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

impl NodeWrapper{
  pub fn new(center: (f32,f32), text: &str, color: NodeColor) -> Self{
    NodeWrapper { 
      visual_node: Some(Node::new(CS::Circle(DIAMETER as f32), text, center.0, center.1, color)),
      // visual_node: None,
      center,
      neighbors: Vec::new(), 
      new_neighbors: 0, // Root is the first node
      force: (0.0, 0.0),
      velocity: (0.0, 0.0),
    }
  }
}

impl<'a> Graph <'a>{
  pub fn new(node_labels: &'a Vec<String>, weight_strings: &'a HashMap<u32, HashMap<u32, String>>, initial_node_position: (i32,i32), root: usize) -> Self{
    Graph { 
      node_labels,
      weight_strings,
      nodes: HashMap::new(),
      edges: HashMap::new(),
      root,
      note: String::from(""),
      initial_node_position,
    }
  }
  
  pub fn add_edge(&mut self, from_id: usize, to_id: usize, weight_str: &str){
    let first_node = self.nodes.get(&from_id).unwrap();
    let second_node = self.nodes.get(&to_id).unwrap();
    let angle = angle_between_points!(first_node.center, second_node.center);
    self.edges.insert((from_id,to_id), Line::new(LineState::StartToEnd(0),
      Point::new(first_node.center.0 +  DIAMETER as f32 * angle.cos(), 
        first_node.center.1 - DIAMETER as f32 * angle.sin()),
        Point::new(second_node.center.0 - DIAMETER as f32 * angle.cos(), 
          second_node.center.1 + DIAMETER as f32 * angle.sin()),
          weight_str,
        )
      );
  }
  pub fn node_exists(&self, id: usize) -> bool{
    self.nodes.contains_key(&id)
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
    
    // Setup BFS queue and visited set
    let mut queue: VecDeque<(usize, f32)> = VecDeque::new();
    let mut visited: HashSet<usize> = HashSet::new();
    visited.insert(self.root);
    
    // Setup root's direct neighbors
    self.setup_root_neighbors(self_nodes, global_center, &mut queue, weight_strings, &mut visited);

    let mut position: (f32,f32);
    while let Some((current_id, c_arch_radians)) = queue.pop_front() {
      let current_node = unsafe { (*self_nodes).get_mut(&current_id).unwrap() };
      
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
          position = rotate_around!(global_center, position, -neighbor_angles);
        }

        let weight = weight_strings.get(&(current_id as u32)).and_then(|m| m.get(&(neighbor_id as u32)));
        create_edge_between_nodes!(self, current_id, neighbor_id, current_node.center, c_neighbor_node.center,
           weight.unwrap().as_str()
        );
      }
      // println!(" queue {:?}", queue);

    }
    
       self
  }

  fn setup_root_neighbors(&mut self, self_nodes: *mut HashMap<usize,NodeWrapper>, 
                         global_center: (f32, f32), queue: &mut VecDeque<(usize, f32)>,
                          weight_strings: &HashMap<u32, HashMap<u32, String>>,
                        visited: &mut HashSet<usize>) {
  let root_node = unsafe { (*self_nodes).get_mut(&self.root).unwrap() };
    
    let mut c_rotation = 2.0 * PI / root_node.new_neighbors as f32;
    c_rotation = if c_rotation > PI { 3.0 * PI / 4.0 } else { c_rotation };
    let distance_from_root = distance_center!(2.0 * PI, 0.0);
    
    let mut position = (global_center.0 + distance_from_root, global_center.1);
    let mut neighbor_id_ptr = root_node.neighbors.as_ptr();

    // println!("[setup_root_neighbors] root_id={} neighbors_order={:?} c_rotation={:.6} distance_from_root={:.4} start_pos=({:.4},{:.4})",
    //   self.root, root_node.neighbors, c_rotation, distance_from_root, position.0, position.1);

    for i in 0..root_node.neighbors.len() {
      let neighbor_id = unsafe { *neighbor_id_ptr };
      
      if self.root == neighbor_id {
        unsafe { neighbor_id_ptr = neighbor_id_ptr.add(1) };
        root_node.visual_node.as_mut().unwrap().color_node(NodeColor::Blue);
        root_node.visual_node.as_mut().unwrap().weight_node(weight_strings.get(&(self.root as u32))
          .unwrap().get(&(neighbor_id as u32)).unwrap().as_str());
        continue;
      }

      let c_node = unsafe { (*self_nodes).get_mut(&neighbor_id).unwrap() };

      queue.push_back((neighbor_id, c_rotation));
      // println!("[setup_root_neighbors] queued neighbor id={} arch_rad={:.6}", neighbor_id, c_rotation);
      // Detailed initial placement trace for the first child
      if i == 0 {
        let start_pos = (global_center.0 + distance_from_root, global_center.1);
        // println!("\n*** ROOT-FIRST-CHILD TRACE (root_id={} -> neighbor_id={}) ***", self.root, neighbor_id);
        // println!("  distance_from_root = {:.6}", distance_from_root);
        // println!("  unrotated_start_pos = ({:.6},{:.6})", start_pos.0, start_pos.1);
        // println!("  rotation_about_global (c_rotation) = {:.6} rad (~{:.1} deg)", c_rotation, c_rotation.to_degrees());
        // show the position after the rotate_around call that will be used
        let rotated = rotate_around!(global_center, start_pos, c_rotation);
        // println!("  start_pos AFTER rotate_about_global = ({:.6},{:.6})\n", rotated.0, rotated.1);
      }
  visited.insert(neighbor_id);
      create_visual_node_at_position!(self, c_node, neighbor_id, position);
      let w = weight_strings.get(&(self.root as u32)).and_then(|m| m.get(&(neighbor_id as u32))); 
      create_edge_between_nodes!(self, self.root, neighbor_id, root_node.center, c_node.center,
      w.unwrap().as_str());

      position = rotate_around!(global_center, position, c_rotation);
      unsafe { neighbor_id_ptr = neighbor_id_ptr.add(1) };
    }
  }


  fn calculate_neighbor_positioning(&self, current_node: &NodeWrapper, c_arch_radians: f32, 
                                  global_center: (f32, f32)) -> (f32, (f32, f32)) {
    let neighbor_angles = c_arch_radians / current_node.new_neighbors as f32;
    let angle_root_parent = angle_between_points!(global_center, current_node.center);
    let center_global_distance = distance_between_points!(current_node.center, global_center);
    
      // compute initial additional rotation and show intermediate values
      let initial_additional_rotation = {
        let mut half_neighbors = current_node.new_neighbors / 2;
        let computed: f32;
        if current_node.new_neighbors % 2 == 0 {
          half_neighbors -= 1;
          computed = neighbor_angles / 2.0 + half_neighbors as f32 * neighbor_angles;
          // println!("[initial_additional_rotation] node_center=({:.4},{:.4}) new_neighbors={} (even) half_neighbors(after-1)={} neighbour_angles={:.6} computed_additional_rotation={:.6}",
          //   current_node.center.0, current_node.center.1, current_node.new_neighbors, half_neighbors, neighbor_angles, computed);
        } else {
          computed = half_neighbors as f32 * neighbor_angles;
          // println!("[initial_additional_rotation] node_center=({:.4},{:.4}) new_neighbors={} (odd) half_neighbors={} neighbour_angles={:.6} computed_additional_rotation={:.6}",
          //   current_node.center.0, current_node.center.1, current_node.new_neighbors, half_neighbors, neighbor_angles, computed);
        }
        computed
      };

    let distance_from_root = distance_center!(c_arch_radians, center_global_distance);
    let start_pos_unrotated = (global_center.0 + distance_from_root, global_center.1);
    let rotation = angle_root_parent + initial_additional_rotation;
    let mut position = rotate_around!(global_center, start_pos_unrotated, rotation);


    (neighbor_angles, position)
  }
  
  // visual node and edge helpers replaced by macros above
  pub fn remove_edges_of_node(&mut self, id: usize) {
    self.edges.iter_mut().
      for_each(|((from_id, to_id), edge)|{
        if from_id == &id || to_id == &id {
          edge.remove_line();
        }
    });
    self.edges.retain(|&(from_id, to_id), _| from_id != id && to_id != id);
  }
  
  pub fn remove_node(&mut self, id: usize) {
    let c_node = self.nodes.get_mut(&id).unwrap();
    c_node.visual_node.as_mut().unwrap().remove_node();
    self.remove_edges_of_node(id);
    self.nodes.remove(&id);
  }
  
  pub fn remove_edge(&mut self, from_id: usize, to_id: usize) {
    let c_edge = self.edges.get_mut(&(from_id, to_id)).unwrap();
    c_edge.remove_line();
    self.edges.remove(&(from_id, to_id));
  }

  fn add_node(&mut self, key: usize, node: NodeWrapper){
    self.nodes.insert(key, node);
  }
  
  pub fn clean_graph(&mut self){
    for node in self.nodes.values_mut(){
      if let Some(c_node) = node.visual_node.as_mut() {
        c_node.remove_node();
      }
    }
    for edge in self.edges.values_mut(){
      edge.remove_line();
    }
    self.nodes.clear();
    self.edges.clear();
  }

  //fn add_edge(&mut self, key: (usize,usize), )
}
