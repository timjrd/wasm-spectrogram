#![feature(try_trait)]

mod error;
mod canvas;
mod audio;
mod ring;
mod spectrogram;


use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use audio::Processor;
use canvas::Renderer;
use spectrogram::Spectrogram;


#[wasm_bindgen]
pub fn main() {
  console_error_panic_hook::set_once();
  
  let spectrogram = Rc::new(RefCell::new(
    Spectrogram::new(9, 40.0, 87.0, 25.0)
  ));
  
  let s = spectrogram.clone();
  audio::start_processing( Processor {
    buffer_size: 512,
    process: Box::new(move |buffer| {
      s.borrow_mut().process(buffer)
    }),
  }).unwrap();
  
  let s = spectrogram;
  canvas::start_rendering( Renderer {
    canvas_id: "canvas".to_string(),
    resolution: 0.7,
    draw_frame: Box::new(move |context| {
      s.borrow_mut().draw_frame(context)
    }),
  }).unwrap();
}
