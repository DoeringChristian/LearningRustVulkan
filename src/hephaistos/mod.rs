
pub mod instance;
pub mod utils;
pub mod descriptors;
pub mod adapter;
pub mod device;
pub mod surface;
pub mod swapchain;

use fxhash::FxHashMap;
pub use instance::*;
pub use descriptors::*;
pub use adapter::*;
pub use device::*;
pub use surface::*;
pub use swapchain::*;

use std::ffi::{CStr, CString};

use ash::extensions::{khr, ext};
use ash::vk;
use std::sync::{Arc, Mutex};
use gpu_allocator::vulkan::*;

use derive_more::*;

pub struct Instance{
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug_call_back: Option<vk::DebugUtilsMessengerEXT>,
    pub debug_utils_loader: Option<ext::DebugUtils>,
}

#[derive(Deref, DerefMut)]
pub struct Surface{
    #[deref]
    #[deref_mut]
    pub surface: ash::vk::SurfaceKHR,
    pub surface_loader: khr::Surface,
    pub instance: Arc<Instance>,
    pub swapchain: Option<Swapchain>,
}

pub struct Adapter{
    pub pdevice: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub instance: Arc<Instance>,
}

#[derive(Deref, DerefMut)]
pub struct Device{
    #[deref]
    #[deref_mut]
    pub device: ash::Device,
    pub instance: Arc<Instance>,
    //pub global_allocator: Allocator,
}

#[derive(Deref, DerefMut)]
pub struct Queue{
    #[deref]
    #[deref_mut]
    pub queue: vk::Queue,
    pub device: Arc<Device>,
    pub family_index: u32,
}

pub struct Swapchain{
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: khr::Swapchain,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub device: Arc<Device>,
    pub images: Vec<Arc<Image>>,
    pub acquire_semaphores: Vec<vk::Semaphore>,
    pub rendering_finished_semaphores: Vec<vk::Semaphore>,
    pub next_semaphore: Mutex<usize>,
}

pub struct SwapchainImage{
    pub image: Arc<Image>,
    pub image_index: usize,
    pub acquire_semaphore: vk::Semaphore,
    pub rendering_finished_semaphore: vk::Semaphore,
}

// TODO: Implement
pub struct Image{
    pub image: vk::Image,
    pub desc: ImageDescriptor,
}
