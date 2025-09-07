use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Root{
  pub bases: Vec<BaseInfo>,
  pub diffs: Vec<Vec<DiffInfo>>,
  pub nodes: Vec<String>,//node id is the index
  pub total_listeners: u8,
}

#[derive(Clone, Default, Debug)]
pub struct BaseInfo{
  pub edges: HashMap<u32, HashMap<u32, String>>,
  pub root_id: u32,
  note: String,
}

#[derive(Debug, Default)]
pub struct DiffInfo{
  pub added_edges: Vec<Edge>,
  pub removed_edges: Vec<Edge>,
  pub added_nodes: Vec<Node>,
  pub removed_nodes: Vec<Node>,
  pub weights_changed: Vec<Edge>,
  pub labels_changed: Vec<Node>,
  pub note: String,
  pub root_id: u32,
}

#[derive(Debug,Default)]
pub struct Edge{
  pub from_id: u32,
  pub to_id: u32,
  pub weight: String,
}

#[derive(Debug, Default, Clone)]
pub struct Node{
  pub id: u32,
  pub label: String,
}

pub fn deserialize_json(path: &str) -> Result<Root,Box<dyn Error>>{
  let file = File::open(path)?;
  let reader = BufReader::new(file);

  let json_raw: Value = serde_json::from_reader(reader)?;
  let total_listeners = json_raw.get("specifiers")
    .unwrap()
    .as_object()
    .unwrap()
    .get("total_listeners")
    .unwrap()
    .as_u64()
    .unwrap() as u8;

  let mut bases = Vec::with_capacity(total_listeners as usize);
  let mut diffs: Vec<Vec<DiffInfo>> = Vec::with_capacity(total_listeners as usize);

  bases.resize(total_listeners as usize, BaseInfo::default());
  diffs.resize_with(total_listeners as usize, Vec::new);

  let mut root = Root{
    bases,
    diffs,
    nodes: Vec::new(),
    total_listeners,
  };

  destructure_base(&mut root, &json_raw);

  Ok(root)
}

pub fn destructure_base(root: &mut Root, raw_json: &Value){

  raw_json.get("bases")
    .unwrap()
    .as_object()
    .unwrap()
    .iter()
    .for_each(|(key, value)|{
      let listener_id: u8 =  (&key[1..]).parse().expect("Digit could not be parsed to u8");
      root.bases[listener_id as usize] = base_info_helper(value);
    });

  raw_json.get("diffs")
    .unwrap()
    .as_object()
    .unwrap()
    .iter()
    .for_each(|(key, value)|{
      let listener_id: u8 =  (&key[1..]).parse().expect("Digit could not be parsed to u8");
      root.diffs[listener_id as usize] = diff_info_helper(value);
    });

  root.nodes = node_info_helper(raw_json.get("nodes").unwrap());
}

pub fn node_info_helper(node_value: &Value) -> Vec<String>{
  let array_unwrapped = node_value.as_array().unwrap();
  let mut new_array = Vec::with_capacity(array_unwrapped.len());
  new_array.resize(array_unwrapped.len(),String::default());
  for node_obj in array_unwrapped{
    let node_map = node_obj.as_object().unwrap();
    let id_index: u32 = node_map.get("id").unwrap().as_u64().unwrap() as u32;
    new_array[id_index as usize] = node_map.get("label").unwrap().as_str().unwrap().to_string();
  }
  new_array
}

