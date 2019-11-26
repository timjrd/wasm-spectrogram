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


type DrawLine = Box<dyn FnMut(Line) -> Result<(),Error>>;
type OnFrame = Closure<dyn FnMut(f64)>;

pub struct Renderer {
  pub canvas_id: String,
  pub draw_frame: DrawLine,
  pub resolution: f32,
}

pub struct Line<'a> {
  pub len: usize,
  pub data: &'a mut LineData<'a>,
}

pub struct LineData<'a>(&'a mut [u8]);

pub struct Pixel<'a> {
  pub x: usize,
  pub r: &'a mut u8,
  pub g: &'a mut u8,
  pub b: &'a mut u8,
}

struct Renderer_ {
  args: Renderer,
  context: CanvasRenderingContext2d,
  data: Vec<u8>,
  on_frame: OnFrame,
}


impl LineData<'_> {
  pub fn iter_mut(&mut self) -> impl Iterator<Item=Pixel> {
    self.0.chunks_exact_mut(4).enumerate().map(move |(x,pixel)| {
      if let [r,g,b,a] = pixel {
        *a = u8::max_value();
        Pixel { x:x, r:r, g:g, b:b }
      }
      else {
        panic!()
      }
    })
  }
}


pub fn start_rendering(args: Renderer) -> Result<(),Error> {
  let renderer = Rc::new(RefCell::new(None));
  
  let on_frame = {
    let renderer = renderer.clone();
    Closure::new(move |_| {
      if renderer.borrow_mut().as_mut()
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
  
  let (width,_) = resize(&canvas, args.resolution);

  context.set_fill_style(&"black".into());
  
  window()?.request_animation_frame(
    on_frame.as_ref().dyn_ref()?
  )?;
  
  *renderer.borrow_mut() = Some( Renderer_ {
    args: args,
    context: context,
    data: vec![0; 4 * width],
    on_frame: on_frame,
  });
  
  Ok(())
}

fn on_frame(renderer: &mut Renderer_) -> Result<(),Error> {
  
  let (width,height) = resize(
    &renderer.context.canvas()?,
    renderer.args.resolution
  );
  renderer.data.resize(4 * width, 0);
  
  (renderer.args.draw_frame)( Line {
    len: width,
    data: &mut LineData(&mut renderer.data),
  })?;
  
  let line = ImageData::new_with_u8_clamped_array(
    Clamped(&mut renderer.data), width as u32 )?;
  
  renderer.context.draw_image_with_html_canvas_element(
    &renderer.context.canvas()?, 0.0, -1.0
  )?;
  
  renderer.context.put_image_data(&line, 0.0, (height - 1) as f64)?;
  
  window()?.request_animation_frame(
    renderer.on_frame.as_ref().dyn_ref()?
  )?;
  
  Ok(())
}

fn resize(canvas: &HtmlCanvasElement, resolution: f32) -> (usize,usize) {
  let w = canvas.client_width();
  let h = canvas.client_height();

  let width  = (w as f32 * resolution).round() as u32;
  let height = (h as f32 * resolution).round() as u32;

  if width != canvas.width() {
    canvas.set_width(width);
  }
  
  if height != canvas.height() {
    canvas.set_height(height);
  }
  
  (width as usize, height as usize)
}
