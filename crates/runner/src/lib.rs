#![cfg(windows)]

mod flutter;

use flutter::*;
use std::{
    ffi::{CString, c_void},
    mem::size_of,
    ptr::{null, null_mut},
};
use windows_sys::{
    Win32::{
        Foundation::*,
        Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute},
        System::{Com::*, Console::*, Diagnostics::Debug::IsDebuggerPresent, Registry::*},
        UI::{
            HiDpi::GetDpiForSystem, Input::KeyboardAndMouse::SetFocus,
            Shell::SetCurrentProcessExplicitAppUserModelID, WindowsAndMessaging::*,
        },
    },
    w,
};

#[derive(Default)]
struct Window {
    engine: Handle,
    controller: Handle,
    child: HWND,
}

unsafe fn update_theme(hwnd: HWND) {
    let mut light = 1u32;
    let mut len = size_of::<u32>() as u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            null_mut(),
            (&raw mut light).cast(),
            &mut len,
        );
        let dark = i32::from(light == 0);
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            (&raw const dark).cast(),
            size_of::<i32>() as u32,
        );
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    // Calls into Win32 and Flutter can re-enter this function. Copy the handles
    // before dispatch so no Rust reference is held during re-entrant callbacks.
    unsafe {
        let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Window;
        if !state.is_null() {
            let Window {
                controller,
                engine,
                child,
            } = *state;
            let mut result = 0;
            if !controller.is_null()
                && FlutterDesktopViewControllerHandleTopLevelWindowProc(
                    controller,
                    hwnd,
                    msg,
                    wp,
                    lp,
                    &mut result,
                )
            {
                return result;
            }
            match msg {
                WM_SIZE if !child.is_null() => {
                    let mut rect = RECT::default();
                    GetClientRect(hwnd, &mut rect);
                    MoveWindow(child, 0, 0, rect.right, rect.bottom, 1);
                    return 0;
                }
                WM_SETFOCUS if !child.is_null() => {
                    SetFocus(child);
                    return 0;
                }
                WM_DPICHANGED => {
                    let rect = *(lp as *const RECT);
                    SetWindowPos(
                        hwnd,
                        null_mut(),
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        SWP_NOACTIVATE | SWP_NOZORDER,
                    );
                    return 0;
                }
                WM_FONTCHANGE if !engine.is_null() => FlutterDesktopEngineReloadSystemFonts(engine),
                WM_SETTINGCHANGE | WM_DWMCOLORIZATIONCOLORCHANGED => update_theme(hwnd),
                WM_DESTROY => {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    PostQuitMessage(0);
                    return 0;
                }
                _ => (),
            }
        }
        DefWindowProcW(hwnd, msg, wp, lp)
    }
}

unsafe extern "C" fn show(hwnd: Handle) {
    unsafe {
        ShowWindow(hwnd, SW_SHOWNORMAL);
    }
}

unsafe fn run(instance: HINSTANCE) -> i32 {
    let arguments: Vec<_> = std::env::args_os()
        .skip(1)
        .map(|s| {
            CString::new(s.to_string_lossy().as_bytes())
                .expect("process arguments cannot contain NUL")
        })
        .collect();
    let argv: Vec<_> = arguments.iter().map(|s| s.as_ptr()).collect();
    let properties = Properties {
        assets_path: w!("data\\flutter_assets"),
        icu_data_path: w!("data\\icudtl.dat"),
        aot_library_path: w!("data\\app.so"),
        dart_entrypoint_argc: argv.len() as i32,
        dart_entrypoint_argv: argv.as_ptr(),
        ..Default::default()
    };
    unsafe {
        let class = WNDCLASSW {
            hInstance: instance,
            lpszClassName: w!("FLUTTER_RUNNER_WIN32_WINDOW"),
            lpfnWndProc: Some(window_proc),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hIcon: LoadIconW(instance, 101 as *const u16),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return 1;
        }
        let dpi = GetDpiForSystem() as i32;
        let hwnd = CreateWindowExW(
            0,
            class.lpszClassName,
            w!("crabforge"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1280 * dpi / 96,
            720 * dpi / 96,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return 1;
        }
        update_theme(hwnd);
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect);
        let engine = FlutterDesktopEngineCreate(&properties);
        if engine.is_null() {
            DestroyWindow(hwnd);
            return 1;
        }
        let controller = FlutterDesktopViewControllerCreate(rect.right, rect.bottom, engine);
        if controller.is_null() {
            DestroyWindow(hwnd);
            return 1;
        }
        let child = FlutterDesktopViewGetHWND(FlutterDesktopViewControllerGetView(controller));
        let state = Window {
            engine,
            controller,
            child,
        };
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, (&raw const state) as isize);
        SetParent(child, hwnd);
        MoveWindow(child, 0, 0, rect.right, rect.bottom, 1);
        SetFocus(child);
        FlutterDesktopEngineSetNextFrameCallback(engine, show, hwnd);
        FlutterDesktopViewControllerForceRedraw(controller);
        let mut msg = MSG::default();
        let exit = loop {
            match GetMessageW(&mut msg, null_mut(), 0, 0) {
                -1 => break 1,
                0 => break msg.wParam as i32,
                _ => {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        };
        if IsWindow(hwnd) != 0 {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DestroyWindow(hwnd);
        }
        FlutterDesktopViewControllerDestroy(controller);
        exit
    }
}

/// Entrypoint invoked by the Windows CRT with a live process instance handle.
///
/// # Safety
/// Only the process startup routine may invoke this on the platform thread.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn wWinMain(
    instance: HINSTANCE,
    _: HINSTANCE,
    _: *mut u16,
    _: i32,
) -> i32 {
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0
            && IsDebuggerPresent() != 0
            && AllocConsole() != 0
        {
            FlutterDesktopResyncOutputStreams();
        }
        let initialized = CoInitializeEx(null::<c_void>(), COINIT_APARTMENTTHREADED as u32) >= 0;
        SetCurrentProcessExplicitAppUserModelID(w!("com.blue.crabforge"));
        let result = run(instance);
        if initialized {
            CoUninitialize();
        }
        result
    }
}
