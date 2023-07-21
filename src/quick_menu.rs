use std::{
    cmp::max,
    isize,
    sync::{mpsc::Sender, Arc, Mutex},
};

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
    window::ApplicationWindow,
};

pub struct QuickMenu {
    app: app::App,
    quick_menu_window: Window,
    window_list: Flex,
    event_sender: Arc<Mutex<Sender<HarpoonEvent>>>,
    config: QuickMenuConfig,
    state: QuickMenuState,
}

#[derive(Debug, Clone)]
pub enum MoveCursor {
    ToWindow(isize),
    By(isize),
}

pub struct QuickMenuState {
    pub open: bool,
    pub cursor: isize,
    pub windows: Vec<ApplicationWindow>,
    pub active_window: Option<isize>,
}

/// QuickMenuStateUpdate is used to update the state of the quick menu
#[derive(Debug, Clone)]
pub struct QuickMenuStateUpdate<'a> {
    pub windows: Option<&'a Vec<ApplicationWindow>>,
    pub move_cursor: Option<MoveCursor>,
}

impl<'a> QuickMenuStateUpdate<'a> {
    pub fn new() -> Self {
        Self {
            windows: None,
            move_cursor: None,
        }
    }

    /// Update the cursor with the given delta
    ///
    /// If the cursor is out of bounds, it will be clamped to the bounds
    pub fn with_cursor_delta(&'a mut self, cursor_delta: isize) -> &'a Self {
        self.move_cursor = Some(MoveCursor::By(cursor_delta));
        self
    }

    /// Set the windows to the given windows
    pub fn with_windows(&'a mut self, windows: &'a Vec<ApplicationWindow>) -> &'a mut Self {
        self.windows = Some(windows);
        self
    }

    /// Set the active window to the window with the given handle
    ///
    /// If the window with the given handle is not found, the active window will not be changed
    pub fn with_active_window(&'a mut self, active_window: isize) -> &'a mut Self {
        self.move_cursor = Some(MoveCursor::ToWindow(active_window));
        self
    }
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
        let (quick_menu_window, window_list) = QuickMenu::create_window();
        let config = config.into();
        let mut quick_menu = QuickMenu {
            app,
            quick_menu_window,
            window_list,
            state: QuickMenuState {
                open: false,
                cursor: 0,
                windows: vec![],
                active_window: None,
            },
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

    fn create_window() -> (Window, Flex) {
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

        let window_list = Flex::default()
            .with_size(400, 250)
            .with_pos(0, 50)
            .column()
            .with_align(Align::Top | Align::Inside);

        window.add(&banner);
        window.end();

        (window, window_list)
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

            Event::KeyDown => QuickMenu::handle_keydown_event(&event_sender, &actions),
            _ => false,
        });
    }

    /// Hides the quick menu.
    pub fn hide(&mut self) {
        self.quick_menu_window.hide();
        self.state.open = false;
    }

