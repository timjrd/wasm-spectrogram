use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::Clamped;
use wasm_bindgen::closure::Closure;

use web_sys::window;
use web_sys::HtmlCanvasElement;
use web_sys::CanvasRenderingContext2d;
use web_sys::ImageData;

use crate::error::Error;


type PixelFn  = Box<dyn FnMut(Coord) -> Pixel>;
type CancelFn = Box<dyn FnMut() -> bool>;
type Callback = Closure<dyn FnMut(f64)>;

pub struct Renderer {
  pub canvas_id: String,
  pub pixel: PixelFn,
  pub cancel: CancelFn,
  pub resolution: f32,
}

pub struct Coord {
  pub x: usize,
  pub y: usize,
  pub width: usize,
  pub height: usize,
}

pub struct Pixel(pub u8, pub u8, pub u8);

struct Renderer_ {
  args: Renderer,
  canvas: HtmlCanvasElement,
  context: CanvasRenderingContext2d,
  data: Vec<u8>,
  callback: Callback,
}


pub fn start_rendering(args: Renderer) -> Result<(),Error> {
  let renderer = Rc::new(RefCell::new(None));
  
  let callback = {
    let renderer = renderer.clone();
    Closure::new(move |_| {
      if renderer
        .borrow_mut()
        .as_mut()
        .and_then(|r| on_frame(r).ok())
        .is_none() {
          renderer.replace(None);
        }
    })
  };

  let canvas: HtmlCanvasElement = window()?
    .document()?
    .get_element_by_id(&args.canvas_id)?
    .dyn_into()?;

  let context: CanvasRenderingContext2d = canvas
    .get_context("2d")??
    .dyn_into()?;
  
  let (width,height) = resize(&canvas, args.resolution);
  
  window()?.request_animation_frame(
    callback.as_ref().dyn_ref()?
  )?;
  
  *renderer.borrow_mut() = Some( Renderer_ {
    args: args,
    canvas: canvas,
    context: context,
    data: vec![0; 4*width*height],
    callback: callback,
  });
  
  Ok(())
}

// struct Pixel_<'a> {
//   x: usize,
//   y: usize,
//   red: &'a mut u8,
//   green: &'a mut u8,
//   blue: &'a mut u8,
// }

fn on_frame(renderer: &mut Renderer_) -> Result<(),Error> {
  if (renderer.args.cancel)() {
    return Err(Error());
  }
  
  let (width,height) = resize(&renderer.canvas, renderer.args.resolution);
  renderer.data.resize(4*width*height, 0);
  
  for (y,row) in renderer.data.chunks_exact_mut(4*width).enumerate() {
    for (x,p) in row.chunks_exact_mut(4).enumerate() {
      let pixel = (renderer.args.pixel)( Coord {
        x: x,
        y: y,
        width: width,
        height: height,
      });
      p[0] = pixel.0;
      p[1] = pixel.1;
      p[2] = pixel.2;
      p[3] = u8::max_value();
    }
  }
  
  // let iter = renderer.data
  //   .chunks_exact_mut(4*width).enumerate().flat_map(|(y,row)| {
  //     row.chunks_exact_mut(4).enumerate().map(move |(x,pixel)| {
  //       if let [r,g,b,a] = pixel {
  //         *a = u8::max_value();
  //         Pixel_ {
  //           x: x,
  //           y: y,
  //           red: r,
  //           green: g,
  //           blue: b,
  //         }
  //       }
  //       else {
  //         panic!()
  //       }
  //     })
  //   });
  
  let image = ImageData::new_with_u8_clamped_array(
    Clamped(&mut renderer.data), width as u32 )?;
  
  renderer.context.put_image_data(&image, 0.0, 0.0)?;
  
  window()?.request_animation_frame(
    renderer.callback.as_ref().dyn_ref()?
  )?;
  
  Ok(())
}

fn resize(canvas: &HtmlCanvasElement, resolution: f32) -> (usize,usize) {
  let w = canvas.client_width();
  let h = canvas.client_height();

  let width  = (w as f32 * resolution).round() as usize;
  let height = (h as f32 * resolution).round() as usize;

  canvas.set_width(width as u32);
  canvas.set_height(height as u32);
  
  (width, height)
}
