use std::collections::{hash_set, HashSet};
use std::hash::Hash;
use std::thread::current;

macro_rules! point_in_screen_space {
    ($self:expr, $point:expr) => {
        $point.0 <= $self.screen_space.1.0 
            && $point.0 >= $self.screen_space.0.0
            && $point.1 <= $self.screen_space.1.1 
            && $point.1 >= $self.screen_space.0.1
    };
}


#[derive(Default)]
pub struct HashGrid<T>{
  grid: Vec<Vec<Option<T>>>,
  data: HashSet<T>,
  screen_space: ((i32,i32), (i32,i32)), //top left, bottom right
  ds: u16,
}


//remove debug
impl<T: Eq + Clone + Hash + std::fmt::Debug> HashGrid<T>{
  pub fn new(screen_space: ((i32,i32),(i32,i32)), ds: u16) -> Self{
    let ((tx, ty), (bx, by)) = screen_space;
    let width = bx - tx;
    let height = by - ty;
    let num_columns = (width / ds as i32) + 1;
    let num_rows = (height / ds as i32) + 1;

    if width % ds as i32 != 0 {
      println!("width is not direct multiple of ds\n");
    }
    if height % ds as i32 != 0{
      println!("height is not direct multiple of ds\n");
    }

    let mut new_hash_grid = HashGrid{
      grid: Vec::with_capacity(num_rows as usize),
      data: HashSet::new(),
      screen_space,
      ds,
    };
    new_hash_grid.grid.resize_with(num_rows as usize, Vec::new);

    for vec in new_hash_grid.grid.iter_mut() {
      vec.resize_with(num_columns as usize, || None);
    }

    new_hash_grid
  }

  pub fn get_element(&mut self, coord: (i32, i32)) -> Option<T>{
    let(x_index, y_index) = self.coord_to_index(coord);

    if x_index >= self.grid.len() || y_index >= self.grid[x_index].len(){
      return None;
    }

    Some(self.grid[x_index][y_index].as_ref().unwrap().clone())

  }

  pub fn insert_element(&mut self, coord: (i32, i32), item: &T) {
    let(x_index, y_index) = self.coord_to_index(coord);

    self.grid[x_index][y_index] = Some(item.clone());
  }

  pub fn insert_rectangle(&mut self, coord: (i32, i32), dimensions: (i32,i32), item: &T) -> Result<(), HashSet<T>>{
    let top_left = (coord.0 - (dimensions.0 / 2), coord.1 - (dimensions.1 / 2));
    let bottom_right = (top_left.0 + dimensions.0, top_left.1 + dimensions.1);
    let top_right = (bottom_right.0, top_left.1);
    let bottom_left = (top_left.0, bottom_right.1);
    
    // Check if all corners are within screen_space
    if !point_in_screen_space!(self, top_left) || 
       !point_in_screen_space!(self, top_right) || 
       !point_in_screen_space!(self, bottom_left) || 
       !point_in_screen_space!(self, bottom_right) {
      return Err(HashSet::new());
    }

    if self.data.contains(item){
      panic!("All keys must be unique");
    }
    else{
      self.data.insert(item.clone());
    }
    
    let mut collisions = HashSet::new();
    
    // First pass: check for collisions
    let mut current_point = top_left;
    while current_point.0 <= bottom_right.0 {
      while current_point.1 <= bottom_right.1 {
        let (x_index, y_index) = self.coord_to_index(current_point);
        if let Some(existing) = &self.grid[x_index][y_index] {
          collisions.insert(existing.clone());
        }
        
        current_point.1 += self.ds as i32;
      }
      current_point.1 = top_left.1;
      current_point.0 += self.ds as i32;
    }
    
    // Return all collisions if any found
    if !collisions.is_empty() {
      return Err(collisions);
    }
    
    // Second pass: insert if no collisions
    current_point = top_left;
    while current_point.0 <= bottom_right.0 {
      while current_point.1 <= bottom_right.1 {
        self.insert_element(current_point, item);
        current_point.1 += self.ds as i32;
      }
      current_point.1 = top_left.1;
      current_point.0 += self.ds as i32;
    }
    
    Ok(())
  }

