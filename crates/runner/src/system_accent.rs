use std::{ffi::CStr, ptr::null_mut};

use windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor;

use crate::flutter::*;

const CHANNEL: &CStr = c"com.blue.crabforge/system-accent";

fn current() -> [u8; 4] {
    let mut argb = 0;
    let mut opaque = 0;
    // DWM returns an ARGB color with a translucency byte for window rendering.
    // Material's palette generator takes an opaque seed. Zero signals no seed.
    if unsafe { DwmGetColorizationColor(&mut argb, &mut opaque) } >= 0 {
        (argb | 0xff00_0000).to_le_bytes()
    } else {
        [0; 4]
    }
}

unsafe extern "C" fn reply(messenger: Handle, message: *const Message, _: Handle) {
    let response = unsafe { (*message).response_handle };
    if !response.is_null() {
        let color = current();
        unsafe {
            FlutterDesktopMessengerSendResponse(messenger, response, color.as_ptr(), color.len());
        }
    }
}

pub unsafe fn register(engine: Handle) -> Handle {
    unsafe {
        let messenger = FlutterDesktopEngineGetMessenger(engine);
        FlutterDesktopMessengerSetCallback(messenger, CHANNEL.as_ptr(), Some(reply), null_mut());
        messenger
    }
}

pub unsafe fn notify(messenger: Handle) {
    let color = current();
    unsafe {
        FlutterDesktopMessengerSend(messenger, CHANNEL.as_ptr(), color.as_ptr(), color.len());
    }
}
