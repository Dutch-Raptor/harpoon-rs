use fltk::enums::{Key, Shortcut};
use mki::Keyboard;
use serde::{Deserialize, Serialize};

use crate::{harpoon::HarpoonEvent, keyboard::FltkKeyCombination, quick_menu::QuickMenuEvent};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub leader: Vec<Keyboard>,
    pub actions: Vec<Action<HarpoonEvent>>,
    pub quick_menu_config: StoredQuickMenuConfig,
}

#[derive(Debug, Clone)]
pub struct QuickMenuConfig {
    pub actions: Vec<QuickMenuAction>,
}

#[derive(Debug, Clone)]
pub struct QuickMenuAction {
    pub trigger: FltkKeyCombination,
    pub action: QuickMenuEvent,
}

impl QuickMenuAction {
    pub fn is_triggered(&self, event_key: Key, event_state: Shortcut, event_text: &str) -> bool {
        // Remove a random bit from the event_state, this is a hack to make sure that the event_state is the same as the one that is stored in the config
        let mut event_state = event_state;
        event_state.remove(Shortcut::from_i32(0x100000));

        self.trigger
            .is_triggered(event_key, event_state, event_text)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StoredQuickMenuConfig {
    pub actions: Vec<Action<QuickMenuEvent>>,
}

impl<T> Action<T> {
    pub fn to_fltk_shortcut(&self) -> FltkKeyCombination {
        FltkKeyCombination::from_mki_vec(&self.keys)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Action<T> {
    pub keys: Vec<Keyboard>,
    pub action: T,
}

pub fn load_config_from_disk() -> Result<Config, Box<dyn std::error::Error>> {
    if !std::path::Path::new("config.json").exists() {
        let config = Config::default();
        save_config_to_disk(&config);
        return Ok(config);
    }
    let config = match std::fs::read_to_string("config.json") {
        Ok(config) => config,
        Err(_) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to read config.json, does the file exist?",
        )))?,
    };
    match serde_json::from_str(&config) {
        Ok(config) => Ok(config),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn save_config_to_disk(config: &Config) {
    let config = serde_json::to_string_pretty(config).unwrap();
    std::fs::write("config.json", config).unwrap();
}

impl Config {
    pub fn default() -> Config {
        Config {
            leader: vec![Keyboard::LeftControl, Keyboard::LeftAlt],
            actions: vec![
                Action {
                    keys: vec![Keyboard::H],
                    action: HarpoonEvent::ToggleQuickMenu,
                },
                Action {
                    keys: vec![Keyboard::J],
                    action: HarpoonEvent::NavigateToWindowByIndex(0),
                },
                Action {
                    keys: vec![Keyboard::K],
                    action: HarpoonEvent::NavigateToWindowByIndex(1),
                },
                Action {
                    keys: vec![Keyboard::L],
                    action: HarpoonEvent::NavigateToWindowByIndex(2),
                },
                Action {
                    keys: vec![Keyboard::SemiColon],
                    action: HarpoonEvent::NavigateToWindowByIndex(3),
                },
                Action {
                    keys: vec![Keyboard::U],
                    action: HarpoonEvent::NavigateToWindowByIndex(4),
                },
                Action {
                    keys: vec![Keyboard::I],
                    action: HarpoonEvent::NavigateToWindowByIndex(5),
                },
                Action {
                    keys: vec![Keyboard::O],
                    action: HarpoonEvent::NavigateToWindowByIndex(6),
                },
                Action {
                    keys: vec![Keyboard::P],
                    action: HarpoonEvent::NavigateToWindowByIndex(7),
                },
                Action {
                    keys: vec![Keyboard::M],
                    action: HarpoonEvent::NavigateToNextWindow,
                },
                Action {
                    keys: vec![Keyboard::N],
                    action: HarpoonEvent::NavigateToPreviousWindow,
                },
                Action {
                    keys: vec![Keyboard::A],
                    action: HarpoonEvent::AddCurrentApplicationWindow,
                },
                Action {
                    keys: vec![Keyboard::S],
                    action: HarpoonEvent::ToggleInhibit,
                },
            ],
            quick_menu_config: StoredQuickMenuConfig {
                actions: vec![
                    Action {
                        keys: vec![Keyboard::Q],
                        action: QuickMenuEvent::Quit,
                    },
                    Action {
                        keys: vec![Keyboard::Escape],
                        action: QuickMenuEvent::Quit,
                    },
                    Action {
                        keys: vec![Keyboard::J],
                        action: QuickMenuEvent::MoveCursorDown,
                    },
                    Action {
                        keys: vec![Keyboard::K],
                        action: QuickMenuEvent::MoveCursorUp,
                    },
                    Action {
                        keys: vec![Keyboard::Down],
                        action: QuickMenuEvent::MoveCursorDown,
                    },
                    Action {
                        keys: vec![Keyboard::Up],
                        action: QuickMenuEvent::MoveCursorUp,
                    },
                    Action {
                        keys: vec![Keyboard::LeftAlt, Keyboard::J],
                        action: QuickMenuEvent::SwapDown,
                    },
                    Action {
                        keys: vec![Keyboard::LeftAlt, Keyboard::K],
                        action: QuickMenuEvent::SwapUp,
                    },
                    Action {
                        keys: vec![Keyboard::LeftAlt, Keyboard::Down],
                        action: QuickMenuEvent::SwapDown,
                    },
                    Action {
                        keys: vec![Keyboard::LeftAlt, Keyboard::Up],
                        action: QuickMenuEvent::SwapUp,
                    },
                    Action {
                        keys: vec![Keyboard::Enter],
                        action: QuickMenuEvent::Select,
                    },
                    Action {
                        keys: vec![Keyboard::Space],
                        action: QuickMenuEvent::Select,
                    },
                    Action {
                        keys: vec![Keyboard::BackSpace],
                        action: QuickMenuEvent::Cut,
                    },
                    Action {
                        keys: vec![Keyboard::LeftShift, Keyboard::D],
                        action: QuickMenuEvent::Cut,
                    },
                    Action {
                        keys: vec![Keyboard::P],
                        action: QuickMenuEvent::PasteDown,
                    },
                    Action {
                        keys: vec![Keyboard::LeftShift, Keyboard::P],
                        action: QuickMenuEvent::PasteUp,
                    },
                ],
            },
        }
    }

    pub fn get_action_shortcut_string(&self, event: &HarpoonEvent) -> Option<String> {
        let mut shortcut_string = String::new();
        match event {
            HarpoonEvent::QuickMenuEvent(event) => {
                for action in self.quick_menu_config.actions.iter() {
                    if action.action == *event {
                        let mut keys = Vec::new();
                        for key in action.keys.iter() {
                            keys.push(*key);
                        }

                        if shortcut_string.len() > 0 {
                            shortcut_string.push_str(" or ");
                        }
                        shortcut_string.push_str(
                            &keys
                                .iter()
                                .map(|key| format!("{:?}", key))
                                .collect::<Vec<String>>()
                                .join(" + "),
                        );
                    }
                }
            }
            _ => {
                for action in self.actions.iter() {
                    if action.action == *event {
                        let mut keys = self.leader.clone();
                        for key in action.keys.iter() {
                            keys.push(*key);
                        }

                        if shortcut_string.len() > 0 {
                            shortcut_string.push_str(" or ");
                        }
                        shortcut_string.push_str(
                            &keys
                                .iter()
                                .map(|key| format!("{:?}", key))
                                .collect::<Vec<String>>()
                                .join(" + "),
                        );
                    }
                }
            }
        }

        if shortcut_string.is_empty() {
            None
        } else {
            Some(shortcut_string)
        }
    }
}
