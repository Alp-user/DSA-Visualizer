use std::collections::{hash_set, HashSet};
use std::hash::Hash;
use std::thread::current;
use rand::seq::index;
use std::ops::Index;

use crate::{CAMERA_SHIFT, DIMENSIONS};

macro_rules! point_in_screen_space {
    ($self:expr, $point:expr) => {
        $point.0 <= $self.screen_space.1.0 
            && $point.0 >= $self.screen_space.0.0
            && $point.1 <= $self.screen_space.1.1 
            && $point.1 >= $self.screen_space.0.1
    };
}

const FIXED_ARRAY_SIZE: usize = 3;
#[derive(Debug)]
struct Array3e<T>{
  pub data: [T; FIXED_ARRAY_SIZE],
  pub size: usize,
}

impl<T: Default + Copy + PartialEq> Array3e<T>{
  pub fn new() -> Self {
    Array3e {
      data: [T::default(); FIXED_ARRAY_SIZE],
      size: 0,
    }
  }
  pub fn insert(&mut self, element: T) -> Result<(), ()>{
    if self.size == FIXED_ARRAY_SIZE {
      return Err(())
    } 
    self.data[self.size] = element;
    self.size += 1;
    Ok(())
  }

  pub fn remove(&mut self, index: usize) -> Result<(), ()>{
    if index >= self.size{
      return Err(())
    }
    if (self.size - 1) == index {
      self.size -= 1;
    }
    else {
      for i in index..(self.size - 1) {
        self.data[i] = self.data[i+1];
      }
      self.size -= 1;
    }
    Ok(())
  }
  
  pub fn remove_element(&mut self, element: &T) -> Result<(), ()> {
    for i in 0..self.size{
      if &self.data[i] == element{
        return self.remove(i);
      }
    }
    Err(())
  }

  pub fn remove_end(&mut self) -> Result<(),()>{
    if self.size == 0{
      return Err(())
    }
    else{
      self.size -= 1;
    }
    Ok(())
  }

  pub fn is_empty(&self) -> bool{
    self.size == 0
  }

}

impl<T> Index<usize> for Array3e<T>{
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    if index >= self.size{
      panic!("Index out of bounds");
    }
    &self.data[index]
  }
}


#[derive(Default)]
pub struct HashGrid<T>{
  grid: Vec<Vec<Array3e<T>>>,
  data: HashSet<T>,
  screen_space: ((f32,f32), (f32,f32)), //top left, bottom right
  ds: u16,
}


//remove debug
impl<T: Eq + Default + Copy + Hash + std::fmt::Debug> HashGrid<T>{
  pub fn new(screen_space: ((f32,f32),(f32,f32)), ds: u16) -> Self{
    let ((tx, ty), (bx, by)) = screen_space;
    let width = bx - tx;
    let height = by - ty;
    // integer-like behavior: number of cells = floor(width/ds) + 1
    let num_columns = (width / ds as f32).floor() as usize + 1;
    let num_rows = (height / ds as f32).floor() as usize + 1;

    if (width / ds as f32).fract() != 0.0 {
      println!("width is not direct multiple of ds\n");
    }
    if (height / ds as f32).fract() != 0.0{
      println!("height is not direct multiple of ds\n");
    }

    let mut new_hash_grid = HashGrid{
      grid: Vec::with_capacity(num_rows),
      data: HashSet::new(),
      screen_space,
      ds,
    };
    new_hash_grid.grid.resize_with(num_rows, Vec::new);

    for vec in new_hash_grid.grid.iter_mut() {
      vec.resize_with(num_columns, Array3e::new);
    }

    new_hash_grid
  }

  pub fn get_element(&mut self, coord: (f32, f32)) -> Option<&T>{
    let(x_index, y_index) = self.coord_to_index(coord);

    if x_index >= self.grid.len() || y_index >= self.grid[x_index].len(){
      return None;
    }

    if self.grid[x_index][y_index].is_empty() {
      None
    } else {
      Some(&(self.grid[x_index][y_index][0]))
    }

  }

  pub fn insert_element(&mut self, coord: (f32, f32), item: &T) {
    let(x_index, y_index) = self.coord_to_index(coord);

    self.grid[x_index][y_index].insert(item.clone());
  }

