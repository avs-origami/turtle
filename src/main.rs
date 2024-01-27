extern crate libc;
extern crate x11;

use std::io::Write;
use std::{env, fs, process};
use std::mem::zeroed;
use std::ffi::CStr;

use moveslice::Moveslice;
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
    let home = env::var("HOME").unwrap();
    let raw_config = &fs::read_to_string(format!("{}/.config/turtle/config.ron", home)).expect("failed to read config");
    let config: Config = Config::from(ron::from_str::<RonConfig>(raw_config).unwrap());
    let keybinds = ron::from_str::<RonConfig>(raw_config).unwrap().keybinds;
    let mousebinds = ron::from_str::<RonConfig>(raw_config).unwrap().mousebinds;

    // Run autostart script
    let _ = process::Command::new(format!("{}/.config/turtle/autostart", home)).spawn();

    turtle::setup_async_keys(&dpy, &keybinds, &mousebinds);

    start.subwindow = 0;

    let mut event: xlib::XEvent = unsafe { zeroed() };

    // List of open windows
    let mut windows: Vec<u64> = Vec::new();

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
                            turtle::parse_keys(&dpy, &xkey, &config, root, &mut windows, keybind);
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

                xlib::CreateNotify => {
                    let _ev: xlib::XCreateWindowEvent = From::from(event);
                }

                xlib::ConfigureRequest => {
                    let ev: xlib::XConfigureRequestEvent = From::from(event);
                    turtle::layout(&dpy, ev, &config);
                },

                xlib::MapRequest => {
                    let ev: xlib::XMapRequestEvent = From::from(event);
                    turtle::map(&dpy, ev, &config);
                    windows.push(ev.window);
                    windows.moveslice((windows.len() - 1) ..= (windows.len() - 1), 0);
                },

                xlib::DestroyNotify => {
                    let ev: xlib::XCreateWindowEvent = From::from(event);
                    if windows.contains(&ev.window) {
                        let idx = windows.iter().position(|&r| r == ev.window).unwrap();
                        windows.remove(idx);
                    }
                },

                xlib::ButtonRelease => {
                    start.subwindow = 0;
                },

                _ => {}
            };
        };

        let fname = format!("{}/.config/turtle/info.txt", home);
        let mut info_file = fs::File::create(fname).expect("Failed to create ~/.config/turtle/info.txt:");
        if windows.len() > 0 {
            info_file.write_all(
                format!("focused:{}\n", windows[0]).as_bytes()
            ).expect("Failed to write to ~/.config/turtle/info.txt:");
        } else {
            info_file.write_all(
                b"focused: \n"
            ).expect("Failed to write to ~/.config/turtle/info.txt:");
        }
    }
}
