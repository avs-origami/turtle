use std::{process, ffi::CString};

use libc::c_int;
use x11::xlib;

pub fn spawn(com: Vec<&str>) {
    let _ = process::Command::new(com[0]).args(&com[1..]).status();
}

pub fn setup_async_keys(dpy: &*mut xlib::Display) {
    let q = CString::new("q").unwrap();
    let w = CString::new("w").unwrap();
    let space = CString::new("space").unwrap();
    
    unsafe {
        xlib::XGrabKey(*dpy, xlib::XKeysymToKeycode(*dpy, xlib::XStringToKeysym(q.as_ptr())) as c_int, xlib::Mod4Mask|xlib::ShiftMask,
        xlib::XDefaultRootWindow(*dpy), true as c_int, xlib::GrabModeAsync, xlib::GrabModeAsync);

        xlib::XGrabKey(*dpy, xlib::XKeysymToKeycode(*dpy, xlib::XStringToKeysym(w.as_ptr())) as c_int, xlib::Mod4Mask,
        xlib::XDefaultRootWindow(*dpy), true as c_int, xlib::GrabModeAsync, xlib::GrabModeAsync);

        xlib::XGrabKey(*dpy, xlib::XKeysymToKeycode(*dpy, xlib::XStringToKeysym(space.as_ptr())) as c_int, xlib::Mod4Mask,
        xlib::XDefaultRootWindow(*dpy), true as c_int, xlib::GrabModeAsync, xlib::GrabModeAsync);
    };
}