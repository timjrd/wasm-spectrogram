#![feature(try_trait)]

mod error;
mod canvas;
mod audio;


use std::f32::consts::PI;

use wasm_bindgen::prelude::*;

// use rustfft::algorithm::Radix4;
// use rustfft::FFT;
// use rustfft::num_complex::Complex;
// use rustfft::num_traits::Zero;

use canvas::Renderer;
use canvas::Coord;
use canvas::Pixel;

use audio::Processor;
use audio::Input;


#[wasm_bindgen]
pub fn main() {
  console_error_panic_hook::set_once();
  
  let pixel = Box::new(|coord: Coord| {
    let nx = coord.x as f32 / coord.width as f32;
    let r = (nx * 255.0) as u8;
    let g = (nx * 200.0) as u8;
    let b = (nx * 100.0) as u8;
    Pixel(r,g,b)    
  });

  let mut angle = 0.0;
  let process = Box::new(move |input: Input| {
    let dt = 1.0 / input.sample_rate;
    angle += (2.0*PI * 440.0 * dt) % (2.0*PI);
    f32::sin(angle) * 0.8
  });
  
  canvas::start_rendering( Renderer {
    canvas_id: "canvas".to_string(),
    pixel: pixel,
    cancel: Box::new(|| false),
    resolution: 0.5,
  }).unwrap();

  audio::start_processing( Processor {
    buffer_size: 1024,
    process: process,
    cancel: Box::new(|| false),
  }).unwrap();
}
