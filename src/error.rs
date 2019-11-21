use std::option::NoneError;

use wasm_bindgen::JsValue;

use web_sys::Element;

use js_sys::Object;


#[derive(Debug, PartialEq, Eq)]
pub struct Error();

impl From<NoneError> for Error {
  fn from(_: NoneError) -> Error { Error() }
}
impl From<Element> for Error {
  fn from(_: Element) -> Error { Error() }
}
impl From<JsValue> for Error {
  fn from(_: JsValue) -> Error { Error() }
}
impl From<Object> for Error {
  fn from(_: Object) -> Error { Error() }
}
