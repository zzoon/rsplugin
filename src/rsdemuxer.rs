//  Copyright (C) 2016 Sebastian Dröge <sebastian@centricular.com>
//
//  This library is free software; you can redistribute it and/or
//  modify it under the terms of the GNU Library General Public
//  License as published by the Free Software Foundation; either
//  version 2 of the License, or (at your option) any later version.
//
//  This library is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  Library General Public License for more details.
//
//  You should have received a copy of the GNU Library General Public
//  License along with this library; if not, write to the
//  Free Software Foundation, Inc., 51 Franklin St, Fifth Floor,
//  Boston, MA 02110-1301, USA.

use libc::c_char;
use std::os::raw::c_void;
use std::ffi::CString;

use std::panic::{self, AssertUnwindSafe};

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use std::u64;

use utils::*;
use error::*;
use buffer::*;

pub type StreamIndex = u32;

#[derive(Debug)]
pub enum SeekResult {
    TooEarly,
    Ok(u64),
    Eos,
}

#[derive(Debug)]
pub enum HandleBufferResult {
    NeedMoreData,
    // NeedDataFromOffset(u64),
    StreamsChanged,
    StreamChanged(StreamIndex),
    HaveBufferForStream(StreamIndex, Buffer),
    Eos(StreamIndex),
    AllEos,
}

pub trait Demuxer {
    fn start(&mut self,
             upstream_size: Option<u64>,
             random_access: bool)
             -> Result<(), ErrorMessage>;
    fn stop(&mut self) -> Result<(), ErrorMessage>;

    fn seek(&mut self, start: u64, stop: Option<u64>) -> Result<SeekResult, ErrorMessage>;
    fn handle_buffer(&mut self, buffer: Option<Buffer>) -> Result<HandleBufferResult, FlowError>;
    fn end_of_stream(&mut self) -> Result<(), ErrorMessage>;

    fn is_seekable(&self) -> bool;
    fn get_position(&self) -> Option<u64>;
    fn get_duration(&self) -> Option<u64>;

    fn get_streams(&self) -> &Vec<Stream>;
}

pub struct Stream {
    pub index: StreamIndex,
    pub format: String,
    pub stream_id: String,
}

impl Stream {
    pub fn new(index: StreamIndex, format: String, stream_id: String) -> Stream {
        Stream {
            index: index,
            format: format,
            stream_id: stream_id,
        }
    }
}

pub struct DemuxerWrapper {
    raw: *mut c_void,
    demuxer: Mutex<Box<Demuxer>>,
    panicked: AtomicBool,
}

impl DemuxerWrapper {
    fn new(raw: *mut c_void, demuxer: Box<Demuxer>) -> DemuxerWrapper {
        DemuxerWrapper {
            raw: raw,
            demuxer: Mutex::new(demuxer),
            panicked: AtomicBool::new(false),
        }
    }
}