  pub fn insert_rectangle(&mut self, coord: (f32, f32), dimensions: (f32,f32), item: &T) -> Result<(), HashSet<T>>{
    let top_left = (coord.0 - (dimensions.0 / 2.0), coord.1 - (dimensions.1 / 2.0));
    let bottom_right = (top_left.0 + dimensions.0, top_left.1 + dimensions.1);
    let top_right = (bottom_right.0, top_left.1);
    let bottom_left = (top_left.0, bottom_right.1);
    
    // Check if all corners are within screen_space
    self.grid_dimensions(top_left, bottom_right);

    if self.data.contains(item){
      panic!("All keys must be unique");
    }
    else{
      self.data.insert(item.clone());
    }
    
    // let mut collisions = HashSet::new();
    // while current_point.0 <= bottom_right.0 {
    //   while current_point.1 <= bottom_right.1 {
    //     let (x_index, y_index) = self.coord_to_index(current_point);
    //     for existing in &self.grid[x_index][y_index] {
    //       collisions.insert(existing.clone());
    //     }

    //     current_point.1 += self.ds as f32;
    //   }
    //   current_point.1 = top_left.1;
    //   current_point.0 += self.ds as f32;
    // }
    
    // // Return all collisions if any found
    // if !collisions.is_empty() {
    //   return Err(collisions);
    // }
    
    // Second pass: insert if no collisions
    let mut current_point = top_left;
    while current_point.0 <= bottom_right.0 {
      while current_point.1 <= bottom_right.1 {
        self.insert_element(current_point, item);
        current_point.1 += self.ds as f32;
      }
      current_point.1 = top_left.1;
      current_point.0 += self.ds as f32;
    }
    
    Ok(())
  }

  pub fn move_rectangle(&mut self, from_coord: (f32, f32),dimensions:(f32,f32),to_coord:(f32,f32))->Result<(),HashSet<T>> {
    let top_left_moved = (to_coord.0 - dimensions.0 /2.0, to_coord.1 - dimensions.1 / 2.0);
    let bottom_right_moved = (to_coord.0 + dimensions.0 /2.0, to_coord.1 + dimensions.1 / 2.0);

    if !point_in_screen_space!(self, top_left_moved) && !point_in_screen_space!(self, bottom_right_moved){
      panic!("Moved rectangle not in grid space");
    }

    let (x_index, y_index) = self.coord_to_index(from_coord);
    let value = &self.grid[x_index][y_index];

    if value.is_empty() {
      panic!("You cannot move something that doesn't exist!");
    }
    // choose the first item in the cell to be the key to move
    let key: T = value[0].clone();
      
    
    let top_left_from = (from_coord.0 - dimensions.0 / 2.0, from_coord.1 - dimensions.1 / 2.0);
    let bottom_right_from = (from_coord.0 + dimensions.0 / 2.0, from_coord.1 + dimensions.1 / 2.0);

    // indices for source rectangle (not used directly here)
    let _ft = self.coord_to_index(top_left_from);
    let _fb = self.coord_to_index(bottom_right_from);

    let mt = self.coord_to_index(top_left_moved);
    let mb = self.coord_to_index(bottom_right_moved);

    //check if any collision occurs
    let mut collided_keys = HashSet::new();
    let mut current = mt;
    while current.0 <= mb.0{
      while current.1 <= mb.1{
        for data in &self.grid[current.0][current.1].data {
          if (*data) != key{
            collided_keys.insert(data.clone());
          }
        }
        current.1 += 1;
      }
      current.0 += 1;
      current.1 = mt.1;
    }

    if !collided_keys.is_empty(){
      return Err(collided_keys);
    }

    let _ = self.remove_rectangle(from_coord, dimensions, &key);
    

    println!("{:#?}, {:#?}", mt, mb);
    println!("{:#?}", self);
    let mut current = mt;
    while current.0 <= mb.0{
      while current.1 <= mb.1{
        self.grid[current.0][current.1].insert(key.clone());
        current.1 += 1;
      }
      current.0 += 1;
      current.1 = mt.1;
    }


    Ok(())
    // TODO: Complete the rest of the function
  }


