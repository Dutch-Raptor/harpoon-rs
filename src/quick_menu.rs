use std::sync::{mpsc::Sender, Arc, Mutex};

use fltk::{
    app::{self, event_key, event_state, event_text},
    enums::{Align, Color, Event, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Flex, Group},
    prelude::*,
    window::Window,
};
use serde::{Deserialize, Serialize};
use windows::Win32::{
    Foundation::HWND,
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::SetActiveWindow,
        WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
        },
    },
};

use crate::{
    config::{QuickMenuAction, QuickMenuConfig, StoredQuickMenuConfig},
    harpoon::HarpoonEvent,
};

pub struct QuickMenu {
    pub app: app::App,
    pub quick_menu_window: Window,
    pub open: bool,
    event_sender: Arc<Mutex<Sender<HarpoonEvent>>>,
    config: QuickMenuConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QuickMenuEvent {
    /// Move the cursor down
    MoveCursorDown,
    /// Move the cursor up
    MoveCursorUp,
    /// Navigate to the selected window and close the quick menu
    Select,
    /// Close the quick menu
    Quit,
    /// Cut the selected window and put it in the clipboard
    Cut,
    /// Paste the selected window from the clipboard after the selected window
    PasteDown,
    /// Paste the selected window from the clipboard before the selected window
    PasteUp,
    /// Swap the selected window with the window above it
    SwapUp,
    /// Swap the selected window with the window below it
    SwapDown,
}

impl Into<QuickMenuConfig> for StoredQuickMenuConfig {
    fn into(self) -> QuickMenuConfig {
        QuickMenuConfig {
            actions: self
                .actions
                .into_iter()
                .map(|action| QuickMenuAction {
                    trigger: action.to_fltk_shortcut(),
                    action: action.action,
                })
                .collect(),
        }
    }
}

impl QuickMenu {
    pub fn new(
        event_sender: Arc<Mutex<Sender<HarpoonEvent>>>,
        config: StoredQuickMenuConfig,
    ) -> Self {
        let app = QuickMenu::create_app();
        let quick_menu_window = QuickMenu::create_window();
        let config = config.into();
        let mut quick_menu = QuickMenu {
            app,
            quick_menu_window,
            open: false,
            event_sender,
            config,
        };

        quick_menu.register_window_event_handlers();

        quick_menu
    }

    fn create_app() -> app::App {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        app::background(31, 41, 59);
        app
    }

    fn create_window() -> Window {
        let (screen_w, screen_h) = app::screen_size();
        let mut window = Window::default()
            .with_size(400, 300)
            .with_pos(screen_w as i32 / 2 - 200, screen_h as i32 / 2 - 150)
            .with_label("Quick Menu");
        window.set_border(false);
        window.set_color(Color::from_rgb(31, 41, 59));

        let mut banner = Frame::default()
            .with_label("Harpoon")
            .with_size(400, 50)
            .with_pos(0, 0);

        banner.set_frame(FrameType::FlatBox);
        banner.set_color(Color::from_rgb(51, 65, 85));
        banner.set_label_size(20);
        banner.set_label_color(Color::from_rgb(248, 250, 252));
        banner.set_align(Align::Center | Align::Inside);

        window.add(&banner);
        window.end();

        window
    }

    fn register_window_event_handlers(&mut self) {
        let event_sender = Arc::clone(&self.event_sender);
        let actions = self.config.actions.clone();

        self.quick_menu_window.handle(move |_, ev| match ev {
            Event::Unfocus => {
                match event_sender.lock() {
                    Ok(sender) => {
                        _ = sender.send(HarpoonEvent::CloseQuickMenu);
                    }
                    Err(_) => {}
                }

                true
            }

            Event::KeyDown => {
                let event_key = event_key();
                let event_state = event_state();
                let event_text = event_text().to_lowercase();

                println!(
                    "Key down event: {:?} {:?} {:?}",
                    event_key, event_state, event_text
                );

                // Loop through all actions and check if any of them match the key combination
                for key_combination in actions.iter() {
                    if key_combination.trigger.keys != event_key
                        || key_combination.trigger.modifiers != event_state
                    {
                        continue;
                    }

                    let text = &key_combination.trigger.text;

                    // check if the text contains exactly the same characters
                    // as the event text
                    let text_matches = text.is_empty()
                        || (text.len() == event_text.len()
                            && text.chars().all(|char| event_text.contains(char))
                            && event_text.chars().all(|char| text.contains(char)));

                    if !text_matches {
                        continue;
                    }

                    match event_sender.lock() {
                        Ok(event_sender) => {
                            _ = event_sender
                                .send(HarpoonEvent::QuickMenuEvent(key_combination.action));
                        }
                        Err(err) => {
                            println!("Failed to lock event sender: {}", err);
                        }
                    }
                }

                true
            }
            _ => false,
        });
    }

    /// Hides the quick menu.
    pub fn hide(&mut self) {
        self.quick_menu_window.hide();
        self.open = false;
    }

    /// Shows the quick menu.
    ///
    /// Also tries to set the window as the foreground window.
    pub fn show(&mut self) {
        let window = &mut self.quick_menu_window;
        self.open = true;
        window.show();

        let hwnd = HWND(window.raw_handle() as isize);

        unsafe {
            let foreground_window = GetForegroundWindow();
            if foreground_window.0 == 0 {
                println!("Failed to get foreground window");
                return;
            }

            // get the current foreground thread
            let foreground_thread = GetWindowThreadProcessId(foreground_window, None);

            if foreground_thread == 0 {
                println!("Failed to get foreground thread");
                return;
            }

            // get the current thread
            let current_thread = GetCurrentThreadId();

            // attach the current thread to the foreground thread
            let thread_attached = current_thread == foreground_thread
                || AttachThreadInput(current_thread, foreground_thread, true).as_bool();

            if !thread_attached {
                println!("Failed to attach thread");
                return;
            }

            SetForegroundWindow(hwnd);
            BringWindowToTop(hwnd);
            SetActiveWindow(hwnd);

            if thread_attached {
                AttachThreadInput(current_thread, foreground_thread, false);
            }
        }
    }

    /// Toggle the visibility of the quick menu
    pub fn toggle(&mut self) {
        let should_open = !self.open;
        match should_open {
            true => self.show(),
            false => self.hide(),
        }
    }

    pub fn handle_event(&mut self, event: QuickMenuEvent) {
        println!("Handling event in qm: {:?}", event);
    }
}
