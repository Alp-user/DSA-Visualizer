use crate::tree::{Point, Line};
use crate::{angle_between_points, distance_between_points, tree};
use crate::json_deserialize;
use crate::hashgrid::{HashGrid};
use crate::dsas::{Graph, NodeWrapper, DIAGONAL, DIAMETER, SPACE_NODES};
use core::hash;
use std::collections::HashMap;

const SPRING_CONSTANT: f32 = 10e2;
const COULOMB_CONSTANT: f32 = 10.0;
const NODE_MASS: f32 = 3.0;
const ELEMENTARY_CHARGE: f32 = 1.0;
const DAMPING_CONSTANT: f32 = 1000.0;
const RESTING_LENGTH: f32 = 10.0;
const GRID_SPACE: u16  = 5;

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
  spring_constant: f32,
  coulomb_constant: f32,
  mass_constant: f32,
  damping_constant: f32,
  resting_length: f32,
  pub graphs: Vec<Graph<'a>>,
}

pub struct GraphDrawBuilder<'a>{
  viewport: Option<((i32,i32), (i32,i32))>,//top left, bottom right
  grid_spacing: Option<u16>,
  root: Option<&'a json_deserialize::Root>,
  initial_algorithm: Option<Algorithm>,
  listener_id: Option<usize>, //which listener to draw
  spring_constant: Option<f32>,
  coulomb_constant: Option<f32>,
  damping_constant: Option<f32>,
  mass_constant: Option<f32>,
  resting_length: Option<f32>,
}
  
impl<'a> GraphDraw<'a>{
  pub fn simulation_step(&mut self) -> &mut Self{
    let graph: *mut Graph<'a> =  &mut self.graphs[self.listener_id];
    let graph_nodes = unsafe { &mut (*graph).nodes };
    let mut graph_edges = unsafe { &mut (*graph).edges };
    let diameter = DIAMETER as f32;
    let k_spring = self.spring_constant;
    let mut k_coulomb = self.coulomb_constant;
    let r_length = self.resting_length;
    let mass = self.mass_constant;
    let damping = self.damping_constant; // Damping coefficient (tune as needed)
    let dt = 0.001; // Time step (tune as needed)
    let overlap_repulsion = k_coulomb * 100.0; // Stronger repulsion for overlap


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
          c_spring_force.1 += sforce * angle.sin();
          neighbor_force.0 -= sforce * angle.cos();//same as adding pi
          neighbor_force.1 -= sforce * angle.sin();
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
        let dist = distance_between_points!(node.center, other_node.center);
        let angle = angle_between_points!(node.center, other_node.center) + std::f32::consts::PI; // Repulsive force
        k_coulomb = if dist < DIAGONAL * 2.0 + SPACE_NODES as f32 { k_coulomb * 3.0 } else { k_coulomb };

        let eforce = k_coulomb * (ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / (dist * dist);
        electric_force.0 += eforce * angle.cos();
        electric_force.1 += eforce * angle.sin();
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
      let force = forces.get(id).unwrap();
      let acceleration = (force.0 / mass, force.1 / mass);
      node.velocity.0 += acceleration.0 * dt;
      node.velocity.1 += acceleration.1 * dt;
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

    self
  }
  
}

impl<'a> GraphDrawBuilder<'a>{
  pub fn new() -> Self{
    GraphDrawBuilder { viewport: None,
      grid_spacing: None,
      root: None,
      initial_algorithm: None,
      listener_id: None,
      spring_constant: None,
      coulomb_constant: None,
      mass_constant: None,
      resting_length: None,
      damping_constant: None
    }
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
      spring_constant: self.spring_constant.unwrap_or(SPRING_CONSTANT),
      coulomb_constant: self.coulomb_constant.unwrap_or(COULOMB_CONSTANT),
      mass_constant: self.mass_constant.unwrap_or(NODE_MASS),
      damping_constant: self.damping_constant.unwrap_or(DAMPING_CONSTANT),
      resting_length: self.resting_length.unwrap_or(RESTING_LENGTH),
      graphs: Vec::new(),
    };

    new_graph_draw.grid = HashGrid::new(new_graph_draw.viewport, new_graph_draw.grid_spacing);
    Ok(new_graph_draw)
  }
}









