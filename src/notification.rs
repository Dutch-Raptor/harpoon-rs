use std::{ffi::CString, io::Write};

use anyhow::Result;
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::HWND,
        Graphics::Gdi::LoadBitmapA,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{self, NOTIFYICONDATAA},
            WindowsAndMessaging::{
                LoadImageA, LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
            },
        },
    },
};

use crate::assets::get_app_icon_filepath;

pub fn notify(hwnd: isize, title: &str, content: &str) -> Result<()> {
    let mut icon_path = dbg!(match get_app_icon_filepath() {
        Ok(icon_path) => icon_path,
        Err(_) => panic!("Failed to get icon path"),
    });
    // append null terminator
    icon_path.push('\0');

    let h_instance = match unsafe { GetModuleHandleW(None) } {
        Ok(h_instance) => h_instance,
        Err(_) => panic!("Failed to get module handle"),
    };

    // create a bitmap from the raw bytes of the icon
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

    let mut tray_icon_data = NOTIFYICONDATAA::default();

    tray_icon_data.cbSize = std::mem::size_of::<NOTIFYICONDATAA>() as u32;
    tray_icon_data.hWnd = HWND(hwnd);
    tray_icon_data.uID = 129861;
    tray_icon_data.uFlags = Shell::NIF_INFO | Shell::NIF_ICON;
    tray_icon_data.uCallbackMessage = 0;
    tray_icon_data.hIcon = icon;
    tray_icon_data.szInfoTitle = title.to_utf8_arr64();
    tray_icon_data.szInfo = content.to_utf8_arr256();
    tray_icon_data.dwInfoFlags = Shell::NIIF_NOSOUND;

    unsafe {
        Shell::Shell_NotifyIconA(Shell::NIM_ADD, &mut tray_icon_data);
        Shell::Shell_NotifyIconA(Shell::NIM_DELETE, &mut tray_icon_data);
    }

    Ok(())
}

pub trait ToUtf8 {
    fn to_utf8_arr64(&self) -> [u8; 64];
    fn to_utf8_arr128(&self) -> [u8; 128];
    fn to_utf8_arr256(&self) -> [u8; 256];
}

impl ToUtf8 for String {
    fn to_utf8_arr64(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        let mut buf_ref = buf.as_mut();
        let _ = buf_ref.write(self.as_bytes());
        buf
    }

    fn to_utf8_arr128(&self) -> [u8; 128] {
        let mut buf = [0u8; 128];
        let mut buf_ref = buf.as_mut();
        let _ = buf_ref.write(self.as_bytes());
        buf
    }

    fn to_utf8_arr256(&self) -> [u8; 256] {
        let mut buf = [0u8; 256];
        let mut buf_ref = buf.as_mut();
        let _ = buf_ref.write(self.as_bytes());
        buf
    }
}

impl ToUtf8 for &str {
    fn to_utf8_arr64(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        let mut buf_ref = buf.as_mut();
        let _ = buf_ref.write(self.as_bytes());
        buf
    }

    fn to_utf8_arr128(&self) -> [u8; 128] {
        let mut buf = [0u8; 128];
        let mut buf_ref = buf.as_mut();
        let _ = buf_ref.write(self.as_bytes());
        buf
    }

    fn to_utf8_arr256(&self) -> [u8; 256] {
        let mut buf = [0u8; 256];
        let mut buf_ref = buf.as_mut();
        let _ = buf_ref.write(self.as_bytes());
        buf
    }
}
