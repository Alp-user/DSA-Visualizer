use crate::tree::{CS,Line, NodeColor, Point};
use crate::{angle_between_points, distance_between_points, tree, rotate_around, create_visual_node_at_position};
use crate::json_deserialize;
use crate::json_deserialize::{Node, Edge};
use crate::hashgrid::{HashGrid};
use crate::dsas::{Graph, NodeWrapper, DIAGONAL, DIAMETER, SPACE_NODES};
use core::hash;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

// Tuned constants for more stable and visually pleasing force-directed graph simulation
const SPRING_CONSTANT: f32 = 0.35;           // Softer springs for less oscillation
const COULOMB_CONSTANT: f32 = 1000.0;        // Lower repulsion for less jitter
const NODE_MASS: f32 = 20.0;                  // Slightly heavier nodes for smoother movement
const ELEMENTARY_CHARGE: f32 = 1.0;          // Keep as is
const DAMPING_CONSTANT: f32 = 8.0;           // Higher damping to quickly reduce velocity
const EMPTY_SPACE: f32 = 300.0;               // Reasonable spacing for clarity
const RESTING_LENGTH: f32 = DIAGONAL * 2.0 + SPACE_NODES as f32 + EMPTY_SPACE;
const OVERLAP_COULOMB_MULTIPLIER: f32 = 300.0; // Lower to avoid explosive repulsion on overlap
const GRID_SPACE: u16 = 5;                   // Keep as is
const DT: f32 = 0.08;                        // Small time step for stability
const STABLE_VELOCITY: f32 = 3.0;            // Lower threshold for stability
const MAX_COULOMB_FORCE: f32 = 5000.0;       // Clamp to avoid force spikes


#[derive(Debug)]
pub enum Algorithm{
  BigBang,
  Randomized,
}

pub trait Render{
  fn render(viewport: ((i32,i32), (i32,i32)), number: usize);
}

struct PolarNode{
  center: Point,
  visual_node: tree::Node,
}

#[derive(Debug)]
pub struct GraphDraw<'a>{
  viewport: ((i32,i32), (i32,i32)),//top left, bottom right
  grid_spacing: u16,
  grid: HashGrid<i32>,
  root: &'a json_deserialize::Root,
  initial_algorithm: Algorithm,
  listener_id: usize, //which listener to draw
  initial_position: (i32,i32),
  spring_constant: f32,
  coulomb_constant: f32,
  mass_constant: f32,
  damping_constant: f32,
  resting_length: f32,
  pub diff_step: usize,
  pub graphs: Vec<Graph<'a>>,
}

pub struct GraphDrawBuilder<'a>{
  viewport: Option<((i32,i32), (i32,i32))>,//top left, bottom right
  grid_spacing: Option<u16>,
  root: Option<&'a json_deserialize::Root>,
  initial_algorithm: Option<Algorithm>,
  listener_id: Option<usize>, //which listener to draw
  initial_position: Option<(i32,i32)>,
  spring_constant: Option<f32>,
  coulomb_constant: Option<f32>,
  damping_constant: Option<f32>,
  mass_constant: Option<f32>,
  resting_length: Option<f32>,
}
  
