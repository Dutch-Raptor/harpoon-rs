use std::sync::{
    mpsc::{self, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};

use crate::{
    assets::get_app_icon_filepath,
    config,
    notification::notify,
    quick_menu::{QuickMenu, QuickMenuStateUpdate},
    window::{self, create_window, get_current_window, get_window_title, navigate_to_window},
};
use crate::{quick_menu::QuickMenuEvent, window::ApplicationWindow};
use active_win_pos_rs::get_active_window;
use anyhow::Result;
use fltk::{
    app::{self, event_key, event_state, event_text},
    enums::{Align, Color, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Flex, Group},
    prelude::*,
    window::Window,
};
use mki::Keyboard;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use windows::{
    core::{Error, HSTRING},
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{GetForegroundWindow, IsWindow},
    },
};

pub struct Harpoon {
    quick_menu: QuickMenu,
    pub event_receiver: Receiver<HarpoonEvent>,
    pub event_sender: Arc<Mutex<Sender<HarpoonEvent>>>,
    config: config::Config,
    /// whether or not to disable keyboard events from being inhibited to other applications
    disable_inhibit: bool,
    windows: Vec<ApplicationWindow>,
    /// the last window id that was focused
    last_window_id: Option<isize>,
    clipboard: Option<ApplicationWindow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HarpoonEvent {
    AddCurrentApplicationWindow,
    ToggleQuickMenu,
    CloseQuickMenu,
    NavigateToNextWindow,
    NavigateToPreviousWindow,
    NavigateToWindowByIndex(usize),
    ToggleInhibit,
    Quit,
    SwapWindows { from: usize, to: usize },
    CutWindow(usize),
    PasteWindow(usize),
    QuickMenuEvent(QuickMenuEvent),
}

impl Harpoon {
    pub fn new() -> Harpoon {
        let (event_sender, event_receiver) = mpsc::channel::<HarpoonEvent>();
        let event_sender = Arc::new(Mutex::new(event_sender));

        let config = match config::load_config_from_disk() {
            Ok(config) => config,
            Err(e) => {
                println!("Error loading config: {}", e);
                config::Config::default()
            }
        };
        let quick_menu =
            QuickMenu::new(Arc::clone(&event_sender), config.quick_menu_config.clone());

        let mut harpoon = Harpoon {
            quick_menu,
            event_receiver,
            event_sender,
            config,
            disable_inhibit: false,
            windows: vec![],
            last_window_id: None,
            clipboard: None,
        };

        let app_hwnd = create_window();

        _ = dbg!(notify(
            app_hwnd,
            "This is a test",
            "TEST TEST TEST TEST TEST"
        ));

        harpoon.register_hooks();

        harpoon
    }

    pub fn run(&mut self) {
        loop {
            self.handle_main_events();
            // Somehow waiting for events also handles them in fltk-rs (??) so we don't need to
            // explicitly handle them here.
            match app::wait_for(1.0 / 120.0) {
                Ok(_) => (),
                Err(err) => println!("Error waiting for fltk events: {}", err),
            };
        }
    }

    fn handle_main_events(&mut self) {
        let msg = self.event_receiver.try_recv();
        match msg {
            Ok(event) => match event {
                HarpoonEvent::ToggleQuickMenu => self.quick_menu.toggle(),

                HarpoonEvent::CloseQuickMenu
                | HarpoonEvent::QuickMenuEvent(QuickMenuEvent::Quit) => self.quick_menu.hide(),

                HarpoonEvent::QuickMenuEvent(event) => {
                    self.quick_menu.handle_event(event);
                }

                HarpoonEvent::AddCurrentApplicationWindow => {
                    self.add_current_application_window().unwrap_or_else(|err| {
                        println!("Error adding current application window: {}", err)
                    });
                    self.quick_menu
                        .update_state(QuickMenuStateUpdate::new().with_windows(&self.windows));
                }

                HarpoonEvent::NavigateToNextWindow => self.navigate_relative(1),
                HarpoonEvent::NavigateToPreviousWindow => self.navigate_relative(-1),
                HarpoonEvent::NavigateToWindowByIndex(i) => {
                    self.navigate_to_window_by_index(i);
                }

                HarpoonEvent::SwapWindows { from, to } => self.swap_windows(from, to),

                HarpoonEvent::CutWindow(i) => self.cut_window(i),
                HarpoonEvent::PasteWindow(i) => self.paste_window(i),

                _ => {
                    println!("Handling event {:?}", event);
                }
            },
            Err(TryRecvError::Empty) => {
                // No events to handle
            }
            Err(TryRecvError::Disconnected) => {
                println!("Event channel disconnected");
                app::quit();
            }
        }
    }

    fn register_hooks(&mut self) {
        let config = &self.config;
        let disable_inhibit = self.disable_inhibit;

        for action in config.actions.iter() {
            let mut hotkey = config.leader.clone();
            hotkey.extend(action.keys.clone());

            let event = action.action.clone();

            self.register_hotkey(&hotkey, event, !disable_inhibit);
        }
    }

