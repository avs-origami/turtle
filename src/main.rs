extern crate libc;
extern crate x11;

use std::{fs, process};
use std::mem::zeroed;
use std::ffi::CStr;

use x11::xlib;

use turtle::{Config, RonConfig};

fn main() {
    let mut arg0 = 0x0_i8;
    let dpy: *mut xlib::Display = unsafe { xlib::XOpenDisplay(&mut arg0) };

    // let mut attr: xlib::XWindowAttributes = unsafe { zeroed() };
    let mut start: xlib::XButtonEvent = unsafe { zeroed() };

    if dpy.is_null() {
        process::exit(1);
    }

    let raw_config = &fs::read_to_string("/home/pineapple/.config/turtle/config.ron").expect("failed to read config");
    let config: Config = Config::from(ron::from_str::<RonConfig>(raw_config).unwrap());
    let keybinds = ron::from_str::<RonConfig>(raw_config).unwrap().keybinds;

    turtle::setup_async_keys(&dpy, &keybinds);

    start.subwindow = 0;

    let mut event: xlib::XEvent = unsafe { zeroed() };

    loop {
        unsafe {
            xlib::XNextEvent(dpy, &mut event);

            match event.get_type() {
                xlib::KeyPress => {
                    let xkey: xlib::XKeyEvent = From::from(event);
                    for keybind in &keybinds {
                        if keybind.1 == CStr::from_ptr(
                            xlib::XKeysymToString(
                                xlib::XKeycodeToKeysym(dpy, xkey.keycode as u8, 0)
                            )
                        ).to_str().unwrap() {
                            turtle::parse(&dpy, &xkey, &config, keybind);
                        }
                    }
                },

                xlib::ButtonRelease => {
                    start.subwindow = 0;
                },

                _ => {}
            };
        };
    }
}