use rustfft::algorithm::Radix4;
use rustfft::FFT;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

//use web_sys::CanvasRenderingContext2d;

use crate::error::Error;
use crate::ring::Ring;
use crate::audio::Buffer;
use crate::canvas::Line;


pub struct Spectrogram {
  from_key: f32,
  to_key: f32,
  boost: f32,
  sample_rate: f32,
  
  fft: Radix4<f32>,
  queue: Ring<Complex<f32>>,
  input: Vec<Complex<f32>>,
  output: Vec<Complex<f32>>,
  
  freq_sum: Vec<f32>,
  freq_n: usize,
}


impl Spectrogram {
  pub fn new( buffer_size_power: u32,
              from_key: f32,
              to_key: f32,
              boost: f32 ) -> Spectrogram {
    
    let queue_size = 2_usize.pow(buffer_size_power);
    let buffer_size = 2_usize.pow(buffer_size_power - 1);
    
    Spectrogram {
      from_key: from_key,
      to_key: to_key,
      boost: boost,
      sample_rate: 1.0,
      
      fft: Radix4::new(buffer_size, false),
      queue: Ring::new(queue_size, Complex::zero()),
      input: vec![Complex::zero(); buffer_size],
      output: vec![Complex::zero(); buffer_size],
      
      freq_sum: vec![0.0; buffer_size / 2 - 1],
      freq_n: 0,
    }
  }
  
  pub fn process(&mut self, buffer: Buffer) -> Result<(),Error> {
    self.sample_rate = buffer.sample_rate / 2.0;
    
    for sample in buffer.data.iter_mut() {
      self.queue.enqueue(Complex::new(*sample.left, 0.0));

      for (src,dst) in self.queue.chunks_exact(2)
        .zip(self.input.iter_mut()) {
          if let [a,b] = src {
            *dst = (a + b) / 2.0;
          }
        }
      
      self.fft.process(&mut self.input, &mut self.output);
      
      for (bin,sum) in
        self.output[1 .. self.output.len() / 2]
        .iter().zip(self.freq_sum.iter_mut()) {
          *sum += 2.0 * bin.norm() / self.output.len() as f32;
        }
      
      self.freq_n += 1;
      
      *sample.left  = 0.0;
      *sample.right = 0.0;
    }

    Ok(())
  }

  // pub fn draw_first_frame(context: &CanvasRenderingContext2d)
  //                         -> Result<(),Error> {
    
  //   let width  = context.canvas()?.width()  as f64;
  //   let height = context.canvas()?.height() as f64;
    
  //   context.set_fill_style(&"black".into());
  //   context.set_stroke_style(&"white".into());
  //   context.set_line_join("round");
  //   context.set_line_width(0.005 * height);
    
  //   context.rect(0.0, 0.0, width, height);
  //   context.fill();

  //   Ok(())
  // }
  
  pub fn draw_frame(&mut self, line: Line) -> Result<(),Error> {
    if self.freq_n == 0 {
      return Ok(());
    }

    let keys = self.to_key - self.from_key + 1.0;
    
    for pixel in line.data.iter_mut() {
      let x = pixel.x as f32 / (line.len - 1) as f32;
      let f = from_piano_key(
        x * keys + self.from_key - 0.5
      );
      
      let i = (f * self.output.len() as f32 / self.sample_rate - 1.0)
        .max(0.0).min(self.freq_sum.len() as f32 - 1.0);
      
      let i0 = i.floor() as usize;
      let i1 = i.ceil()  as usize;
      let di = i.fract();

      let v0 = self.freq_sum[i0] / self.freq_n as f32;
      let v1 = self.freq_sum[i1] / self.freq_n as f32;

      let v = boost(v0 * (1.0 - di) + v1 * di, self.boost);

      let c = ((1.0 - v) * u8::max_value() as f32) as u8;
      
      *pixel.r = c;
      *pixel.g = c;
      *pixel.b = c;
    }
    
    self.freq_n = 0;
    for sum in self.freq_sum.iter_mut() {
      *sum = 0.0;
    }
    
    Ok(())
    
    // let last = self.freq_sum.len() - 1;
    
    // let width  = context.canvas()?.width()  as f64;
    // let height = context.canvas()?.height() as f64;

    // let graph_height = 0.05 * height;
    // let shift = 0.01 * height;
    
    // context.set_global_composite_operation("copy")?;
    // context.draw_image_with_html_canvas_element(
    //   &context.canvas()?, 0.0, -shift
    // )?;
    // context.set_global_composite_operation("source-over")?;
    
    // // context.rect(0.0, height - shift, width, shift);
    // // context.fill();

    // // context.set_global_alpha(0.02);
    // // context.rect(0.0, 0.0, width, height);
    // // context.fill();
    // // context.set_global_alpha(1.0);
    
    // context.begin_path();
    
    // for (index,sum) in self.freq_sum.iter_mut().enumerate() {
    //   let hz = (index + 1) as f32
    //     * self.sample_rate
    //     / self.output.len() as f32;
      
    //   let key = piano_key(hz);
      
    //   let x = (key - 28.0) / 86.0;
    //   let j = x as f64 * width;

    //   let y = boost(*sum / self.freq_n as f32, self.boost);
    //   let i = height - graph_height * y as f64;

    //   if index == 0 {
    //     context.move_to(-width, i);
    //   }
      
    //   context.line_to(j,i);
      
    //   if index == last {
    //     context.line_to(2.0 * width, i);
    //   }
      
    //   *sum = 0.0;
    // }

    // context.stroke();
    
    // self.freq_n = 0;
    
    // Ok(())
  }
}


fn boost(value: f32, by: f32) -> f32 {
  ((by + 1.0) * value) / (by * value + 1.0)
}

// https://en.wikipedia.org/wiki/Piano_key_frequencies
fn from_piano_key(n: f32) -> f32 {
  2.0_f32.powf((n - 49.0) / 12.0) * 440.0
}
