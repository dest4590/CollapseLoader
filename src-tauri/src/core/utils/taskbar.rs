use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::LazyLock;

static LAST_PROGRESS: LazyLock<AtomicU8> = LazyLock::new(|| AtomicU8::new(255));

pub fn set_progress(percentage: u8) {
    let last = LAST_PROGRESS.load(Ordering::Relaxed);
    
    if last == percentage {
        return;
    }
    
    if percentage > 0 && percentage < 100 && last != 255 {
        let diff = if percentage > last {
            percentage - last
        } else {
            last - percentage
        };
        if diff < 3 {
            return;
        }
    }
    
    LAST_PROGRESS.store(percentage, Ordering::Relaxed);

    #[cfg(target_os = "windows")]
    set_progress_windows(percentage);

    #[cfg(target_os = "macos")]
    set_progress_macos(percentage);

    #[cfg(target_os = "linux")]
    set_progress_linux(percentage);
}

pub fn clear_progress() {
    LAST_PROGRESS.store(255, Ordering::Relaxed);

    #[cfg(target_os = "windows")]
    clear_progress_windows();

    #[cfg(target_os = "macos")]
    clear_progress_macos();

    #[cfg(target_os = "linux")]
    clear_progress_linux();
}

#[cfg(target_os = "windows")]
fn find_main_hwnd() -> Option<windows::Win32::Foundation::HWND> {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
    };

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut (u32, Option<HWND>));
        let (target_pid, ref mut found) = *data;

        let mut window_pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

        if window_pid == target_pid && IsWindowVisible(hwnd).as_bool() {
            *found = Some(hwnd);
            return BOOL(0);
        }
        BOOL(1)
    }

    unsafe {
        let current_pid = GetCurrentProcessId();
        let mut data: (u32, Option<HWND>) = (current_pid, None);
        let _ = EnumWindows(Some(enum_callback), LPARAM(&mut data as *mut _ as isize));
        data.1
    }
}

#[cfg(target_os = "windows")]
fn with_taskbar<F>(f: F)
where
    F: FnOnce(&windows::Win32::UI::Shell::ITaskbarList3, windows::Win32::Foundation::HWND),
{
    use std::cell::RefCell;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{ITaskbarList3, TaskbarList};

    thread_local! {
        static TASKBAR: RefCell<Option<ITaskbarList3>> = RefCell::new(None);
    }

    let Some(hwnd) = find_main_hwnd() else { return };

    TASKBAR.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            unsafe {
                if let Ok(tb) = CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                    let _ = tb.HrInit();
                    *borrow = Some(tb);
                }
            }
        }
        if let Some(tb) = borrow.as_ref() {
            f(tb, hwnd);
        }
    });
}

#[cfg(target_os = "windows")]
fn set_progress_windows(percentage: u8) {
    use windows::Win32::UI::Shell::TBPF_NORMAL;

    with_taskbar(|taskbar, hwnd| unsafe {
        let _ = taskbar.SetProgressState(hwnd, TBPF_NORMAL);
        let _ = taskbar.SetProgressValue(hwnd, percentage as u64, 100u64);
    });
}

#[cfg(target_os = "windows")]
fn clear_progress_windows() {
    use windows::Win32::UI::Shell::TBPF_NOPROGRESS;

    with_taskbar(|taskbar, hwnd| unsafe {
        let _ = taskbar.SetProgressState(hwnd, TBPF_NOPROGRESS);
    });
}

#[cfg(target_os = "macos")]
fn set_progress_macos(percentage: u8) {
    use objc2_app_kit::NSApplication;
    use objc2_foundation::NSString;

    unsafe {
        let app = NSApplication::sharedApplication();
        let dock_tile = app.dockTile();
        let label = if percentage > 0 && percentage < 100 {
            NSString::from_str(&format!("{}%", percentage))
        } else {
            NSString::from_str("")
        };
        dock_tile.setBadgeLabel(Some(&label));
    }
}

#[cfg(target_os = "macos")]
fn clear_progress_macos() {
    use objc2_app_kit::NSApplication;
    use objc2_foundation::NSString;

    unsafe {
        let app = NSApplication::sharedApplication();
        let dock_tile = app.dockTile();
        let empty = NSString::from_str("");
        dock_tile.setBadgeLabel(Some(&empty));
    }
}

#[cfg(target_os = "linux")]
fn set_progress_linux(percentage: u8) {
    tokio::task::spawn(async move {
        let _ = update_unity_launcher(percentage as f64 / 100.0, true).await;
    });
}

#[cfg(target_os = "linux")]
fn clear_progress_linux() {
    tokio::task::spawn(async move {
        let _ = update_unity_launcher(0.0, false).await;
    });
}

#[cfg(target_os = "linux")]
async fn update_unity_launcher(progress: f64, visible: bool) -> Result<(), zbus::Error> {
    use zbus::Connection;
    use zbus::zvariant::Value;
    use std::collections::HashMap;

    let conn = Connection::session().await?;

    let mut props: HashMap<&str, Value<'_>> = HashMap::new();
    props.insert("progress-visible", Value::from(visible));
    props.insert("progress", Value::from(progress));

    let app_uri = "application://collapseloader.desktop";

    conn.call_method(
        Some("com.canonical.Unity.LauncherEntry"),
        "/com/canonical/unity/launcherentry/1",
        Some("com.canonical.Unity.LauncherEntry"),
        "Update",
        &(app_uri, props),
    )
    .await?;

    Ok(())
}
