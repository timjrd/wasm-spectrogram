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


type DrawFrame = Box<dyn FnMut(Frame) -> bool>;
type OnFrame = Closure<dyn FnMut(f64)>;

pub struct Renderer {
  pub canvas_id: String,
  pub draw_frame: DrawFrame,
  pub resolution: f32,
}

pub struct Frame<'a> {
  pub width: usize,
  pub height: usize,
  pub data: &'a mut FrameData<'a>,
}

pub struct FrameData<'a> {
  width: usize,
  data: &'a mut [u8],
}

pub struct Pixel<'a> {
  pub x: usize,
  pub y: usize,
  pub red: &'a mut u8,
  pub green: &'a mut u8,
  pub blue: &'a mut u8,
}

struct Renderer_ {
  args: Renderer,
  canvas: HtmlCanvasElement,
  context: CanvasRenderingContext2d,
  data: Vec<u8>,
  on_frame: OnFrame,
}


impl FrameData<'_> {
  pub fn iter_mut(&mut self) -> impl Iterator<Item=Pixel> {
    self.data.chunks_exact_mut(4*self.width).enumerate().flat_map(|(y,row)| {
      row.chunks_exact_mut(4).enumerate().map(move |(x,pixel)| {
        if let [r,g,b,a] = pixel {
          *a = u8::max_value();
          Pixel {
            x: x,
            y: y,
            red: r,
            green: g,
            blue: b,
          }
        }
        else {
          panic!()
        }
      })
    })
  }
}


pub fn start_rendering(args: Renderer) -> Result<(),Error> {
  let renderer = Rc::new(RefCell::new(None));
  
  let on_frame = {
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
    on_frame.as_ref().dyn_ref()?
  )?;
  
  *renderer.borrow_mut() = Some( Renderer_ {
    args: args,
    canvas: canvas,
    context: context,
    data: vec![0; 4*width*height],
    on_frame: on_frame,
  });
  
  Ok(())
}

fn on_frame(renderer: &mut Renderer_) -> Result<(),Error> {
  
  let (width,height) = resize(&renderer.canvas, renderer.args.resolution);
  renderer.data.resize(4*width*height, 0);
  
  if (renderer.args.draw_frame)( Frame {
    width: width,
    height: height,
    data: &mut FrameData {
      width: width,
      data: &mut renderer.data,
    }
  }) {
    let image = ImageData::new_with_u8_clamped_array(
      Clamped(&mut renderer.data), width as u32 )?;
    
    renderer.context.put_image_data(&image, 0.0, 0.0)?;
    
    window()?.request_animation_frame(
      renderer.on_frame.as_ref().dyn_ref()?
    )?;
    
    Ok(())
  }
  else {
    Err(Error())
  }
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
