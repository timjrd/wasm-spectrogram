use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

use web_sys::window;
use web_sys::AudioContext;
use web_sys::ScriptProcessorNode;
use web_sys::AudioProcessingEvent;
use web_sys::MediaStream;
use web_sys::MediaStreamConstraints;

use crate::error::Error;


type Process = Box<dyn FnMut(Buffer) -> Result<(),Error>>;

pub struct Processor {
  pub buffer_size: u32,
  pub process: Process,
}

pub struct Buffer<'a> {
  pub sample_rate: f32,
  pub data: &'a mut BufferData<'a>,
}

pub struct BufferData<'a> {
  left: &'a mut [f32],
  right: &'a mut [f32]
}

pub struct Sample<'a> {
  pub left: &'a mut f32,
  pub right: &'a mut f32,
}

struct Processor_ {
  args: Processor,
  left_buffer: Vec<f32>,
  right_buffer: Vec<f32>,
  context: AudioContext,
  proc_node: ScriptProcessorNode,
  delete: Closure<dyn FnMut(JsValue)>,
  
  _on_source: Closure<dyn FnMut(JsValue)>,
  _on_rejection: Closure<dyn FnMut(JsValue)>,
  _on_proc: Closure<dyn FnMut(AudioProcessingEvent)>,
}


impl BufferData<'_> {
  pub fn iter_mut(&mut self) -> impl Iterator<Item=Sample> {
    self.left.iter_mut().zip(self.right.iter_mut()).map(|(l,r)| Sample {
      left: l,
      right: r,
    })
  }
}


pub fn start_processing(args: Processor) -> Result<(),Error> {
  let processor = Rc::new(RefCell::new(None));
  
  let on_source = {
    let processor = processor.clone();
    Closure::new(move |source| {
      cleanup(&processor, |p| on_source(p,source));
    })
  };

  let on_rejection = {
    let processor = processor.clone();
    Closure::new(move |_| {
      cleanup(&processor, |_| Err(Error()));
    })
  };

  let on_proc = {
    let processor = processor.clone();
    Closure::new(move |event| {
      cleanup(&processor, |p| on_proc(p,event));
    })
  };

  let delete = {
    let processor = processor.clone();
    Closure::new(move |_| {
      processor.replace(None);
    })
  };
  
  let context = AudioContext::new()?;
  
  let proc_node = context
    .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
      args.buffer_size, 1, 2 )?;
  
  proc_node.set_onaudioprocess(Some(on_proc.as_ref().dyn_ref()?));
  proc_node.connect_with_audio_node(&context.destination())?;
  
  window()?
    .navigator()
    .media_devices()?
    .get_user_media_with_constraints(
      MediaStreamConstraints::new().audio(&true.into()) )?
    .then(&on_source)
    .catch(&on_rejection);

  *processor.borrow_mut() = Some( Processor_ {
    left_buffer: vec![0.0; args.buffer_size as usize],
    right_buffer: vec![0.0; args.buffer_size as usize],
    args: args,    
    context: context,
    proc_node: proc_node,
    delete: delete,    
    _on_source: on_source,
    _on_rejection: on_rejection,
    _on_proc: on_proc,
  });

  Ok(())
}

fn on_source( processor: &mut Processor_,
              source: JsValue ) -> Result<(),Error> {
  let source: MediaStream = source.dyn_into()?;
  let source_node = processor.context.create_media_stream_source(&source)?;
  source_node.connect_with_audio_node(&processor.proc_node)?;
  Ok(())
}

fn on_proc( processor: &mut Processor_,
            event: AudioProcessingEvent ) -> Result<(),Error> {
  
  let input_buffer = event.input_buffer()?;
  let output_buffer = event.output_buffer()?;
  
  input_buffer.copy_from_channel(&mut processor.left_buffer, 0)?;
  
  (processor.args.process)( Buffer {
    sample_rate: input_buffer.sample_rate(),
    data: &mut BufferData {
      left: &mut processor.left_buffer,
      right: &mut processor.right_buffer,
    }
  })?;
  
  output_buffer.copy_to_channel(&mut processor.left_buffer, 0)?;
  output_buffer.copy_to_channel(&mut processor.right_buffer, 1)?;
  
  Ok(())    
}

fn cleanup<F>(processor: &RefCell<Option<Processor_>>, f: F) where
  F: FnOnce(&mut Processor_) -> Result<(),Error> {
  if processor
    .borrow_mut()
    .as_mut()
    .and_then(|p| {
      match f(p) {
        Ok(_) => Some(()),
        Err(_) => match p.context.close() {
          Ok(close) => {
            close.then(&p.delete).catch(&p.delete);
            Some(())
          },
          Err(_) => None,
        }
      }
    }).is_none() {
      processor.replace(None);
    }
}