#[no_mangle]
pub extern "C" fn demuxer_new(demuxer: *mut c_void,
                              create_instance: fn() -> Box<Demuxer>)
                              -> *mut DemuxerWrapper {
    Box::into_raw(Box::new(DemuxerWrapper::new(demuxer, create_instance())))
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_drop(ptr: *mut DemuxerWrapper) {
    Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_start(ptr: *const DemuxerWrapper,
                                       upstream_size: u64,
                                       random_access: GBoolean)
                                       -> GBoolean {
    let wrap: &DemuxerWrapper = &*ptr;

    panic_to_error!(wrap, GBoolean::False, {
        let demuxer = &mut wrap.demuxer.lock().unwrap();

        let upstream_size = if upstream_size == u64::MAX {
            None
        } else {
            Some(upstream_size)
        };

        match demuxer.start(upstream_size, random_access.to_bool()) {
            Ok(..) => GBoolean::True,
            Err(ref msg) => {
                msg.post(wrap.raw);
                GBoolean::False
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_stop(ptr: *const DemuxerWrapper) -> GBoolean {
    let wrap: &DemuxerWrapper = &*ptr;

    panic_to_error!(wrap, GBoolean::False, {
        let demuxer = &mut wrap.demuxer.lock().unwrap();

        match demuxer.stop() {
            Ok(..) => GBoolean::True,
            Err(ref msg) => {
                msg.post(wrap.raw);
                GBoolean::False
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_is_seekable(ptr: *const DemuxerWrapper) -> GBoolean {
    let wrap: &DemuxerWrapper = &*ptr;

    panic_to_error!(wrap, GBoolean::False, {
        let demuxer = &wrap.demuxer.lock().unwrap();

        GBoolean::from_bool(demuxer.is_seekable())
    })
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_get_position(ptr: *const DemuxerWrapper,
                                              position: *mut u64)
                                              -> GBoolean {
    let wrap: &DemuxerWrapper = &*ptr;

    panic_to_error!(wrap, GBoolean::False, {
        let demuxer = &wrap.demuxer.lock().unwrap();

        match demuxer.get_position() {
            None => {
                *position = u64::MAX;
                GBoolean::False
            }
            Some(pos) => {
                *position = pos;
                GBoolean::True
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_get_duration(ptr: *const DemuxerWrapper,
                                              duration: *mut u64)
                                              -> GBoolean {
    let wrap: &DemuxerWrapper = &*ptr;

    panic_to_error!(wrap, GBoolean::False, {
        let demuxer = &wrap.demuxer.lock().unwrap();

        match demuxer.get_duration() {
            None => {
                *duration = u64::MAX;
                GBoolean::False
            }
            Some(dur) => {
                *duration = dur;
                GBoolean::True
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_seek(ptr: *mut DemuxerWrapper,
                                      start: u64,
                                      stop: u64,
                                      offset: *mut u64)
                                      -> GBoolean {
    extern "C" {
        fn gst_rs_demuxer_stream_eos(raw: *mut c_void, index: u32);
    };

    let wrap: &mut DemuxerWrapper = &mut *ptr;

    panic_to_error!(wrap, GBoolean::False, {
        let res = {
            let mut demuxer = &mut wrap.demuxer.lock().unwrap();

            let stop = if stop == u64::MAX { None } else { Some(stop) };

            match demuxer.seek(start, stop) {
                Ok(res) => res,
                Err(ref msg) => {
                    msg.post(wrap.raw);
                    return GBoolean::False;
                }
            }
        };

        match res {
            SeekResult::TooEarly => GBoolean::False,
            SeekResult::Ok(off) => {
                *offset = off;
                GBoolean::True
            }
            SeekResult::Eos => {
                *offset = u64::MAX;

                let indices: Vec<u32> = {
                    let demuxer = &wrap.demuxer.lock().unwrap();
                    demuxer.get_streams().iter().map(|ref stream| stream.index).collect()
                };
                for index in indices {
                    gst_rs_demuxer_stream_eos(wrap.raw, index);
                }

                GBoolean::True
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_handle_buffer(ptr: *mut DemuxerWrapper,
                                               buffer: *mut c_void)
                                               -> GstFlowReturn {
    extern "C" {
        fn gst_rs_demuxer_stream_eos(raw: *mut c_void, index: u32);
        fn gst_rs_demuxer_add_stream(raw: *mut c_void,
                                     index: u32,
                                     format: *const c_char,
                                     stream_id: *const c_char);
        fn gst_rs_demuxer_added_all_streams(raw: *mut c_void);
        fn gst_rs_demuxer_remove_all_streams(raw: *mut c_void);
        fn gst_rs_demuxer_stream_format_changed(raw: *mut c_void,
                                                index: u32,
                                                format: *const c_char);
        fn gst_rs_demuxer_stream_push_buffer(raw: *mut c_void,
                                             index: u32,
                                             buffer: *mut c_void)
                                             -> GstFlowReturn;
    };

    let wrap: &mut DemuxerWrapper = &mut *ptr;

    panic_to_error!(wrap, GstFlowReturn::Error, {

        let mut res = {
            let mut demuxer = &mut wrap.demuxer.lock().unwrap();
            let buffer = Buffer::new_from_ptr_owned(buffer);

            match demuxer.handle_buffer(Some(buffer)) {
                Ok(res) => res,
                Err(flow_error) => {
                    match flow_error {
                        FlowError::NotNegotiated(ref msg) |
                        FlowError::Error(ref msg) => msg.post(wrap.raw),
                        _ => (),
                    }
                    return flow_error.to_native();
                }
            }
        };

        // Loop until AllEos, NeedMoreData or error when pushing downstream
        loop {
            match res {
                HandleBufferResult::NeedMoreData => {
                    return GstFlowReturn::Ok;
                }
                HandleBufferResult::StreamsChanged => {
                    gst_rs_demuxer_remove_all_streams(wrap.raw);

                    let streams: Vec<(u32, CString, CString)> = {
                        let demuxer = &wrap.demuxer.lock().unwrap();
                        demuxer.get_streams()
                            .iter()
                            .map(|ref stream| {
                                (stream.index,
                                 CString::new(stream.format.as_bytes()).unwrap(),
                                 CString::new(stream.stream_id.as_bytes()).unwrap())
                            })
                            .collect()
                    };

                    for (index, format_cstr, stream_id_cstr) in streams {
                        let format_ptr = format_cstr.as_ptr();
                        let stream_id_ptr = stream_id_cstr.as_ptr();
                        gst_rs_demuxer_add_stream(wrap.raw, index, format_ptr, stream_id_ptr);
                    }
                    gst_rs_demuxer_added_all_streams(wrap.raw);
                }
                HandleBufferResult::StreamChanged(index) => {
                    let format_cstr = {
                        let demuxer = &wrap.demuxer.lock().unwrap();
                        let format = &demuxer.get_streams()[index as usize].format;

                        CString::new(format.as_bytes()).unwrap()
                    };

                    let format_ptr = format_cstr.as_ptr();

                    gst_rs_demuxer_stream_format_changed(wrap.raw, index, format_ptr);
                }
                HandleBufferResult::HaveBufferForStream(index, buffer) => {
                    let flow_ret =
                        gst_rs_demuxer_stream_push_buffer(wrap.raw, index, buffer.into_ptr());
                    if flow_ret != GstFlowReturn::Ok {
                        return flow_ret;
                    }
                }
                HandleBufferResult::Eos(index) => {
                    gst_rs_demuxer_stream_eos(wrap.raw, index);
                }
                HandleBufferResult::AllEos => {
                    let indices: Vec<u32> = {
                        let demuxer = &wrap.demuxer.lock().unwrap();
                        demuxer.get_streams().iter().map(|ref stream| stream.index).collect()
                    };
                    for index in indices {
                        gst_rs_demuxer_stream_eos(wrap.raw, index);
                    }

                    return GstFlowReturn::Eos;
                }
            };

            res = {
                let mut demuxer = &mut wrap.demuxer.lock().unwrap();
                match demuxer.handle_buffer(None) {
                    Ok(res) => res,
                    Err(flow_error) => {
                        match flow_error {
                            FlowError::NotNegotiated(ref msg) |
                            FlowError::Error(ref msg) => msg.post(wrap.raw),
                            _ => (),
                        }
                        return flow_error.to_native();
                    }
                }
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn demuxer_end_of_stream(ptr: *mut DemuxerWrapper) {
    let wrap: &mut DemuxerWrapper = &mut *ptr;

    panic_to_error!(wrap, (), {
        let mut demuxer = &mut wrap.demuxer.lock().unwrap();

        match demuxer.end_of_stream() {
            Ok(_) => (),
            Err(ref msg) => {
                msg.post(wrap.raw);
            }
        }
    })
}
