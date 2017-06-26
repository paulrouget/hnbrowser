extern crate open;
extern crate servoapi;
extern crate servoglwindows;

use std::process;
use std::rc::Rc;

use servoglwindows::GLWindow;

use servoapi::Constellation;
use servoapi::{BrowserEvent, ServoUrl, WindowEvent, WindowNavigateMsg};
use servoapi::{Key, SUPER};

fn main() {
    let url = ServoUrl::parse("https://news.ycombinator.com").unwrap();
    let original_domain = url.domain().unwrap().to_owned();

    // The embedder needs to provide 2 pieces:
    // gl buffer + mean to wakeup the event loop

    // window implements GLMethods
    let window = Rc::new(GLWindow::new(800, 600));

    // waker implements EventLoopWaker
    let waker = window.create_event_loop_waker();

    // The embedder creates a constellation, a compositor, a view and a browser.
    // These are the minimal setup for a browser.

    // One global constellation.
    let constellation = Constellation::new().unwrap();
    let geometry = window.get_geometry();
    // One compositor per native window.
    let compositor = constellation.new_compositor(window.clone(), waker, geometry /*temporary - should go to view*/);
    // We can have multiple view per compositor.
    let view = compositor.new_view(geometry);
    // We can have multiple browser per constellation.
    let browser_id = constellation.new_browser(url, &compositor /*temporary*/).unwrap();

    // A browser is either offscreen or rendered in a view.
    view.show(Some(browser_id));

    window.set_title("Loading");

    // The main event loop.

    servoglwindows::run(|event, win_id| {
        match (event, win_id) {

            (e, Some(_window_id)) => {
                // Got an event from the OS. Mouse event, keyboard events, etc.
                // They can be sent to servo, or maybe the embedder wants to handle
                // some directly. Here, we intercept 3 of them.
                //
                // Note:
                // As for now, events are sent to compositor. In the future, it will
                // be browser, constellation or compositor. See #15934
                match e {
                    WindowEvent::KeyEvent(_, Key::Left, _, SUPER) => {
                        compositor.handle_event(WindowEvent::Navigation(browser_id, WindowNavigateMsg::Back));
                    }
                    WindowEvent::KeyEvent(_, Key::Right, _, SUPER) => {
                        compositor.handle_event(WindowEvent::Navigation(browser_id, WindowNavigateMsg::Forward));
                    }
                    WindowEvent::KeyEvent(_, Key::Escape, _, _) => {
                        process::exit(0);
                    }
                    _ => {
                        compositor.handle_event(e);
                    }
                }
            }
            (WindowEvent::Idle, None) => {
                // If the event loop is awaken with no window event, it means
                // servo itself has events.

                // Because of #15934 events come from compositor, not the browser.
                let browser_events = compositor.get_events();
                for e in browser_events {
                    match e {
                        BrowserEvent::CursorChanged(cursor) => {
                            window.set_cursor(cursor);
                        }
                        BrowserEvent::TitleChanged(_browser, title_opt) => {
                            window.set_title(&title_opt.unwrap_or("No Title".to_owned()));
                        }
                        BrowserEvent::LoadStart(_browser) => {
                            window.set_title("Loading");
                        }
                        BrowserEvent::AllowNavigation(_browser, url, chan) => {
                            let follow = url.domain().unwrap() == original_domain;
                            chan.send(follow).unwrap();
                            if !follow {
                                open::that(url.as_str()).ok();
                            }
                        }
                        BrowserEvent::Key(_, Some('r'), _, SUPER) => {
                            // We handled 3 key bindings earlier.
                            // In this case, the Cmd-R combo is handled after the event
                            // has been through the web page.
                            compositor.handle_event(WindowEvent::Reload(browser_id));
                        }
                        _ => {
                            // Some more events
                        }
                    }
                }
                // compositor heartbeat.
                compositor.perform_updates();
            }
            (e, None) => {
                println!("Got unexpected window-less window event: {:?}", e);
            }
        }
    });
}
