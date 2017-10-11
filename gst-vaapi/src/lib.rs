// Copyright (C) 2017 Hyunjun Ko <zzoon@igalia.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_type = "cdylib"]

#[macro_use]
extern crate gst_plugin;
#[macro_use]
extern crate gstreamer as gst;
extern crate gstreamer_video as gst_video;
extern crate url;

extern crate libva_rust;
extern crate x11;

use gst_plugin::sink::*;

mod vaapisink;

use vaapisink::VAapiSink;

fn plugin_init(plugin: &gst::Plugin) -> bool {
    sink_register(
        plugin,
        SinkInfo {
            name: "rsvaapisink".into(),
            long_name: "VA-API Sink".into(),
            description: "A VA-API based videosink".into(),
            classification: "Sink/Video".into(),
            author: "Hyunjun Ko <zzoon@igalia.com>".into(),
            rank: 256 + 100,
            create_instance: VAapiSink::new_boxed,
            protocols: vec!["vaapi".into()],
        },
    );

    true
}

plugin_define!(
    b"rsvaapi\0",
    b"Rust VA-API Plugin\0",
    plugin_init,
    b"1.0\0",
    b"MIT/X11\0",
    b"rsvaapi\0",
    b"rsvaapi\0",
    b"https://github.com/sdroege/gst-plugin-rs\0",
    b"2017-09-27\0"
);