    fn register_hotkey(&self, hotkey: &[Keyboard], event: HarpoonEvent, inhibit: bool) {
        let sender_clone = Arc::clone(&self.event_sender);
        mki::register_hotkey(
            hotkey,
            move || {
                let sender = sender_clone.lock().unwrap();
                sender.send(event.clone()).unwrap();
            },
            inhibit,
        );
    }

    fn add_current_application_window(&mut self) -> Result<()> {
        let windows = &mut self.windows;
        let application_window = match get_current_window() {
            Some(window) => window,
            None => return Err(anyhow!("No window found")),
        };

        if windows.contains(&application_window) {
            return Ok(());
        }

        if windows.len() == 0 {
            windows.push(application_window);
            return Ok(());
        }

        // if the window does aleady exist, update it
        if let Some(index) = windows
            .iter()
            .position(|w| w.window_id == application_window.window_id)
        {
            windows[index] = application_window;
            return Ok(());
        }

        windows.push(application_window);

        Ok(())
    }

    fn navigate_to_window_by_index(&mut self, index: usize) {
        let window = match self.windows.get(index) {
            Some(window) => window,
            None => return,
        };
        self.navigate_to_window(window.clone());
    }

    /// get the current active window
    ///
    /// If the current window is in the list of windows, then we can
    /// navigate relative to it.
    /// Otherwise, navigate relative to the window last navigated to.
    /// If all else fails, navigate to the first window in the list.
    fn navigate_relative(&mut self, delta: isize) {
        let hwnd = unsafe { GetForegroundWindow() }.0;

        let windows = &self.windows;
        let current_window_index = windows.iter().position(|w| w.window_id == hwnd);

        if current_window_index.is_none() {
            // navigate to the window to which the user navigated most recently
            let hwnd = self.last_window_id;
            if let Some(hwnd) = hwnd {
                // find the window in the list of windows
                match windows.iter().find(|w| w.window_id == hwnd) {
                    Some(window) => {
                        let window_clone = window.clone();
                        self.navigate_to_window(window_clone);
                        return;
                    }
                    None => {
                        // if we can't find the last window, navigate to the first window
                        self.navigate_to_window_by_index(1);
                        return;
                    }
                };
            }
            self.navigate_to_window_by_index(1); // if there is no last window, navigate to the first window;
            return;
        }

        let current_window_index = current_window_index.unwrap();

        let windows_len = windows.len();

        let next_window_index =
            (current_window_index as isize + windows_len as isize + delta) as usize % windows_len;

        match windows.get(next_window_index) {
            Some(window) => {
                let window = window.clone();
                self.navigate_to_window(window);
            }
            None => {
                // if we can't find the last window, navigate to the first window
                self.navigate_to_window_by_index(1);
            }
        }
    }

    fn navigate_to_window(&mut self, window: ApplicationWindow) {
        let exists = unsafe { IsWindow(HWND(window.window_id)).as_bool() };

        let closed_prefix = "[CLOSED] ";

        if !exists {
            if window.process_name.starts_with(closed_prefix) {
                return;
            }
            let windows = &mut self.windows;

            if let Some(index) = windows.iter().position(|w| w.window_id == window.window_id) {
                windows[index].process_name = format!("{}{}", closed_prefix, window.process_name);
            }
            return;
        }

        navigate_to_window(&window);
        self.last_window_id = Some(window.window_id);

        let _ = self.update_window_title(window.window_id);
        self.quick_menu.update_state(
            QuickMenuStateUpdate::new()
                .with_windows(&self.windows)
                .with_active_window(window.window_id),
        );
    }

    fn update_window_title(&mut self, window_id: isize) -> Result<()> {
        let title = match get_window_title(window_id) {
            Some(title) => title,
            None => {
                return Err(anyhow!(
                    "Failed to get window title for window id: {:?}",
                    window_id
                ));
            }
        };

        let windows = &mut self.windows;

        if let Some(index) = windows.iter().position(|w| w.window_id == window_id) {
            windows[index].title = title;
        }

        Ok(())
    }

    fn swap_windows(&mut self, from_index: usize, to_index: usize) {
        self.windows.swap(from_index, to_index);
        let cursor_delta = to_index as isize - from_index as isize;
        self.quick_menu.update_state(
            QuickMenuStateUpdate::new()
                .with_windows(&self.windows)
                .with_cursor_delta(cursor_delta),
        );
    }

    fn cut_window(&mut self, index: usize) {
        if self.windows.get(index).is_none() {
            return;
        }
        let window = self.windows.remove(index);
        self.quick_menu.update_state(
            QuickMenuStateUpdate::new()
                .with_windows(&self.windows)
                .with_cursor_delta(-1),
        );
        self.clipboard = Some(window);
    }

    fn paste_window(&mut self, index: usize) {
        if let Some(window) = self.clipboard.take() {
            let mut index = index;
            if index > self.windows.len() {
                index = self.windows.len();
            }
            self.windows.insert(index, window);
            self.quick_menu.update_state(
                QuickMenuStateUpdate::new()
                    .with_windows(&self.windows)
                    .with_cursor_delta(1),
            );
        }
    }
}