  pub fn remove_rectangle(&mut self, coord: (f32, f32), dimensions: (f32, f32), element: &T) -> Result<(), ()> {
    //remove these extra checks. You cannot click on sth not existing
    let top_left = (coord.0 - (dimensions.0 / 2.0), coord.1 - (dimensions.1 / 2.0));
    let bottom_right = (top_left.0 + dimensions.0, top_left.1 + dimensions.1);
    let top_right = (bottom_right.0, top_left.1);
    let bottom_left = (top_left.0, bottom_right.1);
    
    // Check if all corners are within screen_space
    if !point_in_screen_space!(self, top_left) || 
       !point_in_screen_space!(self, top_right) || 
       !point_in_screen_space!(self, bottom_left) || 
       !point_in_screen_space!(self, bottom_right) {
      return Err(());
    }

    let top_left_indices = self.coord_to_index(top_left);
    let bottom_right_indices = self.coord_to_index(bottom_right);

    // let keys_to_remove = self.grid[top_left_indices.0][top_left_indices.1].clone();
    // if !keys_to_remove.is_empty() {
    //   // remove all items found in the top-left cell from the master set
    //   for d in keys_to_remove.iter() {
    //     self.data.remove(d);
    //   }
    // }
    // else{
    //   panic!("You cannot remove something that does not exist!");
    // }
    
    // Iterate through rectangle and remove matching items from each cell
    let mut current_indices = top_left_indices;
    while current_indices.0 <= bottom_right_indices.0 {
      while current_indices.1 <= bottom_right_indices.1 {
        // remove all items that were present in the top-left cell
        self.grid[current_indices.0][current_indices.1].remove_element(element);
        current_indices.1 += 1 as usize;
      }
      current_indices.1 = top_left_indices.1;
      current_indices.0 += 1 as usize;
    }
    
    Ok(())
  }

  #[inline(always)]
  fn coord_to_index(&self, coord: (f32, f32))->(usize, usize){
    let dx = coord.0 - self.screen_space.0.0;
    let dy = coord.1 - self.screen_space.0.1;

    if dx < 0.0 || dy < 0.0{
      panic!("Error: Negative indices!");
    }

    let x = (dx / self.ds as f32).floor() as usize;
    let y = (dy / self.ds as f32).floor() as usize;

    (x, y)
    
  }


  pub fn grid_dimensions(&mut self,top_left: (f32, f32) ,bottom_right: (f32, f32)){
    let new_screen_space = ((top_left.0.min(self.screen_space.0.0), top_left.1.min(self.screen_space.0.1)), 
                            (bottom_right.0.max(self.screen_space.1.0), bottom_right.1.max(self.screen_space.1.1)));
    
    let left_diff = if self.screen_space.0.0 > new_screen_space.0.0 {
      ((self.screen_space.0.0 - new_screen_space.0.0) / self.ds as f32).floor() as usize
    } else { 0 };
    let top_diff = if self.screen_space.0.1 > new_screen_space.0.1 {
      ((self.screen_space.0.1 - new_screen_space.0.1) / self.ds as f32).floor() as usize
    } else { 0 };
    let width_increase = left_diff + if new_screen_space.1.0 > self.screen_space.1.0 {
      ((new_screen_space.1.0 - self.screen_space.1.0) / self.ds as f32).floor() as usize
    } else { 0 };
    let height_increase = top_diff + if new_screen_space.1.1 > self.screen_space.1.1 {
      ((new_screen_space.1.1 - self.screen_space.1.1) / self.ds as f32).floor() as usize
    } else { 0 };

    if width_increase == 0 && height_increase == 0 {
      println!("nothing");
      return;
    }

    self.screen_space = new_screen_space;
    self.grid.resize_with(self.grid.len() + width_increase, Vec::new );
    if left_diff != 0 {
      self.grid.rotate_right(left_diff);
    }

    let new_height = ((new_screen_space.1.1 - new_screen_space.0.1) / (self.ds as f32).floor()) as usize + 1;
    if top_diff != 0 {
      for vec in self.grid.iter_mut() {
        vec.resize_with(new_height as usize,|| Array3e::new() );
        vec.rotate_right(top_diff);
      }
    }
    else{
      for vec in self.grid.iter_mut() {
        vec.resize_with( new_height as usize,|| Array3e::new() );
      }
    }
  }
}

impl<T: std::fmt::Debug> std::fmt::Debug for HashGrid<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "HashGrid {{")?;
    writeln!(f, "  screen_space: {:?},", self.screen_space)?;
    writeln!(f, "  ds: {},", self.ds)?;
    writeln!(f, "  grid: [")?;
    
    for (row_idx, row) in self.grid.iter().enumerate() {
      write!(f, "    [{:2}] ", row_idx)?;
      for cell in row {
        if cell.size == 0 {
          write!(f, "   ·")?;
        } else {
          // show a short preview: number of items in cell
          write!(f, "{:>4}", format!("#{}", cell.size))?;
        }
      }
      writeln!(f)?;
    }
    
    writeln!(f, "  ]")?;
    write!(f, "}}")
  }
}

