use std::sync::{
    mpsc::{self, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};

use crate::{config, quick_menu::QuickMenu};
use crate::{quick_menu::QuickMenuEvent, window::ApplicationWindow};
use fltk::{
    app::{self, event_key, event_state, event_text},
    enums::{Align, Color, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Flex, Group},
    prelude::*,
    window::Window,
};
use mki::Keyboard;
use serde::{Deserialize, Serialize};
use windows::core::{Error, Result, HSTRING};

pub struct Harpoon {
    quick_menu: QuickMenu,
    pub event_receiver: Receiver<HarpoonEvent>,
    pub event_sender: Arc<Mutex<Sender<HarpoonEvent>>>,
    config: config::Config,
    /// whether or not to disable keyboard events from being inhibited to other applications
    disable_inhibit: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HarpoonEvent {
    AddCurrentApplicationWindow,
    ToggleQuickMenu,
    CloseQuickMenu,
    NavigateToNextWindow,
    NavigateToPreviousWindow,
    NavigateToNthWindow(usize),
    ToggleInhibit,
    Quit,
    SetWindows(Vec<ApplicationWindow>),
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
        };

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
}
