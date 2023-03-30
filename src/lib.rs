use std::{process, ffi::CString};

use libc::c_int;
use x11::xlib::{self, Display, XKeyEvent};

pub fn quit(dpy: &*mut Display) {
    unsafe {
        eprintln!("Exiting turtle");
        xlib::XCloseDisplay(*dpy);
    }
}

pub fn kill_window(dpy: &*mut Display, xkey: &XKeyEvent) {
    unsafe {
        if xkey.subwindow != 0 { xlib::XKillClient(*dpy, xkey.subwindow); }
    }
}

pub fn spawn(com: &Vec<&str>) {
    let _ = process::Command::new(com[0]).args(&com[1..]).status();
}

pub fn parse(dpy: &*mut Display, xkey: &XKeyEvent, keybind: &(u32, &str, &str, Option<Vec<&str>>)) {
    match keybind.2 {
        "quit" => quit(dpy),
        "kill_window" => kill_window(dpy, xkey),
        "spawn" => spawn(keybind.3.as_ref().unwrap()),
        _ => ()
    }
}

pub fn setup_async_keys(dpy: &*mut Display, keybinds: &Vec<(u32, &str, &str, Option<Vec<&str>>)>) {
    for keybind in keybinds {
        let key = CString::new(keybind.1).unwrap();
        unsafe {
            xlib::XGrabKey(*dpy, xlib::XKeysymToKeycode(*dpy, xlib::XStringToKeysym(key.as_ptr())) as i32,
            keybind.0, xlib::XDefaultRootWindow(*dpy), true as c_int, xlib::GrabModeAsync, xlib::GrabModeAsync);
        };
    }
}