impl<'a> GraphDraw<'a>{
  pub fn simulation_step(&mut self) -> bool{
    let graph: *mut Graph<'a> =  &mut self.graphs[self.listener_id];
    let graph_nodes = unsafe { &mut (*graph).nodes };
    let mut graph_edges = unsafe { &mut (*graph).edges };
    let diameter = DIAMETER as f32;
    let k_spring = self.spring_constant;
    let mut k_coulomb = self.coulomb_constant;
    let r_length = self.resting_length;
    let mass = self.mass_constant;
    let damping = self.damping_constant; // Damping coefficient (tune as needed)
    let dt = DT; // Time step (tune as needed)
    let mut stable = true;


    let spring_force_loop = |id: &usize, node: &NodeWrapper, hash_force: &mut HashMap<usize, (f32, f32)>| {
      let mut c_spring_force = (0.0, 0.0);
      for &neighbor_id in &node.neighbors {
        if neighbor_id == (*id) { continue; }
        let mut neighbor_force = (0.0, 0.0);
        if let Some(neighbor) = graph_nodes.get(&neighbor_id) {
          let dist = distance_between_points!(node.center, neighbor.center);
          let angle = angle_between_points!(node.center, neighbor.center);
          let sforce = k_spring * (dist - r_length);

          c_spring_force.0 += sforce * angle.cos();
          c_spring_force.1 -= sforce * angle.sin();
          neighbor_force.0 -= sforce * angle.cos();//same as adding pi
          neighbor_force.1 += sforce * angle.sin();
          let neighbor_entry = hash_force.entry(neighbor_id).or_insert((0.0, 0.0));
          neighbor_entry.0 += neighbor_force.0;
          neighbor_entry.1 += neighbor_force.1;
        }
      }

      let c_entry = hash_force.entry(*id).or_insert((0.0, 0.0));
      c_entry.0 += c_spring_force.0;
      c_entry.1 += c_spring_force.1;
    };
    

    let mut electric_force_loop = |id: &usize, node: &NodeWrapper| -> (f32, f32) {
      let mut electric_force = (0.0, 0.0);
      for (other_id, other_node) in graph_nodes.iter() {
        if other_id == id { continue; }
        let mut dist = distance_between_points!(node.center, other_node.center);
        let angle = angle_between_points!(node.center, other_node.center) + std::f32::consts::PI; // Repulsive force direction

        if dist <= 0.0001 { dist = 0.0001; }

        let overlap_multiplier = if dist < DIAMETER as f32 * 2.0 + SPACE_NODES as f32 {
          OVERLAP_COULOMB_MULTIPLIER
        }  else {
          1.0
        };

        let local_k = k_coulomb * overlap_multiplier;
        let mut eforce = local_k * (ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / (dist * dist);
        // clamp to avoid numerical explosions
        if eforce.is_nan() || eforce.is_infinite() { eforce = MAX_COULOMB_FORCE; }
        if eforce > MAX_COULOMB_FORCE { eforce = MAX_COULOMB_FORCE; }

        electric_force.0 += eforce * angle.cos();
        electric_force.1 -= eforce * angle.sin();
      }
      electric_force
    };
    
    let damping_force_loop = |node: &NodeWrapper| -> (f32, f32) {
      (-damping * node.velocity.0, -damping * node.velocity.1)
    };

    let mut forces: HashMap<usize, (f32, f32)> = HashMap::new();
    let mut hash_force: HashMap<usize, (f32, f32)> = HashMap::new();
    for (id, node) in graph_nodes.iter() {
      spring_force_loop(id, node, &mut hash_force);
      let total_electric_force = electric_force_loop(id, node);
      let total_damping_force = damping_force_loop(node);
      let total_force = (
        total_electric_force.0 + total_damping_force.0,
        total_electric_force.1 + total_damping_force.1,
      );
      // let acceleration = (total_force.0 / mass, total_force.1 / mass);
      forces.insert(*id, total_force);
    }

    for (id, (fx, fy)) in hash_force.iter() {
      let total_force = forces.get_mut(id).unwrap();
      total_force.0 += *fx;
      total_force.1 += *fy;
    }
    
    for (id, node) in graph_nodes.iter_mut() {
      if unsafe {*id == (*graph).root } { 
        continue;
      }
      let force = forces.get(id).unwrap();
      let acceleration = (force.0 / mass, force.1 / mass);
      node.velocity.0 += acceleration.0 * dt;
      node.velocity.1 += acceleration.1 * dt;
      let velocity_magnitude = (node.velocity.0.powi(2) + node.velocity.1.powi(2)).sqrt();
      // println!("Velocity magnitude: {}", velocity_magnitude);
      if stable && velocity_magnitude > STABLE_VELOCITY {
        stable = false;
      }
      node.center.0 += node.velocity.0 * dt;
      node.center.1 += node.velocity.1 * dt;
      node.visual_node.as_ref().unwrap().move_node(node.center.0, node.center.1);
    }
    
    for ((from_id, to_id), line) in graph_edges.iter_mut() {
      let first_node_center = graph_nodes.get(&from_id).unwrap().center;
      let second_node_center = graph_nodes.get(&to_id).unwrap().center;
      let angle = angle_between_points!(first_node_center, second_node_center);
      line.override_line(
        Point::new(first_node_center.0 + DIAMETER as f32 * angle.cos(), first_node_center.1 - DIAMETER as f32 * angle.sin()),
        Point::new(second_node_center.0 - DIAMETER as f32 * angle.cos(), second_node_center.1 + DIAMETER as f32 * angle.sin()),
      );
    }
    stable
  }
  
  pub fn forward_diff(&mut self){
    if self.diff_step >= self.root.diffs[self.listener_id].len(){
      return;
    }
    let current_diff: *const json_deserialize::DiffInfo = &self.root.diffs[self.listener_id][self.diff_step];
    let mut nodes_to_place = HashSet::new();
    let old_root_id = self.graphs[self.listener_id].root;
    println!("old root id: {}", old_root_id);
    self.graphs[self.listener_id].root = unsafe {(*current_diff).root_id as usize};
    {
      let old_root_node = self.graphs[self.listener_id].nodes.get_mut(&old_root_id).unwrap();
      old_root_node.visual_node.as_mut().unwrap().color_node(NodeColor::Default);
      let root_node = self.graphs[self.listener_id].nodes.get_mut(&unsafe{(*current_diff).root_id as usize}).unwrap();
      root_node.visual_node.as_mut().unwrap().color_node(NodeColor::Blue);
    }
    println!("current diff: {:?}", unsafe{&(*current_diff)});
    
    for node in unsafe{&(*current_diff).labels_changed} {
      let c_node = self.graphs[self.listener_id].nodes.get_mut(&(node.id as usize)).unwrap();
      let after_str_index = node.label.find("===").unwrap();
      c_node.visual_node.as_mut().unwrap().label_node(&node.label[after_str_index + 3..]);
    }

    for edge in unsafe{&(*current_diff).weights_changed} {
      let c_edge = self.graphs[self.listener_id].edges.get_mut(&(edge.from_id as usize, edge.to_id as usize)).unwrap();
      let after_str_index = edge.weight.find("===").unwrap();
      c_edge.weight_line(&edge.weight[after_str_index + 3..]);
    }

    for node in unsafe{&(*current_diff).added_nodes} {
      let global_root_id = self.graphs[self.listener_id].root;
      let global_root_center = self.graphs[self.listener_id].nodes.get(&global_root_id).unwrap().center;
      self.graphs[self.listener_id].nodes.insert(node.id as usize,
         NodeWrapper{
           visual_node: Some(tree::Node::new(
             CS::Circle(DIAMETER as f32),
             &self.root.nodes[node.id as usize],
             0.0,
             0.0,
             NodeColor::Red,
           )),
           center: global_root_center,
           neighbors: Vec::new(),
           new_neighbors: 0,
           force: (0.0, 0.0),
           velocity: (0.0, 0.0),
         });
      nodes_to_place.insert(node.id as usize);
    }

    for edge in unsafe{&(*current_diff).added_edges}{
      let first_tobe_placed = nodes_to_place.contains(&(edge.from_id as usize));
      let second_tobe_placed = nodes_to_place.contains(&(edge.to_id as usize));
      let global_center = self.graphs[self.listener_id].nodes.get(&self.graphs[self.listener_id].root).unwrap().center.clone();


      let first_center: (f32,f32);
      if first_tobe_placed {
        // first node is newly placed: give it a random position and visual
        let new_center = (rand::random::<f32>() * unsafe{crate::DIMENSIONS.0} ,
        rand::random::<f32>() * unsafe{crate::DIMENSIONS.1});

        let first_node = self.graphs[self.listener_id].nodes.get_mut(&(edge.from_id as usize)).unwrap();
        first_node.center = new_center;
        first_node.visual_node = Some(tree::Node::new(
          CS::Circle(DIAMETER as f32),
          &self.root.nodes[edge.from_id as usize],
          first_node.center.0,
          first_node.center.1,
          NodeColor::Red
        ));
        first_center = new_center;
      }
      else{
        let first_node = self.graphs[self.listener_id].nodes.get_mut(&(edge.from_id as usize)).unwrap();
        first_center = first_node.center.clone();
      }

      if !second_tobe_placed {
        {
          let first_node = self.graphs[self.listener_id].nodes.get_mut(&(edge.from_id as usize)).unwrap();
          first_node.neighbors.push(edge.to_id as usize);
        }
      } 
      else {
        {
          let first_node = self.graphs[self.listener_id].nodes.get_mut(&(edge.from_id as usize)).unwrap();
          first_node.neighbors.push(edge.to_id as usize);
          first_node.new_neighbors += 1;
        }
        let second_node = &mut self.graphs[self.listener_id].nodes.get_mut(&(edge.to_id as usize)).unwrap();
        second_node.center = (first_center.0 + self.resting_length, first_center.1);
        second_node.center = rotate_around!(global_center, second_node.center, rand::random::<f32>() * 2.0 * PI);
        second_node.visual_node = Some(tree::Node::new(
          CS::Circle(DIAMETER as f32),
          &self.root.nodes[edge.to_id as usize],
          second_node.center.0,
          second_node.center.1,
          NodeColor::Red,
        ));
        nodes_to_place.remove(&(edge.to_id as usize));
      }
      self.graphs[self.listener_id].add_edge(edge.from_id as usize, edge.to_id as usize, &edge.weight);
    }

    for edge in unsafe{&(*current_diff).removed_edges} {
      // println!("removed edge: from id: {} to id: {}", edge.from_id, edge.to_id);
      self.graphs[self.listener_id].remove_edge(edge.from_id as usize, edge.to_id as usize);
    }

    for node in unsafe{&(*current_diff).removed_nodes} {
      if node.id as usize == self.graphs[self.listener_id].root {
        // println!("Root node removed, changing root.");
        self.graphs[self.listener_id].root = *self.graphs[self.listener_id].nodes.iter().next().unwrap().0;
      }
      self.graphs[self.listener_id].remove_node(node.id as usize);
    }
    println!("function forward diff ended");
    unsafe {crate::STABLE_HAPPENED = false;}
    self.diff_step += 1;
  }

  pub fn backward_diff(&mut self){
    println!("backward_diff function started");
    if self.diff_step == 0{
      println!("Already at the beginning, cannot go backward");
      return;
    }
    
    // Move to previous diff step
    self.diff_step -= 1;
    
    if self.diff_step >= self.root.diffs[self.listener_id].len(){
      println!("diff_step out of bounds after decrement");
      return;
    }
    
    let current_diff: *const json_deserialize::DiffInfo = &self.root.diffs[self.listener_id][self.diff_step];
    let mut nodes_to_remove = HashSet::new();
    
    println!("current diff (backward): {:?}", unsafe{&(*current_diff)});
    
    // First, add back removed nodes (opposite of removing them)
    for node in unsafe{&(*current_diff).removed_nodes} {
      println!("Adding back removed node: {}", node.id);
      let global_root_id = self.graphs[self.listener_id].root;
      let global_root_center = self.graphs[self.listener_id].nodes.get(&global_root_id).unwrap().center;
      self.graphs[self.listener_id].nodes.insert(node.id as usize,
         NodeWrapper{
           visual_node: Some(tree::Node::new(
             CS::Circle(DIAMETER as f32),
             &self.root.nodes[node.id as usize],
             global_root_center.0,
             global_root_center.1,
             NodeColor::Default,
           )),
           center: global_root_center,
           neighbors: Vec::new(),
           new_neighbors: 0,
           force: (0.0, 0.0),
           velocity: (0.0, 0.0),
         });
    }

    // Add back removed edges (opposite of removing them)
    for edge in unsafe{&(*current_diff).removed_edges} {
      println!("Adding back removed edge: from {} to {}", edge.from_id, edge.to_id);
      self.graphs[self.listener_id].add_edge(edge.from_id as usize, edge.to_id as usize, &edge.weight);
    }

    // Remove added edges (opposite of adding them)
    for edge in unsafe{&(*current_diff).added_edges}{
      println!("Removing added edge: from {} to {}", edge.from_id, edge.to_id);
      self.graphs[self.listener_id].remove_edge(edge.from_id as usize, edge.to_id as usize);
    }

    // Remove added nodes (opposite of adding them)
    for node in unsafe{&(*current_diff).added_nodes} {
      println!("Removing added node: {}", node.id);
      nodes_to_remove.insert(node.id as usize);
      if node.id as usize == self.graphs[self.listener_id].root {
        println!("Root node being removed, changing root.");
        // Find another node to be root
        if let Some((new_root_id, _)) = self.graphs[self.listener_id].nodes.iter().find(|(id, _)| **id != node.id as usize) {
          self.graphs[self.listener_id].root = *new_root_id;
        }
      }
      self.graphs[self.listener_id].remove_node(node.id as usize);
    }

    // Reverse weight changes (use text before "===")
    for edge in unsafe{&(*current_diff).weights_changed} {
      println!("Reversing weight change for edge from {} to {}", edge.from_id, edge.to_id);
      let c_edge = self.graphs[self.listener_id].edges.get_mut(&(edge.from_id as usize, edge.to_id as usize)).unwrap();
      let before_str_index = edge.weight.find("===").unwrap();
      c_edge.weight_line(&edge.weight[..before_str_index]);
    }

    // Reverse label changes (use text before "===")
    for node in unsafe{&(*current_diff).labels_changed} {
      println!("Reversing label change for node: {}", node.id);
      let c_node = self.graphs[self.listener_id].nodes.get_mut(&(node.id as usize)).unwrap();
      let before_str_index = node.label.find("===").unwrap();
      c_node.visual_node.as_mut().unwrap().label_node(&node.label[..before_str_index]);
    }
    
    // Reverse root change
    let new_root_id = self.graphs[self.listener_id].root;
    println!("new root id: {}", new_root_id);
    
    // Determine the previous root (this would be the root from the previous diff or initial state)
    let previous_root_id = if self.diff_step == 0 {
      // If we're back to the beginning, use the original root from base state
      self.root.bases[self.listener_id].root_id as usize
    } else {
      // Otherwise, use the root from the previous diff
      self.root.diffs[self.listener_id][self.diff_step - 1].root_id as usize
    };
    
    println!("previous root id: {}", previous_root_id);
    self.graphs[self.listener_id].root = previous_root_id;
    
    // Update visual colors
    {
      let old_root_node = self.graphs[self.listener_id].nodes.get_mut(&new_root_id).unwrap();
      old_root_node.visual_node.as_mut().unwrap().color_node(NodeColor::Default);
      let root_node = self.graphs[self.listener_id].nodes.get_mut(&previous_root_id).unwrap();
      root_node.visual_node.as_mut().unwrap().color_node(NodeColor::Blue);
    }
    
    println!("function backward_diff ended");
    unsafe {crate::STABLE_HAPPENED = false;}
  }
  
  pub fn add_new_graph(&mut self, listener_id: usize){
    let root_id = self.root.bases[listener_id].root_id as usize;
    let mut new_graph = Graph::new(&self.root.nodes, &self.root.bases[listener_id].edges, self.initial_position, root_id);
    if listener_id == 0 {
      new_graph.build_base(&(self.root.bases[listener_id]));
      new_graph.bigbang_base(&(self.root.bases[listener_id].edges));
    }
    self.graphs.push(new_graph);
  }
  
  pub fn change_listener_id(&mut self, new_id: usize){
    println!("number of graphs registered are {}", self.graphs.len());
    if new_id > self.graphs.len() || new_id == 0 {
      println!("Listener id out of bounds");
      return;
    }
    self.graphs[self.listener_id].clean_graph();
    self.diff_step = 0;
    self.listener_id = new_id - 1;
    self.graphs[self.listener_id].build_base(&(self.root.bases[self.listener_id]));
    self.graphs[self.listener_id].bigbang_base(&(self.root.bases[self.listener_id].edges));
    
    unsafe {crate::STABLE_HAPPENED = false;}
  }
  
}

impl<'a> GraphDrawBuilder<'a>{
  pub fn new() -> Self{
    GraphDrawBuilder { viewport: None,
      grid_spacing: None,
      root: None,
      initial_algorithm: None,
      listener_id: None,
      initial_position: None,
      spring_constant: None,
      coulomb_constant: None,
      mass_constant: None,
      resting_length: None,
      damping_constant: None
    }
  }

