#[macro_use]
extern crate log;
extern crate loggerv;

extern crate synchro_servo;
extern crate synchro_glwindows;

use synchro_glwindows::GLWindow;
use synchro_servo::{Constellation, Compositor, View, Browser};
use synchro_servo::{BrowserVisibility, BrowserEvent, Cursor, ServoUrl, WindowEvent};


fn main() {
    loggerv::init_quiet().unwrap();

    let url = ServoUrl::parse("https://servo.org").unwrap();

    let win1 = GLWindow::new();

    let gl = win1.get_gl();
    let geometry = win1.get_geometry();
    let riser = win1.create_event_loop_riser();

    let constellation = Constellation::new().unwrap();
    let compositor = Compositor::new(&constellation, gl);

    let view = View::new(&compositor, geometry, riser);
    let browser = Browser::new(&constellation, url, view.clone());

    browser.set_visibility(BrowserVisibility::Visible);

    synchro_glwindows::run(|event, window_id| {
        match event {
            WindowEvent::Idle => {
                // This means the event loop as been awaken by Servo, via the EventLoopRiser.
                // Let's go through the Servo events.

                // Because of https://github.com/servo/servo/issues/15934
                // events come from the view, not the browser.
                let browser_events = (*view).get_events();
                for e in browser_events {
                    match e {
                        BrowserEvent::Present => {
                            win1.swap_buffers();
                        }
                        BrowserEvent::CursorChanged(cursor) => {
                            win1.set_cursor(cursor);
                        }
                        BrowserEvent::StatusChanged(_) => {}
                        BrowserEvent::TitleChanged(title) => {
                            win1.set_title(&title.unwrap_or("No Title".to_owned()));
                        }
                        e => {
                            warn!("Unhandled browser event: {:?}", e);
                        }
                    }
                }
                // By waking up the main thread, it's also expected to let Servo
                // run its compositor tasks.
                // See https://github.com/servo/servo/issues/15734
                browser.perform_updates();
            }
            win_event => {
                // Forward any event directly to Servo.
                // It's also a good place to intercept early key bindings.
                browser.handle_event(win_event);
            }
        }
    });
}
