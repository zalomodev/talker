use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveAppInfo {
    pub pid: u32,
    pub process_name: String,
    pub window_title: String,
    pub executable_path: String,
}

#[cfg(windows)]
pub fn get_foreground_app() -> Option<ActiveAppInfo> {
    use std::path::Path;
    use windows::Win32::Foundation::{CloseHandle, HWND};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        // Avoid tracking Talker itself
        if pid == std::process::id() {
            return None;
        }

        // Get Window Title
        let text_len = GetWindowTextLengthW(hwnd);
        let mut title = String::new();
        if text_len > 0 {
            let mut title_buf = vec![0u16; (text_len + 1) as usize];
            let copied = GetWindowTextW(hwnd, &mut title_buf);
            if copied > 0 {
                title = String::from_utf16_lossy(&title_buf[..copied as usize]);
            }
        }

        // Get Process Path
        let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) => handle,
            Err(_) => {
                return Some(ActiveAppInfo {
                    pid,
                    process_name: format!("PID:{pid}"),
                    window_title: title,
                    executable_path: String::new(),
                });
            }
        };

        let mut path_buf = vec![0u16; 1024];
        let mut path_len = path_buf.len() as u32;

        let exe_path = if QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(path_buf.as_mut_ptr()),
            &mut path_len,
        )
        .is_ok()
        {
            String::from_utf16_lossy(&path_buf[..path_len as usize])
        } else {
            String::new()
        };

        let _ = CloseHandle(process_handle);

        let process_name = if !exe_path.is_empty() {
            Path::new(&exe_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("PID:{pid}"))
        } else {
            format!("PID:{pid}")
        };

        Some(ActiveAppInfo {
            pid,
            process_name,
            window_title: title,
            executable_path: exe_path,
        })
    }
}

#[cfg(not(windows))]
pub fn get_foreground_app() -> Option<ActiveAppInfo> {
    None
}