    fn handle_keydown_event(
        event_sender: &Arc<Mutex<Sender<HarpoonEvent>>>,
        actions: &Vec<QuickMenuAction>,
    ) -> bool {
        let event_key = event_key();
        let event_state = event_state();
        let event_text = event_text().to_lowercase();

        let mut handled = false;

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
                    _ = event_sender.send(HarpoonEvent::QuickMenuEvent(key_combination.action));
                }
                Err(err) => {
                    println!("Failed to lock event sender: {}", err);
                }
            }
            handled = true;
        }
        handled
    }

    /// Shows the quick menu.
    ///
    /// Also tries to set the window as the foreground window.
    pub fn show(&mut self) {
        self.render_window_list();

        let window = &mut self.quick_menu_window;
        self.state.open = true;
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
        let should_open = !self.state.open;
        match should_open {
            true => self.show(),
            false => self.hide(),
        }
    }

    pub fn handle_event(&mut self, event: QuickMenuEvent) {
        println!("Handling event in qm: {:?}", event);

        match event {
            QuickMenuEvent::MoveCursorUp => {
                self.update_state(QuickMenuStateUpdate::new().with_cursor_delta(-1));
            }
            QuickMenuEvent::MoveCursorDown => {
                self.update_state(QuickMenuStateUpdate::new().with_cursor_delta(1));
            }
            QuickMenuEvent::Select => {
                let event_sender = match self.event_sender.lock() {
                    Ok(sender) => sender,
                    Err(_) => return,
                };

                match event_sender.send(HarpoonEvent::NavigateToWindowByIndex(
                    self.state.cursor as usize,
                )) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Failed to send event: {}", err);
                    }
                };
            }
            QuickMenuEvent::SwapUp | QuickMenuEvent::SwapDown => {
                let cursor = self.state.cursor as usize;
                let from = cursor;
                let to = match event {
                    QuickMenuEvent::SwapUp => {
                        if from == 0 {
                            return;
                        }
                        cursor - 1
                    }
                    QuickMenuEvent::SwapDown => {
                        if from >= self.state.windows.len() - 1 {
                            return;
                        }
                        cursor + 1
                    }
                    _ => return,
                };

                let event_sender = match self.event_sender.lock() {
                    Ok(sender) => sender,
                    Err(_) => return,
                };

                match event_sender.send(HarpoonEvent::SwapWindows { from, to }) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error sending event: {}", err);
                    }
                }
            }
            QuickMenuEvent::Cut => {
                let cursor = self.state.cursor as usize;
                let event_sender = match self.event_sender.lock() {
                    Ok(sender) => sender,
                    Err(_) => return,
                };

                match event_sender.send(HarpoonEvent::CutWindow(cursor)) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error sending event: {}", err);
                    }
                }
            }
            QuickMenuEvent::PasteUp => {
                let cursor = self.state.cursor as usize;
                let event_sender = match self.event_sender.lock() {
                    Ok(sender) => sender,
                    Err(_) => return,
                };

                match event_sender.send(HarpoonEvent::PasteWindow(cursor)) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error sending event: {}", err);
                    }
                }
            }
            QuickMenuEvent::PasteDown => {
                let cursor = self.state.cursor as usize;
                let event_sender = match self.event_sender.lock() {
                    Ok(sender) => sender,
                    Err(_) => return,
                };

                match event_sender.send(HarpoonEvent::PasteWindow(cursor + 1)) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error sending event: {}", err);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn render_window_list(&mut self) {
        let window_list = &mut self.window_list;

        let windows = &self.state.windows;

        let cursor_pos = self.state.cursor;

        window_list.clear();
        let item_height: i32 = 30;
        let x = window_list.x();
        let y = match windows.len() as i32 * item_height > 200 {
            true => 50 - (max(cursor_pos as i32 - 2, 0) * item_height),
            false => 50,
        };
        let width = window_list.width();
        let height = match windows.len() {
            0 => 50,
            _ => item_height * windows.len() as i32,
        };
        window_list.resize(x, y, width, height);

        for (index, window) in windows.iter().enumerate() {
            let label = format!(
                "{}: {}: \"{}\"",
                index + 1,
                window.process_name,
                window.title,
            );
            let mut item = Frame::default().size_of_parent().with_label(&label);
            item.set_align(Align::Left | Align::Inside);

            item.set_frame(FrameType::FlatBox);

            item.set_color(Color::from_rgb(31, 41, 59));
            item.set_label_color(Color::from_rgb(226, 232, 240));

            if index == cursor_pos as usize {
                item.set_color(Color::from_rgb(51, 56, 85));
                item.set_label_color(Color::from_rgb(248, 250, 252));
            }
            window_list.add(&item);
        }

        if windows.is_empty() {
            let mut item = Frame::default().size_of_parent();
            item.set_frame(FrameType::FlatBox);
            item.set_align(Align::Left | Align::Inside);
            item.set_label("No windows added, press <ctrl> + <alt> + a to add a window");

            item.set_color(Color::from_rgb(31, 41, 59));
            item.set_label_color(Color::from_rgb(226, 232, 240));
            window_list.add(&item);
        }

        self.app.redraw();
    }

    pub fn update_state(&mut self, state: &QuickMenuStateUpdate) {
        let mut updated = false;
        if let Some(windows) = state.windows {
            // make sure we have enough capacity
            let size = windows.len();
            if self.state.windows.capacity() < size {
                self.state.windows.reserve(size - self.state.windows.len());
            }

            // update the windows
            for (index, window) in windows.iter().enumerate() {
                if index < self.state.windows.len() {
                    self.state.windows[index] = window.clone();
                } else {
                    self.state.windows.push(window.clone());
                }
            }

            // remove any extra windows
            if self.state.windows.len() > size {
                self.state.windows.truncate(size);
            }

            updated = true;
        }

        if let Some(ref move_cursor) = state.move_cursor {
            match move_cursor {
                MoveCursor::ToWindow(id) => {
                    if let Some(index) = self.state.windows.iter().position(|w| w.window_id == *id)
                    {
                        self.state.cursor = index as isize;
                    }
                }

                MoveCursor::By(delta) => {
                    let new_cursor = self.state.cursor + delta;
                    let max = self.state.windows.len() as isize - 1;
                    let cursor = match new_cursor {
                        isize::MIN..=0 => 0,
                        i if i <= max => new_cursor,
                        _ => max,
                    };
                    self.state.cursor = cursor;
                }
            }
            updated = true;
        }

        if updated {
            self.notify_updated();
        }
    }

    /// is called when internal state is updated
    fn notify_updated(&mut self) {
        if self.state.open {
            // handle updates that only need to be handled when the menu is open
            self.render_window_list();
        }
        // handle updates that need to be handled regardless of the menu state
    }
}
