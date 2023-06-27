use serde::{Deserialize, Serialize};
use windows::Win32::{
    Foundation::HWND,
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::SetActiveWindow,
        WindowsAndMessaging::{
            BeginDeferWindowPos, BringWindowToTop, DeferWindowPos, EndDeferWindowPos,
            GetForegroundWindow, GetWindowPlacement, GetWindowTextW, GetWindowThreadProcessId,
            SetForegroundWindow, ShowWindow, HWND_TOP, SWP_DRAWFRAME, SWP_SHOWWINDOW, SW_MAXIMIZE,
            SW_NORMAL, WINDOWPLACEMENT,
        },
    },
};

#[derive(Debug, PartialEq, Clone)]
/// This struct represents a window that is running on the system.
pub struct ApplicationWindow {
    pub window_id: HWND,
    pub title: String,
    pub process_path: String,
    pub position: WindowPosition,
    pub state: WindowState,
    pub process_name: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// This struct represents the position of a window on the screen.
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// This enum represents the state of a window.
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
}
/**
   This function will navigate to the window passed in as a parameter.
   It will set the foreground window and set the keyboard focus to the window.

    # Arguments
    * `window: &ApplicationWindow` - A reference to the window to navigate to.
*/
pub fn navigate_to_window(window: &ApplicationWindow) {
    /*
     * we need to attach to the foreground thread to be able to set
     * the foreground window and set keyboard focus to the window
     */
    let foreground_thread_handle = match attach_to_foreground_thread() {
        Ok(handle) => handle,
        Err(e) => {
            println!(
                "Failed to attach to foreground thread while navigating to a window: {}",
                e
            );
            return;
        }
    };

    // move the window to the saved position
    let defer_window_position = match unsafe { BeginDeferWindowPos(1) } {
        Ok(window_pos_defer) => window_pos_defer,
        Err(e) => {
            println!("Failed to begin deferring window position: {}", e);
            return;
        }
    };

    match unsafe {
        DeferWindowPos(
            defer_window_position,
            window.window_id,
            HWND_TOP,
            window.position.x as i32,
            window.position.y as i32,
            window.position.width as i32,
            window.position.height as i32,
            SWP_SHOWWINDOW | SWP_DRAWFRAME,
        )
    } {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to defer window position: {}", e);
            return;
        }
    };

    // apply the window position
    unsafe { EndDeferWindowPos(defer_window_position) };

    // bring the window to the foreground
    unsafe { SetForegroundWindow(window.window_id) };

    // bring the window to the top and make it active
    unsafe { BringWindowToTop(window.window_id) };
    unsafe { SetActiveWindow(window.window_id) };

    // set the window state
    let target_window_state = match window.state {
        WindowState::Normal => SW_NORMAL,
        WindowState::Maximized => SW_MAXIMIZE,
        _ => SW_MAXIMIZE, // if a window is minimized, we want to maximize it
    };

    let mut current_window_state = WINDOWPLACEMENT::default();
    let got_current_window_state =
        unsafe { GetWindowPlacement(window.window_id, &mut current_window_state).as_bool() };

    let should_restore_window_state =
        !got_current_window_state || current_window_state.showCmd != target_window_state;

    // restore the window state if necessary
    if should_restore_window_state {
        unsafe { ShowWindow(window.window_id, target_window_state) };
    }

    // detach from the foreground thread
    detach_from_foreground_thread(foreground_thread_handle);
}

pub fn get_window_title(hwnd: HWND) -> Option<String> {
    let mut title = [0u16; 1024];
    let len = unsafe { GetWindowTextW(hwnd, &mut title) };

    if len == 0 {
        return None;
    }

    let title = String::from_utf16_lossy(&title[..len as usize]);

    Some(title)
}

fn attach_to_foreground_thread() -> Result<u32, &'static str> {
    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.0 == 0 {
            return Err("Failed to get foreground window");
        }

        // get the current foreground thread
        let foreground_thread = GetWindowThreadProcessId(foreground_window, None);

        if foreground_thread == 0 {
            return Err("Failed to get foreground thread");
        }

        // get the current thread
        let current_thread = GetCurrentThreadId();

        // attach the current thread to the foreground thread
        let thread_attached = current_thread == foreground_thread
            || AttachThreadInput(current_thread, foreground_thread, true).as_bool();

        if !thread_attached {
            return Err("Failed to attach thread");
        }

        return Ok(foreground_thread);
    }
}

fn detach_from_foreground_thread(foreground_thread: u32) {
    unsafe {
        let current_thread = GetCurrentThreadId();

        if current_thread != foreground_thread {
            let thread_detached =
                AttachThreadInput(current_thread, foreground_thread, false).as_bool();

            if !thread_detached {
                println!("Failed to detach thread");
            }
        }
    }
}
