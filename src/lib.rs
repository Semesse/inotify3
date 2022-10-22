#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;
use futures_util::StreamExt;
use inotify::{Event, Inotify, WatchDescriptor, WatchMask};
use napi::bindgen_prelude::ToNapiValue;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::*;
use std::ffi::OsString;

enum CallbackArgType {
  String(JsString),
  Number(JsNumber),
  Null(JsNull),
}
impl ToNapiValue for CallbackArgType {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    match val {
      CallbackArgType::String(s) => JsString::to_napi_value(env, s),
      CallbackArgType::Number(s) => JsNumber::to_napi_value(env, s),
      CallbackArgType::Null(s) => JsNull::to_napi_value(env, s),
    }
  }
}

#[napi(js_name = "WatchDescriptor")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsWatchDescriptor {
  wd: WatchDescriptor,
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
  pub fn watch(&mut self, path: String, flags: u32) -> napi::Result<JsWatchDescriptor> {
    match self
      .inotify
      .add_watch(path.clone(), WatchMask::from_bits_truncate(flags))
    {
      Ok(wd) => Ok(JsWatchDescriptor { wd: wd }),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Failed to watch {}, {:?}", path, e),
      )),
    }
  }

  #[napi]
  pub fn unwatch(&mut self, descriptor: &JsWatchDescriptor) -> napi::Result<()> {
    let wd = descriptor.clone();
    match self.inotify.rm_watch(wd.wd) {
      Ok(_) => Ok(()),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Failed to unwatch {:?}", e),
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
        let null = ctx.env.get_null()?;
        let path = e.name.clone();
        let path = path
          .map(|s| s.to_str().map(|s| ctx.env.create_string(s).ok()))
          .flatten()
          .flatten();
        let path = match path {
          Some(path) => CallbackArgType::String(path),
          None => CallbackArgType::Null(null),
        };
        let mask = CallbackArgType::Number(ctx.env.create_uint32(e.mask.bits())?);
        let cookie = CallbackArgType::Number(ctx.env.create_uint32(e.cookie)?);
        Ok(vec![path, mask, cookie])
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
