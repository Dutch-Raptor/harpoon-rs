use std::{path::Path, sync::mpsc::channel, thread};

use active_win_pos_rs::get_active_window;
use serde::{Deserialize, Serialize};
use windows::{
    core::{PCSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, BOOL, HWND, LPARAM, LRESULT, MAX_PATH, WPARAM},
        Graphics::Gdi::HBRUSH,
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{
                AttachThreadInput, GetCurrentThreadId, OpenProcess, QueryFullProcessImageNameW,
                PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            Input::KeyboardAndMouse::SetActiveWindow,
            WindowsAndMessaging::{
                BeginDeferWindowPos, BringWindowToTop, CreateWindowExA, DefWindowProcA,
                DeferWindowPos, DispatchMessageA, EndDeferWindowPos, GetForegroundWindow,
                GetMessageA, GetWindowPlacement, GetWindowTextW, GetWindowThreadProcessId,
                LoadCursorW, LoadImageA, PostQuitMessage, RegisterClassA,
                SetForegroundWindow, ShowWindow, TranslateMessage, HICON, HWND_TOP, IDC_ARROW,
                IMAGE_ICON, LR_LOADFROMFILE, MSG, SWP_DRAWFRAME,
                SWP_SHOWWINDOW, SW_HIDE, SW_MAXIMIZE, SW_NORMAL, SW_SHOWMAXIMIZED,
                SW_SHOWMINIMIZED, WINDOWPLACEMENT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
                WM_NULL, WNDCLASSA, WNDCLASS_STYLES,
            },
        },
    },
};

use crate::assets::get_app_icon_filepath;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// This struct represents a window that is running on the system.
pub struct ApplicationWindow {
    pub window_id: isize,
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
    // Convert the isize window_id to a HWND
    let window_handle = HWND(window.window_id);

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
            window_handle,
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
    unsafe { SetForegroundWindow(window_handle) };

    // bring the window to the top and make it active
    unsafe { BringWindowToTop(window_handle) };
    unsafe { SetActiveWindow(window_handle) };

    // set the window state
    let target_window_state = match window.state {
        WindowState::Normal => SW_NORMAL,
        WindowState::Maximized => SW_MAXIMIZE,
        _ => SW_MAXIMIZE, // if a window is minimized, we want to maximize it
    };

    let mut current_window_state = WINDOWPLACEMENT::default();
    let got_current_window_state =
        unsafe { GetWindowPlacement(window_handle, &mut current_window_state).as_bool() };

    let should_restore_window_state =
        !got_current_window_state || current_window_state.showCmd != target_window_state;

    // restore the window state if necessary
    if should_restore_window_state {
        unsafe { ShowWindow(window_handle, target_window_state) };
    }

    // detach from the foreground thread
    detach_from_foreground_thread(foreground_thread_handle);
}

/// Returns an ApplicationWindow for the currently active window.
pub fn get_current_window() -> Option<ApplicationWindow> {
    let window = match get_active_window() {
        Ok(window) => window,
        Err(_) => {
            return None;
        }
    };

    // get the number from the hwnd(<number>) string
    let hwnd = window.window_id[5..window.window_id.len() - 1]
        .parse::<isize>()
        .unwrap();

    let mut window_placement = WINDOWPLACEMENT::default();
    unsafe {
        if !GetWindowPlacement(HWND(hwnd), &mut window_placement).as_bool() {
            return None;
        }
    }

    let window_state = match window_placement.showCmd {
        SW_SHOWMAXIMIZED => WindowState::Maximized,
        SW_SHOWMINIMIZED => WindowState::Minimized,
        _ => WindowState::Normal,
    };

    let process_path = match get_window_path_name(window.process_id as u32) {
        Ok(path) => path,
        Err(_) => {
            return None;
        }
    };

    let application_window = ApplicationWindow {
        window_id: hwnd,
        title: window.title,
        process_path,
        process_name: window.process_name,

        position: WindowPosition {
            x: window.position.x,
            y: window.position.y,
            width: window.position.width,
            height: window.position.height,
        },
        state: window_state,
    };

    Some(application_window)
}

pub fn get_window_title(hwnd: isize) -> Option<String> {
    let mut title = [0u16; 1024];
    let len = unsafe { GetWindowTextW(HWND(hwnd), &mut title) };

    if len == 0 {
        return None;
    }

    let title = String::from_utf16_lossy(&title[..len as usize]);

    Some(title)
}

/// Returns the path of the executable of the process with the given process id.
pub fn get_window_path_name(process_id: u32) -> Result<String, ()> {
    let process_handle = match unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            BOOL::from(false),
            process_id,
        )
    } {
        Ok(handle) => handle,
        Err(_) => return Err(()),
    };

    let mut buffer_size = MAX_PATH as u32;
    let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let process_path_pwstr = PWSTR::from_raw(buffer.as_mut_ptr());

    let process_path = unsafe {
        let success = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            process_path_pwstr,
            &mut buffer_size,
        )
        .as_bool();
        CloseHandle(process_handle);
        if !success {
            return Err(());
        }

        process_path_pwstr.to_string().map_err(|_| ())?
    };

    let path = Path::new(&process_path)
        .to_path_buf()
        .to_str()
        .ok_or(())
        .map(|s| s.to_string())?;

    Ok(path)
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

pub fn create_window() -> isize {
    let (sender, receiver) = channel::<isize>();
    thread::spawn(move || {
        let h_instance = match unsafe { GetModuleHandleW(None) } {
            Ok(h_instance) => h_instance,
            Err(_) => panic!("Failed to get module handle"),
        };

        let mut icon_path = dbg!(match get_app_icon_filepath() {
            Ok(icon_path) => icon_path,
            Err(_) => panic!("Failed to get icon path"),
        });
        // append null terminator
        icon_path.push('\0');

        let icon = match unsafe {
            LoadImageA(
                h_instance,
                PCSTR(icon_path.as_ptr()),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE,
            )
        } {
            Ok(icon) => HICON(icon.0),
            Err(e) => panic!("Failed to load icon: {}", e),
        };

        let class_name = PCSTR(b"HarpoonClass\0".as_ptr() as *const u8);

        let cursor = match unsafe { LoadCursorW(None, IDC_ARROW) } {
            Ok(cursor) => cursor,
            Err(_) => panic!("Failed to load cursor"),
        };

        let window_class = WNDCLASSA {
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: icon,
            hCursor: cursor,
            hbrBackground: HBRUSH(0),
            lpszClassName: class_name,
            lpszMenuName: PCSTR(std::ptr::null()),
        };

        unsafe {
            dbg!(RegisterClassA(&window_class));
        }

        let hwnd = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE(0),
                class_name,
                PCSTR("Harpoon\0".as_ptr() as *const u8),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                None,
                None,
                h_instance,
                None,
            )
        };

        sender.send(hwnd.0).unwrap();

        unsafe { ShowWindow(hwnd, SW_HIDE) };

        let mut msg = MSG::default();
        unsafe {
            loop {
                GetMessageA(&mut msg, None, 0, 0).as_bool();
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
                if msg.message == WM_NULL {
                    break;
                }
            }
        }
    });

    let hwnd = receiver.recv().unwrap();
    hwnd
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }

        _ => unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) },
    }
}
