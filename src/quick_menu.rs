use fltk::{
    app::{self, event_key, event_state, event_text},
    enums::{Align, Color, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Flex, Group},
    prelude::*,
    window::Window,
};
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

pub struct QuickMenu {
    pub app: Option<app::App>,
    pub quick_menu_window: Option<Window>,
    pub open: bool,
}

impl QuickMenu {
    pub fn new() -> Self {
        let mut qm = QuickMenu {
            app: None,
            quick_menu_window: None,
            open: false,
        };
        qm.create_app();
        qm
    }

    fn create_app(&mut self) {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        app::background(31, 41, 59);
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

        self.quick_menu_window = Some(window);

        self.app = Some(app);
    }

    pub fn hide(&mut self) {
        self.quick_menu_window.as_mut().unwrap().hide();
    }

    pub fn show(&mut self) {
        self.quick_menu_window.as_mut().unwrap().show();

        let window = match self.quick_menu_window.as_mut() {
            Some(window) => window,
            None => return,
        };

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

    pub fn toggle(&mut self) {
        self.open = !self.open;
        match self.open {
            true => self.show(),
            false => self.hide(),
        }
    }
}