  pub fn initial_position(mut self, pos: (i32,i32)) -> Self {
    self.initial_position = Some(pos);
    self
  }

  pub fn viewport(mut self, top_left: (i32,i32), bottom_right: (i32,i32)) -> Self{
    self.viewport = Some((top_left, bottom_right));
    self
  }

  pub fn grid_spacing(mut self, grid_spacing: u16) -> Self{
    self.grid_spacing = Some(grid_spacing);
    self
  }

  pub fn root(mut self, root: &'a json_deserialize::Root) -> Self {
    self.root = Some(root);
    self
  }

  pub fn initial_algorithm(mut self, algorithm: Algorithm) -> Self {
    self.initial_algorithm = Some(algorithm);
    self
  }

  pub fn listener_id(mut self, id: usize) -> Self {
    self.listener_id = Some(id);
    self
  }

  pub fn spring_constant(mut self, constant: f32) -> Self {
    self.spring_constant = Some(constant);
    self
  }

  pub fn coulomb_constant(mut self, constant: f32) -> Self {
    self.coulomb_constant = Some(constant);
    self
  }

  pub fn damping_constant(mut self, constant: f32) -> Self {
    self.damping_constant = Some(constant);
    self
  }

  pub fn mass_constant(mut self, constant: f32) -> Self {
    self.mass_constant = Some(constant);
    self
  }

