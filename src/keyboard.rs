#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FltkKeyCombination {
    pub keys: fltk::enums::Key,
    pub modifiers: fltk::enums::Shortcut,
    pub text: String,
}

impl FltkKeyCombination {
    /// Create a new FltkKeyCombination from a Vec of mki::Keyboard
    ///
    /// This is necessary to keep the config file format independent from the
    /// fltk crate.
    ///
    /// This way only one way of notating the shortcuts is necessary.
    pub fn from_mki_vec(shortcut: &Vec<mki::Keyboard>) -> Self {
        // prepare... this code is not pretty, fltk has a weird way of handling keydown events
        let mut keys: Vec<fltk::enums::Key> = Vec::with_capacity(shortcut.len());
        let mut modifiers: Vec<fltk::enums::Shortcut> = Vec::with_capacity(shortcut.len());
        let mut text = String::with_capacity(shortcut.len());

        for key in shortcut.iter() {
            match key {
                mki::Keyboard::A => {
                    text.push('a');
                    keys.push(fltk::enums::Key::from_i32(0x61));
                }
                mki::Keyboard::B => {
                    text.push('b');
                    keys.push(fltk::enums::Key::from_i32(0x62));
                }
                mki::Keyboard::C => {
                    text.push('c');
                    keys.push(fltk::enums::Key::from_i32(0x63));
                }
                mki::Keyboard::D => {
                    text.push('d');
                    keys.push(fltk::enums::Key::from_i32(0x64));
                }
                mki::Keyboard::E => {
                    text.push('e');
                    keys.push(fltk::enums::Key::from_i32(0x65));
                }
                mki::Keyboard::F => {
                    text.push('f');
                    keys.push(fltk::enums::Key::from_i32(0x66));
                }
                mki::Keyboard::G => {
                    text.push('g');
                    keys.push(fltk::enums::Key::from_i32(0x67));
                }
                mki::Keyboard::H => {
                    text.push('h');
                    keys.push(fltk::enums::Key::from_i32(0x68));
                }
                mki::Keyboard::I => {
                    text.push('i');
                    keys.push(fltk::enums::Key::from_i32(0x69));
                }
                mki::Keyboard::J => {
                    text.push('j');
                    keys.push(fltk::enums::Key::from_i32(0x6a));
                }
                mki::Keyboard::K => {
                    text.push('k');
                    keys.push(fltk::enums::Key::from_i32(0x6b));
                }
                mki::Keyboard::L => {
                    text.push('l');
                    keys.push(fltk::enums::Key::from_i32(0x6c));
                }
                mki::Keyboard::M => {
                    text.push('m');
                    keys.push(fltk::enums::Key::from_i32(0x6d));
                }
                mki::Keyboard::N => {
                    text.push('n');
                    keys.push(fltk::enums::Key::from_i32(0x6e));
                }
                mki::Keyboard::O => {
                    text.push('o');
                    keys.push(fltk::enums::Key::from_i32(0x6f));
                }
                mki::Keyboard::P => {
                    text.push('p');
                    keys.push(fltk::enums::Key::from_i32(0x70));
                }
                mki::Keyboard::Q => {
                    text.push('q');
                    keys.push(fltk::enums::Key::from_i32(0x71));
                }
                mki::Keyboard::R => {
                    text.push('r');
                    keys.push(fltk::enums::Key::from_i32(0x72));
                }
                mki::Keyboard::S => {
                    text.push('s');
                    keys.push(fltk::enums::Key::from_i32(0x73));
                }
                mki::Keyboard::T => {
                    text.push('t');
                    keys.push(fltk::enums::Key::from_i32(0x74));
                }
                mki::Keyboard::U => {
                    text.push('u');
                    keys.push(fltk::enums::Key::from_i32(0x75));
                }
                mki::Keyboard::V => {
                    text.push('v');
                    keys.push(fltk::enums::Key::from_i32(0x76));
                }
                mki::Keyboard::W => {
                    text.push('w');
                    keys.push(fltk::enums::Key::from_i32(0x77));
                }
                mki::Keyboard::X => {
                    text.push('x');
                    keys.push(fltk::enums::Key::from_i32(0x78));
                }
                mki::Keyboard::Y => {
                    text.push('y');
                    keys.push(fltk::enums::Key::from_i32(0x79));
                }
                mki::Keyboard::Z => {
                    text.push('z');
                    keys.push(fltk::enums::Key::from_i32(0x7a));
                }
                mki::Keyboard::Number0 => {
                    text.push('0');
                    keys.push(fltk::enums::Key::from_i32(0x30));
                }
                mki::Keyboard::Number1 => {
                    text.push('1');
                    keys.push(fltk::enums::Key::from_i32(0x31));
                }
                mki::Keyboard::Number2 => {
                    text.push('2');
                    keys.push(fltk::enums::Key::from_i32(0x32));
                }
                mki::Keyboard::Number3 => {
                    text.push('3');
                    keys.push(fltk::enums::Key::from_i32(0x33));
                }
                mki::Keyboard::Number4 => {
                    text.push('4');
                    keys.push(fltk::enums::Key::from_i32(0x34));
                }
                mki::Keyboard::Number5 => {
                    text.push('5');
                    keys.push(fltk::enums::Key::from_i32(0x35));
                }
                mki::Keyboard::Number6 => {
                    text.push('6');
                    keys.push(fltk::enums::Key::from_i32(0x36));
                }
                mki::Keyboard::Number7 => {
                    text.push('7');
                    keys.push(fltk::enums::Key::from_i32(0x37));
                }
                mki::Keyboard::Number8 => {
                    text.push('8');
                    keys.push(fltk::enums::Key::from_i32(0x38));
                }
                mki::Keyboard::Number9 => {
                    text.push('9');
                    keys.push(fltk::enums::Key::from_i32(0x39));
                }
                mki::Keyboard::LeftAlt | mki::Keyboard::RightAlt => {
                    modifiers.push(fltk::enums::Shortcut::Alt);
                }
                mki::Keyboard::LeftShift | mki::Keyboard::RightShift => {
                    modifiers.push(fltk::enums::Shortcut::Shift);
                }
                mki::Keyboard::LeftControl | mki::Keyboard::RightControl => {
                    modifiers.push(fltk::enums::Shortcut::Ctrl);
                }
                mki::Keyboard::LeftWindows | mki::Keyboard::RightWindows => {
                    modifiers.push(fltk::enums::Shortcut::Meta);
                }
                mki::Keyboard::Space => {
                    text.push(' ');
                }
                mki::Keyboard::BackSpace => {
                    keys.push(fltk::enums::Key::BackSpace);
                }
                mki::Keyboard::Enter => {
                    keys.push(fltk::enums::Key::Enter);
                    text.push('\r');
                }
                mki::Keyboard::Tab => {
                    keys.push(fltk::enums::Key::Tab);
                }
                mki::Keyboard::Escape => {
                    keys.push(fltk::enums::Key::Escape);
                }
                mki::Keyboard::Delete => {
                    keys.push(fltk::enums::Key::Delete);
                }
                mki::Keyboard::Insert => {
                    keys.push(fltk::enums::Key::Insert);
                }
                mki::Keyboard::Home => {
                    keys.push(fltk::enums::Key::Home);
                }
                mki::Keyboard::PageUp => {
                    keys.push(fltk::enums::Key::PageUp);
                }
                mki::Keyboard::PageDown => {
                    keys.push(fltk::enums::Key::PageDown);
                }
                mki::Keyboard::Up => {
                    keys.push(fltk::enums::Key::Up);
                }
                mki::Keyboard::Down => {
                    keys.push(fltk::enums::Key::Down);
                }
                mki::Keyboard::Left => {
                    keys.push(fltk::enums::Key::Left);
                }
                mki::Keyboard::Right => {
                    keys.push(fltk::enums::Key::Right);
                }
                mki::Keyboard::F1 => {
                    keys.push(fltk::enums::Key::F1);
                }
                mki::Keyboard::F2 => {
                    keys.push(fltk::enums::Key::F2);
                }
                mki::Keyboard::F3 => {
                    keys.push(fltk::enums::Key::F3);
                }
                mki::Keyboard::F4 => {
                    keys.push(fltk::enums::Key::F4);
                }
                mki::Keyboard::F5 => {
                    keys.push(fltk::enums::Key::F5);
                }
                mki::Keyboard::F6 => {
                    keys.push(fltk::enums::Key::F6);
                }
                mki::Keyboard::F7 => {
                    keys.push(fltk::enums::Key::F7);
                }
                mki::Keyboard::F8 => {
                    keys.push(fltk::enums::Key::F8);
                }
                mki::Keyboard::F9 => {
                    keys.push(fltk::enums::Key::F9);
                }
                mki::Keyboard::F10 => {
                    keys.push(fltk::enums::Key::F10);
                }
                mki::Keyboard::F11 => {
                    keys.push(fltk::enums::Key::F11);
                }
                mki::Keyboard::F12 => {
                    keys.push(fltk::enums::Key::F12);
                }
                _ => {
                    println!("Unknown key: {:?}", key);
                }
            }
        }

        let mut modifier_enum = fltk::enums::Shortcut::empty();
        for modifier in modifiers {
            modifier_enum.insert(modifier);
        }

        let mut keys_enum = fltk::enums::Key::empty();
        for key in keys {
            keys_enum.insert(key);
        }

        println!("===============================");
        println!("In: {:?}", shortcut);
        println!("************* OUT *************");
        println!("modifiers: {:?}", modifier_enum);
        println!("keys: {:?}", keys_enum);
        println!("text: {:?}", text);

        Self {
            keys: keys_enum,
            modifiers: modifier_enum,
            text,
        }
    }
}
