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
use canvas::Frame;

use audio::Processor;
use audio::Buffer;


#[wasm_bindgen]
pub fn main() {
  console_error_panic_hook::set_once();
  
  let draw_frame = Box::new(|frame: Frame| {
    for pixel in frame.data.iter_mut() {
      let nx = pixel.x as f32 / frame.width as f32;
      *pixel.red = (nx * 255.0) as u8;
      *pixel.green = (nx * 200.0) as u8;
      *pixel.blue = (nx * 100.0) as u8;
    }
    true
  });

  let mut l = 0.0;
  let mut r = 0.0;
  let hz = 440.0;
  let process = Box::new(move |buffer: Buffer| {
    let dt = 1.0 / buffer.sample_rate;
    for sample in buffer.data.iter_mut() {
      l += (2.0*PI * hz * dt) % (2.0*PI);
      r += (2.0*PI * (hz/2.0) * dt) % (2.0*PI);
      *sample.left = f32::sin(l) * 0.8;
      *sample.right = f32::sin(r) * 0.8;
    }
    true
  });
  
  canvas::start_rendering( Renderer {
    canvas_id: "canvas".to_string(),
    draw_frame: draw_frame,
    resolution: 0.5,
  }).unwrap();

  audio::start_processing( Processor {
    buffer_size: 1024,
    process: process,
  }).unwrap();
}
