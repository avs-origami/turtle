use std::{process, ffi::CString};

use libc::{c_int, c_uint};
use x11::xlib::{self, Display, XKeyEvent, Window};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RonConfig<'a> {
    pub top_gap: i32,
    pub edge_gap: i32,
    #[serde(borrow)]
    pub keybinds: Vec<(u32, &'a str, &'a str, Option<Vec<&'a str>>)>,
}

pub struct Config {
    pub top_gap: i32,
    pub edge_gap: i32,
}

impl From<RonConfig<'_>> for Config {
    fn from(item: RonConfig) -> Self {
        Config {
            top_gap: item.top_gap,
            edge_gap: item.edge_gap,
        }
    }
}

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

pub fn maximize(dpy: &*mut Display, win: Window) {
    unsafe {
        let width = xlib::XDisplayWidth(*dpy, xlib::XDefaultScreen(*dpy));
        let height = xlib::XDisplayHeight(*dpy, xlib::XDefaultScreen(*dpy));
        xlib::XMoveResizeWindow(*dpy, win, 0, 0, width as c_uint, height as c_uint);
    };
}

pub fn stack(dpy: &*mut Display, win: Window, cfg: &Config) {
    unsafe {
        let width = xlib::XDisplayWidth(*dpy, xlib::XDefaultScreen(*dpy)) - 2 * cfg.edge_gap;
        let height = xlib::XDisplayHeight(*dpy, xlib::XDefaultScreen(*dpy)) - (cfg.top_gap + cfg.edge_gap);
        xlib::XMoveResizeWindow(*dpy, win, cfg.edge_gap, cfg.top_gap, width as c_uint, height as c_uint);
    };
}

pub fn parse(dpy: &*mut Display, xkey: &XKeyEvent, cfg: &Config, keybind: &(u32, &str, &str, Option<Vec<&str>>)) {
    match keybind.2 {
        "quit" => quit(dpy),
        "kill_window" => kill_window(dpy, xkey),
        "spawn" => spawn(keybind.3.as_ref().unwrap()),
        "maximize" => maximize(dpy, xkey.subwindow),
        "stack" => stack(dpy, xkey.subwindow, cfg),
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