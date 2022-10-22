#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;
use futures_util::StreamExt;
use inotify::{Event, Inotify, WatchMask};
use napi::bindgen_prelude::ToNapiValue;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Error, *};
use std::ffi::OsString;

enum CallbackArgType {
  String(JsString),
  Number(JsNumber),
}
impl ToNapiValue for CallbackArgType {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    match val {
      CallbackArgType::String(s) => JsString::to_napi_value(env, s),
      CallbackArgType::Number(s) => JsNumber::to_napi_value(env, s),
    }
  }
}

#[napi(js_name = "Inotify")]
pub struct JsInotify {
  buf: [u8; 4096],
  inotify: Inotify,
}

#[napi]
impl JsInotify {
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let ino = Inotify::init();
    match ino {
      Ok(inotify) => Ok(JsInotify {
        buf: [0u8; 4096],
        inotify: inotify,
      }),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("failed to read file, {}", e),
      )),
    }
  }

  #[napi]
  pub fn watch(&mut self, path: String, flags: u32) -> napi::Result<()> {
    match self
      .inotify
      .add_watch(path.clone(), WatchMask::from_bits_truncate(flags))
    {
      Ok(_) => Ok(()),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Failed to add {}, {}", path, e),
      )),
    }
  }

  #[napi(
    ts_args_type = "callback: (err: Error | null, path: string, event: number, cookie: number) => void"
  )]
  pub fn on(&mut self, callback: JsFunction) -> Result<()> {
    let stream = self.inotify.event_stream(self.buf);
    let tscb: ThreadsafeFunction<Event<OsString>, ErrorStrategy::CalleeHandled> = callback
      .create_threadsafe_function(0, |ctx| {
        let e: Event<OsString> = ctx.value;
        let path = e.name.clone().ok_or(Error::new(
          Status::GenericFailure,
          format!("failed to convert event path to string {:?}", e.clone()),
        ))?;
        let path = path.to_str().ok_or(Error::new(
          Status::GenericFailure,
          format!("failed to convert path to string {:?}", e.clone()),
        ))?;
        let path = ctx.env.create_string(path)?;
        let mask = ctx.env.create_uint32(e.mask.bits())?;
        let cookie = ctx.env.create_uint32(e.cookie)?;
        Ok(vec![
          CallbackArgType::String(path),
          CallbackArgType::Number(mask),
          CallbackArgType::Number(cookie),
        ])
      })?;
    match stream {
      Ok(mut stream) => {
        tokio::spawn(async move {
          loop {
            match stream.next().await {
              Some(Ok(e)) => tscb.call(Ok(e), ThreadsafeFunctionCallMode::Blocking),
              e => tscb.call(
                Err(Error::new(Status::GenericFailure, format!("{:?}", e))),
                ThreadsafeFunctionCallMode::Blocking,
              ),
            };
          }
        });
        Ok(())
      }
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("failed to listen changes {:?}", e),
      )),
    }
  }
}
