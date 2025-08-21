use crate::tree::{Point};
use crate::tree;
use crate::json_deserialize;
use crate::hashgrid::{HashGrid};

const SPRING_CONSTANT: f32 = 10e5;
const COULOMB_CONSTANT: f32 = 9e9;
const NODE_MASS: f32 = 9.1e-31;
const ELEMENTARY_CHARGE: f32 = 1.6e-19;
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
  listener_id: u8, //which listener to draw
  spring_constant: f32,
  coulomb_constant: f32,
  mass_constant: f32,
}

pub struct GraphDrawBuilder<'a>{
  viewport: Option<((i32,i32), (i32,i32))>,//top left, bottom right
  grid_spacing: Option<u16>,
  root: Option<&'a json_deserialize::Root>,
  initial_algorithm: Option<Algorithm>,
  listener_id: Option<u8>, //which listener to draw
  spring_constant: Option<f32>,
  coulomb_constant: Option<f32>,
  mass_constant: Option<f32>,
}

impl<'a> GraphDraw<'a>{

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
      mass_constant: None 
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

  pub fn listener_id(mut self, id: u8) -> Self {
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

  pub fn mass_constant(mut self, constant: f32) -> Self {
    self.mass_constant = Some(constant);
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
    };

    new_graph_draw.grid = HashGrid::new(new_graph_draw.viewport, new_graph_draw.grid_spacing);
    Ok(new_graph_draw)
  }
}