pub fn diff_info_helper(diff_value: &Value) -> Vec<DiffInfo>{
  let diff_array = diff_value.as_array().unwrap();
  let mut diff_infos = Vec::new();
  
  for diff_obj in diff_array {
    let obj = diff_obj.as_object().unwrap();
    
    let mut added_edges = Vec::new();
    if let Some(added_edges_value) = obj.get("added_edges") {
      for edge_value in added_edges_value.as_array().unwrap() {
        let edge_obj = edge_value.as_object().unwrap();
        added_edges.push(Edge {
          from_id: edge_obj.get("from_id").unwrap().as_u64().unwrap() as u32,
          to_id: edge_obj.get("to_id").unwrap().as_u64().unwrap() as u32,
          weight: edge_obj.get("weight").unwrap().as_str().unwrap().to_string(),
        });
      }
    }
    
    let mut removed_edges = Vec::new();
    if let Some(removed_edges_value) = obj.get("removed_edges") {
      for edge_value in removed_edges_value.as_array().unwrap() {
        let edge_obj = edge_value.as_object().unwrap();
        removed_edges.push(Edge {
          from_id: edge_obj.get("from_id").unwrap().as_u64().unwrap() as u32,
          to_id: edge_obj.get("to_id").unwrap().as_u64().unwrap() as u32,
          weight: edge_obj.get("weight").unwrap().as_str().unwrap().to_string(),
        });
      }
    }
    
    let mut added_nodes = Vec::new();
    if let Some(added_nodes_value) = obj.get("added_nodes") {
      for node_value in added_nodes_value.as_array().unwrap() {
        let node_obj = node_value.as_object().unwrap();
        added_nodes.push(Node {
          id: node_obj.get("id").unwrap().as_u64().unwrap() as u32,
          label: node_obj.get("label").map(|v| v.as_str().unwrap().to_string()).unwrap_or_default(),
        });
      }
    }
    
    let mut removed_nodes = Vec::new();
    if let Some(removed_nodes_value) = obj.get("removed_nodes") {
      for node_value in removed_nodes_value.as_array().unwrap() {
        let node_obj = node_value.as_object().unwrap();
        removed_nodes.push(Node {
          id: node_obj.get("id").unwrap().as_u64().unwrap() as u32,
          label: node_obj.get("label").map(|v| v.as_str().unwrap().to_string()).unwrap_or_default(),
        });
      }
    }
    
    let mut weights_changed = Vec::new();
    if let Some(weights_changed_value) = obj.get("weights_changed") {
      for edge_value in weights_changed_value.as_array().unwrap() {
        let edge_obj = edge_value.as_object().unwrap();
        weights_changed.push(Edge {
          from_id: edge_obj.get("from_id").unwrap().as_u64().unwrap() as u32,
          to_id: edge_obj.get("to_id").unwrap().as_u64().unwrap() as u32,
          weight: edge_obj.get("label").unwrap().as_str().unwrap().to_string(),
        });
      }
    }
    
    let mut labels_changed = Vec::new();
    if let Some(labels_changed_value) = obj.get("labels_changed") {
      for node_value in labels_changed_value.as_array().unwrap() {
        let node_obj = node_value.as_object().unwrap();
        labels_changed.push(Node {
          id: node_obj.get("id").unwrap().as_u64().unwrap() as u32,
          label: node_obj.get("label").unwrap().as_str().unwrap().to_string(),
        });
      }
    }
    
    let note = obj.get("note").unwrap().as_str().unwrap().to_string();
    let root_id = obj.get("root_id").unwrap().as_u64().unwrap() as u32;
    
    diff_infos.push(DiffInfo {
      added_edges,
      removed_edges,
      added_nodes,
      removed_nodes,
      weights_changed,
      labels_changed,
      note,
      root_id,
    });
  }
  
  diff_infos
}

pub fn base_info_helper(base_value: &Value) -> BaseInfo {
  let mut new_base_info = BaseInfo::default();
  let unwrapped_diffobj = base_value.as_object().unwrap();
  unwrapped_diffobj.iter()
    .for_each(|(key, value)|{
      let id_or_text:Option<u32> = key.parse().ok();
    match id_or_text{
        Some(from_id) => {
          new_base_info.edges.insert(from_id, 
            base_info_edge_former(value));
        }
        None => {
          match key.as_str() {
            "root" => {
              new_base_info.root_id = unwrapped_diffobj.get("root").unwrap().as_u64().unwrap() as u32;
            }
            "note" => {
              new_base_info.note = unwrapped_diffobj.get("note").unwrap().as_str().unwrap().to_string();
            }
            _ => {
              panic!("unexpected key");
            }
          }
        }
      }

    });
  new_base_info
}

pub fn base_info_edge_former(base_value: &Value) -> HashMap<u32, String>{
  let mut edge_connections: HashMap<u32, String> = HashMap::default();
  let to_array = base_value.as_array().unwrap();
  for objs in to_array {
    let unwrapped_obj = objs.as_object().unwrap();
    for (to_id_str, weight_wrapped) in unwrapped_obj{
      let to_id:u32 = to_id_str.parse().unwrap();
      let weight = String::from(weight_wrapped.as_str().unwrap());
      edge_connections.insert(to_id, weight);
    }
  }
  edge_connections
}




