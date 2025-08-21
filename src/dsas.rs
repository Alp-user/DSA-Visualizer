use crate::tree::{Node,Line};
use std::{cmp, collections::{HashMap, HashSet, VecDeque}};
use crate::json_deserialize;

const RADIUS: u32 = 50;

struct Graph<'a>{
  node_labels: &'a Vec<String>,
  nodes: HashMap<usize,NodeWrapper>,
  edges: HashMap<(usize,usize), Line>,//index of first and then second node
  note: String,
  initial_node_position: (i32,i32),
}

struct NodeWrapper{
  visual_node: Option<Node>,
  center: (i32,i32),
  neighbors: Vec<usize>,
}

impl<'a> Graph <'a>{
  pub fn new(node_labels: &'a Vec<String>, initial_node_position: (i32,i32)) -> Self{
    Graph { node_labels,
      nodes: HashMap::new(),
      edges: HashMap::new(),
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

    let mut position = self.initial_node_position;
    let mut root_id = base_info.root_id;

    visited.insert(root_id as usize);
    queue.push_back(root_id as usize);

    let pixels_from_center = 0;

    while !queue.is_empty(){
      let mut current_neighbors = Vec::new();
      for (current_id, neighbors) in &base_info.edges {
        let pixels_from_center = cmp::max(pixels_from_center + 1,
          ((neighbors.len() as f32)/8.0).ceil() as i32);

        for (neighbor_id, weight_str) in neighbors {
          current_neighbors.push(neighbor_id);
          if !visited.contains(&(*neighbor_id as usize)){
            queue.push_back(*neighbor_id as usize);
            visited.insert(*neighbor_id as usize);
          }
          new_edges.insert((current_id, neighbor_id) )
        }
      }
    }
    self
  }

  fn add_node(&mut self, key: usize, node: NodeWrapper){
    self.nodes.insert(key, node);
  }

  fn add_edge(&mut self, key: (usize,usize), )
}
