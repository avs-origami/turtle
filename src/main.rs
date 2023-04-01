extern crate libc;
extern crate x11;

use std::{fs, process};
use std::mem::zeroed;
use std::ffi::CStr;

use x11::xlib;

use turtle::{Config, RonConfig};

fn main() {
    let mut arg0 = 0x0_i8;

    // Open the display and get the root window
    let dpy: *mut xlib::Display = unsafe { xlib::XOpenDisplay(&mut arg0) };
    let root = unsafe { xlib::XDefaultRootWindow(dpy) };

    // Select events to be reported
    unsafe {
        xlib::XSelectInput(dpy, root, xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask);
    }

    let mut attr: xlib::XWindowAttributes = unsafe { zeroed() };
    let mut start: xlib::XButtonEvent = unsafe { zeroed() };

    if dpy.is_null() {
        process::exit(1);
    }

    // Load ~/.config/turtle/config.ron
    let raw_config = &fs::read_to_string("/home/pineapple/.config/turtle/config.ron").expect("failed to read config");
    let config: Config = Config::from(ron::from_str::<RonConfig>(raw_config).unwrap());
    let keybinds = ron::from_str::<RonConfig>(raw_config).unwrap().keybinds;
    let mousebinds = ron::from_str::<RonConfig>(raw_config).unwrap().mousebinds;

    turtle::setup_async_keys(&dpy, &keybinds, &mousebinds);

    start.subwindow = 0;

    let mut event: xlib::XEvent = unsafe { zeroed() };

    // List of open windows
    let mut windows: Vec<(u64, bool)> = Vec::new();
    let mut current_window = 0;

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
                            turtle::parse_keys(&dpy, &xkey, &config, keybind);
                        }
                    }
                },

                xlib::ButtonPress => {
                    let xbutton: xlib::XButtonEvent = From::from(event);
                    if xbutton.subwindow != 0 {
                        xlib::XGetWindowAttributes(dpy, xbutton.subwindow, &mut attr);
                        start = xbutton;
                    }
                },

                xlib::MotionNotify => {
                    for mousebind in &mousebinds {
                        if mousebind.1 == start.button {
                            turtle::parse_mouse(&dpy, event, start, attr, &mousebind);
                        }
                    }
                },

                xlib::ConfigureRequest => {
                    let ev: xlib::XConfigureRequestEvent = From::from(event);
                    turtle::layout(&dpy, ev, &config);
                },

                xlib::MapRequest => {
                    let ev: xlib::XMapRequestEvent = From::from(event);
                    turtle::map(&dpy, ev, &config);
                    windows.push((ev.window, true));
                    windows[current_window].1 = false;
                    current_window = windows.len() - 1;
                }

                xlib::DestroyNotify => {
                    let ev: xlib::XDestroyWindowEvent = From::from(event);
                    let mut index: Option<usize> = None;
                    
                    for (i, (window, _)) in windows.iter().enumerate() {
                        if *window == ev.window {
                            index = Some(i);
                            break;
                        }
                    }
                    
                    if let Some(i) = index {
                        windows.remove(i);
                    }
                }

                xlib::ButtonRelease => {
                    start.subwindow = 0;
                },

                _ => {}
            };
        };
    }
}