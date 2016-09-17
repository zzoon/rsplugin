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

use error::*;
use rsdemuxer::*;
use buffer::*;

#[derive(Debug)]
pub struct SillyDemux {

}

impl SillyDemux {
    pub fn new() -> SillyDemux {
        SillyDemux {}
    }

    pub fn new_boxed() -> Box<Demuxer> {
        Box::new(SillyDemux::new())
    }
}

impl Demuxer for SillyDemux {
    fn start(&mut self,
             upstream_size: Option<u64>,
             random_access: bool)
             -> Result<(), ErrorMessage> {
        unimplemented!();
    }

    fn stop(&mut self) -> Result<(), ErrorMessage> {
        unimplemented!();
    }

    fn seek(&mut self, start: u64, stop: Option<u64>) -> Result<SeekResult, ErrorMessage> {
        unimplemented!();
    }

    fn handle_buffer(&mut self, buffer: Option<Buffer>) -> Result<HandleBufferResult, FlowError> {
        unimplemented!();
    }

    fn end_of_stream(&mut self) -> Result<(), ErrorMessage> {
        unimplemented!();
    }

    fn is_seekable(&self) -> bool {
        unimplemented!();
    }

    fn get_position(&self) -> Option<u64> {
        unimplemented!();
    }

    fn get_duration(&self) -> Option<u64> {
        unimplemented!();
    }

    fn get_streams(&self) -> &Vec<Stream> {
        unimplemented!();
    }
}
