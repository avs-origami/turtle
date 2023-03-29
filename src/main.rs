extern crate libc;
extern crate x11;

use std::process;
use std::mem::zeroed;

use x11::xlib;

fn main() {
    let mut arg0 = 0x0_i8;
    let dpy: *mut xlib::Display = unsafe { xlib::XOpenDisplay(&mut arg0) };

    // let mut attr: xlib::XWindowAttributes = unsafe { zeroed() };
    let mut start: xlib::XButtonEvent = unsafe { zeroed() };

    if dpy.is_null() {
        process::exit(1);
    }

    turtle::setup_async_keys(&dpy);

    start.subwindow = 0;

    let mut event: xlib::XEvent = unsafe { zeroed() };

    loop {
        unsafe {
            xlib::XNextEvent(dpy, &mut event);

            match event.get_type() {
                xlib::KeyPress => {
                    let xkey: xlib::XKeyEvent = From::from(event);
                    match xkey.keycode {
                        24 => {
                            eprintln!("Exiting turtle");
                            xlib::XCloseDisplay(dpy);
                        },

                        25 => {
                            if xkey.subwindow != 0 { xlib::XKillClient(dpy, xkey.subwindow); }
                        },

                        65 => turtle::spawn(vec!["dmenu_run"]),

                        _ => eprintln!("Pressed key {}", xkey.keycode)
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