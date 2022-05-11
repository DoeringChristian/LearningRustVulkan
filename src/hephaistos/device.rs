
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

impl Drop for Device{
    fn drop(&mut self) {
        unsafe{
            self.device.destroy_device(None);
            println!("device")
        }
    }
}
