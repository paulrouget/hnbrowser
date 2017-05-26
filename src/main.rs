#![feature(box_syntax)]

#[macro_use]
extern crate log;
extern crate loggerv;

extern crate synchro_servo;
extern crate synchro_glwindows;

use std::rc::Rc;
use std::collections::HashMap;

use synchro_glwindows::{GLWindow, GLWindowId};
use synchro_servo::{Constellation, Compositor, View, Browser};
use synchro_servo::{BrowserEvent, ServoUrl, servo_version, WindowEvent};

fn create_window(windows: &mut HashMap<GLWindowId, (Rc<GLWindow>, Browser, Rc<View>)>, constellation: &Constellation, url: ServoUrl) {
    let win = Rc::new(GLWindow::new());
    let gl = win.get_gl();
    let geometry = win.get_geometry();
    let riser = win.create_event_loop_riser();
    let compositor = Compositor::new(&constellation, gl);
    let view = Rc::new(View::new(&compositor, geometry, riser, win.clone()));
    let browser = Browser::new(constellation, url, view.clone());
    windows.insert(win.id(), (win.clone(), browser, view));
}

fn main() {
    loggerv::init_quiet().unwrap();

    let mut windows = HashMap::new();

    info!("Servo version: {}", servo_version());

    let constellation = Constellation::new().unwrap();

    let url = ServoUrl::parse("https://servo.org").unwrap();
    create_window(&mut windows, &constellation, url);
    let url = ServoUrl::parse("http://example.com").unwrap();
    create_window(&mut windows, &constellation, url);

    synchro_glwindows::run(|event, win_id| {
        match (event, win_id) {

            (WindowEvent::Idle, None) => {
                for &(ref window, ref browser, ref view) in windows.values() {
                    perform_updates(window, browser, view);
                }
            }
            (e, None) => {
                warn!("Got unexpected window-less window event: {:?}", e);
            },
            (e, Some(id)) => {
                let &(_, ref browser, _) = windows.get(&id).unwrap();
                browser.handle_event(e);
            }
        }
    });
}


fn perform_updates(window: &Rc<GLWindow>, browser: &Browser, view: &Rc<View>) {
    // Because of https://github.com/servo/servo/issues/15934
    // events come from the view, not the browser.
    let browser_events = (**view).get_events();
    for e in browser_events {
        match e {
            BrowserEvent::CursorChanged(cursor) => {
                window.set_cursor(cursor);
            }
            BrowserEvent::TitleChanged(title) => {
                window.set_title(&title.unwrap_or("No Title".to_owned()));
            }
            e => {
                warn!("Unhandled browser event: {:?}", e);
            }
        }
    }
    browser.perform_updates();
}
