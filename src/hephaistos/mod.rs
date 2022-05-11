
pub mod instance;
pub mod utils;

pub use instance::*;

use std::ffi::{CStr, CString};

use ash::extensions::{khr, ext};
use ash::vk;

pub struct Instance{
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug_call_back: Option<vk::DebugUtilsMessengerEXT>,
    pub debug_utils_loader: Option<ext::DebugUtils>,
}

pub struct Surface{
    pub surface: ash::vk::SurfaceKHR,
    pub surface_loader: khr::Surface,
}

