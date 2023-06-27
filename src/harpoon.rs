use std::sync::{
    mpsc::{self, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};

use crate::quick_menu::QuickMenu;
use crate::window::ApplicationWindow;
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HarpoonEvent {
    AddCurrentApplicationWindow,
    ToggleQuickMenu,
    NavigateToNextWindow,
    NavigateToPreviousWindow,
    NavigateToNthWindow(usize),
    ToggleInhibit,
    Quit,
}

impl Harpoon {
    pub fn new() -> Harpoon {
        let quick_menu = QuickMenu::new();
        let (event_sender, event_receiver) = mpsc::channel::<HarpoonEvent>();
        let event_sender = Arc::new(Mutex::new(event_sender));

        let mut harpoon = Harpoon {
            quick_menu,
            event_receiver,
            event_sender,
        };

        harpoon.register_hooks();

        harpoon
    }

    pub fn run(&mut self) {
        loop {
            self.handle_main_events();
            match app::wait_for(0.01) {
                Ok(_) => (),
                Err(err) => println!("Error: {}", err),
            };
            self.handle_window_events();
        }
    }

    fn handle_main_events(&mut self) {
        let msg = self.event_receiver.try_recv();
        match msg {
            Ok(event) => match event {
                HarpoonEvent::ToggleQuickMenu => self.quick_menu.toggle(),
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

    fn handle_window_events(&self) {}

    fn register_hooks(&mut self) {
        self.register_hotkey(
            &[Keyboard::LeftControl, Keyboard::LeftAlt, Keyboard::H],
            HarpoonEvent::ToggleQuickMenu,
            true,
        );
        self.register_hotkey(
            &[Keyboard::LeftControl, Keyboard::LeftAlt, Keyboard::J],
            HarpoonEvent::NavigateToNthWindow(1),
            true,
        );
        self.register_hotkey(
            &[Keyboard::LeftControl, Keyboard::LeftAlt, Keyboard::K],
            HarpoonEvent::NavigateToNthWindow(2),
            true,
        );
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