  pub fn resting_length(mut self, length: f32) -> Self {
    self.resting_length = Some(length);
    self
  }

  pub fn build(self) -> Result<GraphDraw<'a>, &'static str> {
    let root = self.root.ok_or("Root is required")?;
    
    let mut new_graph_draw = GraphDraw {
      viewport: self.viewport.unwrap_or(((0, 0), (800, 600))),
      grid_spacing: self.grid_spacing.unwrap_or(GRID_SPACE),
      grid: HashGrid::default(),
      root,
      initial_algorithm: self.initial_algorithm.unwrap_or(Algorithm::BigBang),
      listener_id: self.listener_id.unwrap_or(0),
      initial_position: self.initial_position.unwrap_or({
        let vp = self.viewport.unwrap_or(((0,0),(800,600)));
        let tl = vp.0;
        let br = vp.1;
        ((tl.0 + br.0)/2, (tl.1 + br.1)/2)
      }),
      spring_constant: self.spring_constant.unwrap_or(SPRING_CONSTANT),
      coulomb_constant: self.coulomb_constant.unwrap_or(COULOMB_CONSTANT),
      mass_constant: self.mass_constant.unwrap_or(NODE_MASS),
      damping_constant: self.damping_constant.unwrap_or(DAMPING_CONSTANT),
      resting_length: self.resting_length.unwrap_or(RESTING_LENGTH),
      diff_step: 0,
      graphs: Vec::new(),
    };

    let viewport = ((new_graph_draw.viewport.0.0 as f32, new_graph_draw.viewport.0.1 as f32),
     (new_graph_draw.viewport.1.0 as f32, new_graph_draw.viewport.1.1 as f32));
    new_graph_draw.grid = HashGrid::new(viewport, new_graph_draw.grid_spacing);
    Ok(new_graph_draw)
  }
}









