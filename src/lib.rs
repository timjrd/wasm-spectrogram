use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::Clamped;

use web_sys::*;
use js_sys::*;

use rustfft::algorithm::Radix4;
use rustfft::FFT;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

#[wasm_bindgen]
pub fn main() {
  let mut input:  Vec<Complex<f32>> = vec![Zero::zero(); 4096];
  let mut output: Vec<Complex<f32>> = vec![Zero::zero(); 4096];

  let fft = Radix4::new(4096, false);
  fft.process(&mut input, &mut output);

  let canvas: HtmlCanvasElement = window()
    .unwrap()
    .document()
    .unwrap()
    .get_elements_by_tag_name("canvas")
    .item(0)
    .unwrap()
    .dyn_into()
    .unwrap();

  let canvas_ctx: CanvasRenderingContext2d = canvas
    .get_context("2d")
    .unwrap()
    .unwrap()
    .dyn_into()
    .unwrap();

  let audio_ctx = AudioContext::new().unwrap();

  let proc_node = audio_ctx
    .create_script_processor_with_buffer_size_and_number_of_input_channels(1024, 1)
    .unwrap();
  
  let proc: Closure<dyn FnMut(_)> = Closure::new(move |event: AudioProcessingEvent| {
    let input_data = event
      .input_buffer()
      .unwrap()
      .get_channel_data(0)
      .unwrap();

    let width  = canvas.width();
    let height = canvas.height();

    let pixel = |x,y| {
      let nx = x as f32 / width as f32;
      let ny = y as f32 / height as f32;

      let i = (nx * (input_data.len()-1) as f32) as usize;
      let ty = (input_data[i] + 1.0) / 2.0;

      let c = if ny < ty {50} else {222};
      
      vec![c,0,c,255]
    };
    
    let mut image_data: Vec<u8> = (0..height).flat_map(|y| {
      (0..width).flat_map(move |x| pixel(x,y))
    }).collect();

    let image =
      ImageData::new_with_u8_clamped_array(
        Clamped(&mut image_data),
        canvas.width())
      .unwrap();
    
    canvas_ctx.put_image_data(&image, 0.0, 0.0).unwrap();    
  });
  
  proc_node.set_onaudioprocess(Some(proc.as_ref().unchecked_ref()));
  
  let on_stream = Closure::new(move |x: JsValue| {
    let stream: MediaStream = x.dyn_into().unwrap();
    let node = audio_ctx.create_media_stream_source(&stream).unwrap();
    node.connect_with_audio_node(&proc_node).unwrap();
  });

  window()
    .unwrap()
    .navigator()
    .media_devices()
    .unwrap()
    .get_user_media_with_constraints(
      MediaStreamConstraints::new().audio(&true.into()) )
    .unwrap()
    .then(&on_stream);
    
  proc.forget();
  on_stream.forget();

  log("done.");
}

fn log(msg: &str) {
  console::debug(&Array::of1(&msg.into()));
}
