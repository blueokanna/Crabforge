//! C ABI from Flutter 3.47's flutter_windows.h. CMake links the matching engine.
use std::ffi::{c_char, c_void};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

pub type Handle = *mut c_void;

#[repr(C)]
#[derive(Default)]
pub struct Properties {
    pub assets_path: *const u16,
    pub icu_data_path: *const u16,
    pub aot_library_path: *const u16,
    pub dart_entrypoint: *const c_char,
    pub dart_entrypoint_argc: i32,
    pub dart_entrypoint_argv: *const *const c_char,
    pub gpu_preference: i32,
    pub ui_thread_policy: i32,
    pub accessibility_mode: i32,
    pub impeller_switch: i32,
    pub enable_flutter_gpu: bool,
}

unsafe extern "C" {
    pub fn FlutterDesktopEngineCreate(properties: &Properties) -> Handle;
    pub fn FlutterDesktopViewControllerCreate(width: i32, height: i32, engine: Handle) -> Handle;
    pub fn FlutterDesktopViewControllerDestroy(controller: Handle);
    pub fn FlutterDesktopViewControllerGetView(controller: Handle) -> Handle;
    pub fn FlutterDesktopViewGetHWND(view: Handle) -> HWND;
    pub fn FlutterDesktopViewControllerHandleTopLevelWindowProc(
        controller: Handle,
        window: HWND,
        msg: u32,
        wp: WPARAM,
        lp: LPARAM,
        result: &mut LRESULT,
    ) -> bool;
    pub fn FlutterDesktopEngineReloadSystemFonts(engine: Handle);
    pub fn FlutterDesktopEngineGetPluginRegistrar(
        engine: Handle,
        plugin_name: *const c_char,
    ) -> Handle;
    pub fn FlutterDesktopEngineSetNextFrameCallback(
        engine: Handle,
        callback: unsafe extern "C" fn(Handle),
        data: Handle,
    );
    pub fn FlutterDesktopViewControllerForceRedraw(controller: Handle);
    pub fn FlutterDesktopResyncOutputStreams();
}

// The matching Flutter plugin is linked by windows/flutter/generated_plugins.cmake.
unsafe extern "C" {
    pub fn FileSelectorWindowsRegisterWithRegistrar(registrar: Handle);
}
