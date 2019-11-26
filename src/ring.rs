use std::ops::Deref;
use std::ops::DerefMut;


pub struct Ring<T> {
  data: Vec<T>,
  pos: usize,
}


impl<T: Copy> Ring<T> {
  pub fn new(size: usize, value: T) -> Ring<T> {
    Ring {
      data: vec![value; size * 2],
      pos: 0,
    }
  }
  
  pub fn enqueue(&mut self, value: T) {
    let size = self.data.len() / 2;
    
    if self.pos > 0 {
      self.data[self.pos - 1] = value;
    }
    
    if self.pos < size {
      self.data[self.pos + size] = value;
    }
    
    self.pos = (self.pos + 1) % (size + 1);
  }
}

impl<T> Deref for Ring<T> {
  type Target = [T];
  fn deref(&self) -> &[T] {
    let size = self.data.len() / 2;
    &self.data[self.pos .. self.pos + size]
  }
}

impl<T> DerefMut for Ring<T> {
  fn deref_mut(&mut self) -> &mut [T] {
    let size = self.data.len() / 2;
    &mut self.data[self.pos .. self.pos + size]
  }
}