  pub fn move_rectangle(&mut self, from_coord: (i32, i32),dimensions:(i32,i32),to_coord:(i32,i32))->Result<(),HashSet<T>> {
    let top_left_moved = (to_coord.0 - dimensions.0 /2, to_coord.1 - dimensions.1 / 2);
    let bottom_right_moved = (to_coord.0 + dimensions.0 /2, to_coord.1 + dimensions.1 / 2);

    if !point_in_screen_space!(self, top_left_moved) && !point_in_screen_space!(self, bottom_right_moved){
      panic!("Moved rectangle not in grid space");
    }

    let (x_index, y_index) = self.coord_to_index(from_coord);
    let value = &self.grid[x_index][y_index];

    if value.is_none() {
      panic!("You cannot move something that doesn't exist!");
    }
    //you can't extract owned value from a reference like &self
    let key: T = value.as_ref().unwrap().clone();
      
    
    let top_left_from = (from_coord.0 - dimensions.0 / 2, from_coord.1 - dimensions.1 / 2);
    let bottom_right_from = (from_coord.0 + dimensions.0 / 2, from_coord.1 + dimensions.1 / 2);

    let ft = self.coord_to_index(top_left_from);
    let fb = self.coord_to_index(bottom_right_from);

    let mt = self.coord_to_index(top_left_moved);
    let mb = self.coord_to_index(bottom_right_moved);

    //check if any collision occurs
    let mut collided_keys = HashSet::new();
    let mut current = mt;
    while current.0 <= mb.0{
      while current.1 <= mb.1{
        if let Some(data) = &self.grid[current.0][current.1]{
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

    let _ = self.remove_rectangle(from_coord, dimensions);
    

    println!("{:#?}, {:#?}", mt, mb);
    println!("{:#?}", self);
    let mut current = mt;
    let value = Some(key);
    while current.0 <= mb.0{
      while current.1 <= mb.1{
        self.grid[current.0][current.1] = value.clone();
        current.1 += 1;
      }
      current.0 += 1;
      current.1 = mt.1;
    }


    Ok(())
    // TODO: Complete the rest of the function
  }


  pub fn remove_rectangle(&mut self, coord: (i32, i32), dimensions: (i32, i32)) -> Result<(), ()> {
    //remove these extra checks. You cannot click on sth not existing
    let top_left = (coord.0 - (dimensions.0 / 2), coord.1 - (dimensions.1 / 2));
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

    let data_removed = &self.grid[top_left_indices.0][top_left_indices.1];
    if let Some(data) = data_removed {
      self.data.remove(data);
    }
    else{
      panic!("You cannot remove something that does not exist!");
    }
    
    // Iterate through rectangle and set all cells to None
    let mut current_indices = top_left_indices;
    while current_indices.0 <= bottom_right_indices.0 {
      while current_indices.1 <= bottom_right_indices.1 {
        //change this convert to coord earlier than here
        self.grid[current_indices.0][current_indices.1] = None;
        
        current_indices.1 += 1 as usize;
      }
      current_indices.1 = top_left_indices.1;
      current_indices.0 += 1 as usize;
    }
    
    Ok(())
  }

  #[inline(always)]
  fn coord_to_index(&self, coord: (i32, i32))->(usize, usize){
    let new_indices = ((coord.0 - self.screen_space.0.0)/self.ds as i32 ,
    (coord.1 - self.screen_space.0.1)/ self.ds as i32);

    if new_indices.0 < 0 || new_indices.1 < 0{
      panic!("Error: Negative indices!");
    }
    (new_indices.0 as usize, new_indices.1 as usize)
    
  }


  pub fn grid_dimensions(&mut self,top_left: (i32, i32) ,bottom_right: (i32, i32)){
    let new_screen_space = ((top_left.0.min(self.screen_space.0.0), top_left.1.min(self.screen_space.0.1)), 
                            (bottom_right.0.max(self.screen_space.1.0), bottom_right.1.max(self.screen_space.1.1)));
    
    let left_diff = ((self.screen_space.0.0 - new_screen_space.0.0) / self.ds as i32) as usize;
    let top_diff = ((self.screen_space.0.1 - new_screen_space.0.1) / self.ds as i32) as usize;
    let width_increase = left_diff + ((new_screen_space.1.0 - self.screen_space.1.0) / self.ds as i32) as usize;
    let height_increase = top_diff + ((new_screen_space.1.1 - self.screen_space.1.1) / self.ds as i32) as usize;

    if width_increase == 0 && height_increase == 0 {
      println!("nothing");
      return;
    }

    self.screen_space = new_screen_space;
    self.grid.resize_with(self.grid.len() + width_increase, Vec::new);
    if left_diff != 0 {
      self.grid.rotate_right(left_diff);
    }

    let new_height = (new_screen_space.1.1 - new_screen_space.0.1) / (self.ds as i32) + 1;
    if top_diff != 0 {
      for vec in self.grid.iter_mut() {
        vec.resize_with(new_height as usize,|| None);
        vec.rotate_right(top_diff);
      }
    }
    else{
      for vec in self.grid.iter_mut() {
        vec.resize_with( new_height as usize,|| None);
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
        match cell {
          Some(value) => write!(f, "{:>4}", format!("{:?}", value).chars().take(4).collect::<String>())?,
          None => write!(f, "   ·")?,
        }
      }
      writeln!(f)?;
    }
    
    writeln!(f, "  ]")?;
    write!(f, "}}")
  }
}

