
pub mod instance;
pub mod utils;
pub mod descriptors;
pub mod adapter;

pub use instance::*;
pub use descriptors::*;
pub use adapter::*;

use std::ffi::{CStr, CString};

use ash::extensions::{khr, ext};
use ash::vk;
use std::sync::Arc;

use derive_more::*;

pub struct InstanceShared{
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug_call_back: Option<vk::DebugUtilsMessengerEXT>,
    pub debug_utils_loader: Option<ext::DebugUtils>,
}

#[derive(Deref, DerefMut, Clone)]
pub struct Instance(Arc<InstanceShared>);

#[derive(Deref, DerefMut)]
pub struct Surface{
    #[deref]
    #[deref_mut]
    pub surface: ash::vk::SurfaceKHR,
    pub surface_loader: khr::Surface,
    pub instance: Instance,
}

pub struct Adapter{
    pub pdevice: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub instance: Instance,
}

#[derive(Deref, DerefMut)]
pub struct DeviceShared{
    #[deref]
    #[deref_mut]
    pub device: ash::Device,
    pub instance: Instance,
}

#[derive(Deref, DerefMut, Clone)]
pub struct Device(Arc<DeviceShared>);

#[derive(Deref, DerefMut)]
pub struct Queue{
    #[deref]
    #[deref_mut]
    pub queue: vk::Queue,
    pub device: Device,
    pub family_index: u32,
}
