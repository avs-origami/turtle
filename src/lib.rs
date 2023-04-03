use std::{process, ffi::CString};

use libc::{c_int, c_uint};
use moveslice::Moveslice;
use x11::xlib::{self, Display, XKeyEvent, XButtonEvent, XWindowAttributes, XEvent, XWindowChanges, Window};

/// Contains configuration for turtle
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RonConfig<'a> {
    pub top_gap: i32,
    pub edge_gap: i32,
    #[serde(borrow)]
    pub keybinds: Vec<(u32, &'a str, &'a str, Option<Vec<&'a str>>)>,
    #[serde(borrow)]
    pub mousebinds: Vec<(u32, u32, &'a str)>,
}

/// Config without keybinds or mousebinds
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

/// Exit turtle
pub fn quit(dpy: &*mut Display) {
    unsafe {
        eprintln!("Exiting turtle");
        xlib::XCloseDisplay(*dpy);
    }
}

/// Kill the focused window
pub fn kill_window(dpy: &*mut Display, xkey: &XKeyEvent) {
    unsafe {
        if xkey.subwindow != 0 { xlib::XKillClient(*dpy, xkey.subwindow); }
    }
}
 /// Spawn a program
pub fn spawn(com: &Vec<&str>) {
    let _ = process::Command::new(com[0]).args(&com[1..]).spawn();
}

/// Make a window fullscreen
pub fn maximize(dpy: &*mut Display, xkey: &XKeyEvent) {
    if xkey.subwindow != 0 {
        unsafe {
            let width = xlib::XDisplayWidth(*dpy, xlib::XDefaultScreen(*dpy));
            let height = xlib::XDisplayHeight(*dpy, xlib::XDefaultScreen(*dpy));
            xlib::XMoveResizeWindow(*dpy, xkey.subwindow, 0, 0, width as c_uint, height as c_uint);
        };
    }
}

/// Make a window big
pub fn stack(dpy: &*mut Display, xkey: &XKeyEvent, cfg: &Config) {
    if xkey.subwindow != 0 {
        unsafe {
            let width = xlib::XDisplayWidth(*dpy, xlib::XDefaultScreen(*dpy)) - 2 * cfg.edge_gap;
            let height = xlib::XDisplayHeight(*dpy, xlib::XDefaultScreen(*dpy)) - (cfg.top_gap + cfg.edge_gap);
            xlib::XMoveResizeWindow(*dpy, xkey.subwindow, cfg.edge_gap, cfg.top_gap, width as c_uint, height as c_uint);
        };
    }
}

/// Make a window small
pub fn shrink(dpy: &*mut Display, xkey: &XKeyEvent, cfg: &Config) {
    if xkey.subwindow != 0 {
        unsafe {
            let width = (xlib::XDisplayWidth(*dpy, xlib::XDefaultScreen(*dpy)) - 2 * cfg.edge_gap) / 4;
            let height = (xlib::XDisplayHeight(*dpy, xlib::XDefaultScreen(*dpy)) - (cfg.top_gap + cfg.edge_gap)) / 4;
            xlib::XMoveResizeWindow(*dpy, xkey.subwindow, cfg.edge_gap, cfg.top_gap, width as c_uint, height as c_uint);
        };
    }
}

/// Switch to the next window
pub fn next_window(dpy: &*mut Display, xkey: &XKeyEvent, windows: &mut Vec<u64>, root: Window) {
    if xkey.subwindow != 0 {
        unsafe { xlib::XCirculateSubwindowsDown(*dpy, root); };
        windows.moveslice((windows.len() - 1) ..= (windows.len() - 1), 0);
    }
}

/// Switch to the previous window
pub fn prev_window(dpy: &*mut Display, xkey: &XKeyEvent, windows: &mut Vec<u64>, root: Window) {
    if xkey.subwindow != 0 {
        unsafe { xlib::XCirculateSubwindowsDown(*dpy, root); };
        windows.moveslice(0 ..= 0, windows.len() - 1);
    }
}

/// Switch to the last used window
pub fn last_window(dpy: &*mut Display, xkey: &XKeyEvent, windows: &mut Vec<u64>) {
    if xkey.subwindow != 0 && windows.len() >= 2 {
        unsafe { xlib::XRaiseWindow(*dpy, windows[1]); };
        windows.moveslice(1 ..= 1, 0);
    }
}

/// Configure newly created windows
pub fn layout(dpy: &*mut Display, ev: xlib::XConfigureRequestEvent, cfg: &Config) {
    let mut changes = XWindowChanges { 
        x: cfg.edge_gap,
        y: cfg.top_gap,
        width: unsafe { (xlib::XDisplayWidth(*dpy, xlib::XDefaultScreen(*dpy)) - 2 * cfg.edge_gap) / 4 },
        height: unsafe { (xlib::XDisplayHeight(*dpy, xlib::XDefaultScreen(*dpy)) - (cfg.top_gap + cfg.edge_gap)) / 4 },
        border_width: ev.border_width,
        sibling: ev.above,
        stack_mode: ev.detail
    };

    unsafe { xlib::XConfigureWindow(*dpy, ev.window, ev.value_mask as u32, &mut changes); }
}
 /// Map newly created windows to the screen
pub fn map(dpy: &*mut Display, ev: xlib::XMapRequestEvent, cfg: &Config) {
    unsafe {
        xlib::XMapWindow(*dpy, ev.window);
        let width = xlib::XDisplayWidth(*dpy, xlib::XDefaultScreen(*dpy)) - 2 * cfg.edge_gap;
        let height = xlib::XDisplayHeight(*dpy, xlib::XDefaultScreen(*dpy)) - (cfg.top_gap + cfg.edge_gap);
        xlib::XMoveResizeWindow(*dpy, ev.window, cfg.edge_gap, cfg.top_gap, width as c_uint, height as c_uint);
    };
}

 /// Move a window
pub fn move_win(dpy: &*mut Display, event: XEvent, start: XButtonEvent, attr: XWindowAttributes) {
    if start.subwindow != 0 {
        let xbutton: xlib::XButtonEvent = From::from(event);
        let xdiff : i32 = xbutton.x_root - start.x_root;
        let ydiff : i32 = xbutton.y_root - start.y_root;
        unsafe {
            xlib::XMoveResizeWindow(*dpy, start.subwindow,
                attr.x + xdiff,
                attr.y + ydiff,
                std::cmp::max(1, attr.width) as u32,
                std::cmp::max(1, attr.height) as u32
            );
        }
    }
}

/// Resize a window
pub fn resize_win(dpy: &*mut Display, event: XEvent, start: XButtonEvent, attr: XWindowAttributes) {
    if start.subwindow != 0 {
        let xbutton: xlib::XButtonEvent = From::from(event);
        let xdiff : i32 = xbutton.x_root - start.x_root;
        let ydiff : i32 = xbutton.y_root - start.y_root;
        unsafe {
            xlib::XMoveResizeWindow(*dpy, start.subwindow,
                attr.x,
                attr.y,
                std::cmp::max(1, attr.width + xdiff) as u32,
                std::cmp::max(1, attr.height + ydiff) as u32
            );
        }
    }
}
 /// Parse keybinds
pub fn parse_keys(dpy: &*mut Display,
    xkey: &XKeyEvent,
    cfg: &Config,
    root: Window,
    windows: &mut Vec<u64>,
    keybind: &(u32, &str, &str, Option<Vec<&str>>)
) {
    match keybind.2 {
        "quit" => quit(dpy),
        "kill" => kill_window(dpy, xkey),
        "spawn" => spawn(keybind.3.as_ref().unwrap()),
        "maximize" => maximize(dpy, xkey),
        "stack" => stack(dpy, xkey, cfg),
        "shrink" => shrink(dpy, xkey, cfg),
        "next" => next_window(dpy, xkey, windows, root),
        "prev" => prev_window(dpy, xkey, windows, root),
        "last" => last_window(dpy, xkey, windows),
        _ => ()
    }
}

/// Parse mousebinds
pub fn parse_mouse(
    dpy: &*mut Display,
    event: XEvent,
    start: XButtonEvent,
    attr: XWindowAttributes,
    mousebind: &(u32, u32, &str)
) {
    match mousebind.2 {
        "move" => move_win(dpy, event, start, attr),
        "resize" => resize_win(dpy, event, start, attr),
        _ => (),
    }
}

/// Setup asynchronous capture of key- and mouse- binds
pub fn setup_async_keys(
    dpy: &*mut Display,
    keybinds: &Vec<(u32, &str, &str, Option<Vec<&str>>)>,
    mousebinds: &Vec<(u32, u32, &str)>
) {
    for keybind in keybinds {
        let key = CString::new(keybind.1).unwrap();
        unsafe {
            xlib::XGrabKey(*dpy, xlib::XKeysymToKeycode(*dpy, xlib::XStringToKeysym(key.as_ptr())) as i32,
            keybind.0, xlib::XDefaultRootWindow(*dpy), true as c_int, xlib::GrabModeAsync, xlib::GrabModeAsync);
        };
    }

    for mousebind in mousebinds {
        unsafe {
            xlib::XGrabButton(*dpy, mousebind.1, mousebind.0, xlib::XDefaultRootWindow(*dpy), true as c_int,
            (xlib::ButtonPressMask|xlib::ButtonReleaseMask|xlib::PointerMotionMask) as c_uint,
            xlib::GrabModeAsync, xlib::GrabModeAsync, 0, 0);
        };
    }
}