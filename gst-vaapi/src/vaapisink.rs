// Copyright (C) 2017 Hyunjun Ko <zzoon@igalia.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use url::Url;
use std::convert::From;

use gst_plugin::error::*;
use gst_plugin::sink::*;
use gst;
use gst_video;

use libva_rust::va::*;
use libva_rust::renderer::*;
use libva_rust::renderer_x11::*;

use std::ptr;
use x11::xlib::XOpenDisplay;

#[derive(Debug)]
enum StreamingState {
    Stopped,
    Started,
}

#[derive(Debug)]
pub struct VAapiSink  {
    va_renderer: Box<VARenderer>,
    streaming_state: StreamingState,
    cat: gst::DebugCategory,
}

unsafe impl Send for VAapiSink {}

impl VAapiSink {
    pub fn new(_sink: &RsBaseSink) -> VAapiSink {
        let native_display;

        unsafe {
           native_display = XOpenDisplay(ptr::null());
        }

        let va_disp = VADisplay::initialize(native_display as *mut VANativeDisplay).unwrap();
        let va_renderer = VARendererX11::new(va_disp, 0, 0).unwrap();

        VAapiSink {
            va_renderer: va_renderer,
            streaming_state: StreamingState::Stopped,
            cat: gst::DebugCategory::new(
                "rsvaapisink",
                gst::DebugColorFlags::empty(),
                "Rust VA-API sink",
            ),
        }
    }

    pub fn new_boxed(sink: &RsBaseSink) -> Box<SinkImpl> {
        Box::new(VAapiSink::new(sink))
    }
}

#[allow(unused_variables)]
fn validate_uri(uri: &Url) -> Result<(), UriError> {
    Ok(())
}

#[allow(unused_variables)]
impl SinkImpl for VAapiSink {
    fn uri_validator(&self) -> Box<UriValidator> {
        Box::new(validate_uri)
    }

    fn set_caps (&mut self, sink: &RsBaseSink, caps: &gst::CapsRef) -> Result<(), bool> {
        let info = gst_video::VideoInfo::from_caps(&caps.to_owned()).unwrap();

        self.va_renderer.set_resolution(info.width(), info.height());
        self.va_renderer.open();
        Ok(())
    }

    fn start(&mut self, sink: &RsBaseSink, uri: Url) -> Result<(), ErrorMessage> {
        if let StreamingState::Started { .. } = self.streaming_state {
            return Err(error_msg!(
                gst::LibraryError::Failed,
                ["Sink already started"]
            ));
        }

        self.streaming_state = StreamingState::Started;
        Ok(())
    }

    fn stop(&mut self, _sink: &RsBaseSink) -> Result<(), ErrorMessage> {
        self.streaming_state = StreamingState::Stopped;
        self.va_renderer.close();

        Ok(())
    }

    fn render(&mut self, sink: &RsBaseSink, buffer: &gst::BufferRef) -> Result<(), FlowError> {
        let cat = self.cat;

        gst_trace!(cat, obj: sink, "Rendering {:?}", buffer);

        let map = match buffer.map_readable() {
            None => {
                return Err(FlowError::Error(error_msg!(
                    gst::LibraryError::Failed,
                    ["Failed to map buffer"]
                )));
            }
            Some(map) => map,
        };
        let data = map.as_slice();

        self.va_renderer.render(&data, data.len());

        Ok(())
    }
}
