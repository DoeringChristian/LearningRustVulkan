
pub mod instance;
pub mod utils;
pub mod descriptors;
pub mod adapter;
pub mod device;
pub mod surface;
pub mod swapchain;
pub mod image;
pub mod framebuffer;
pub mod renderpass;
pub mod barrier;
pub mod buffer;
pub mod commandbuffer;

use arrayvec::ArrayVec;
use fxhash::FxHashMap;
pub use self::instance::*;
pub use self::descriptors::*;
pub use self::adapter::*;
pub use self::device::*;
pub use self::surface::*;
pub use self::swapchain::*;
pub use self::image::*;
pub use self::framebuffer::*;
pub use self::renderpass::*;
pub use self::barrier::*;
pub use self::buffer::*;
pub use self::commandbuffer::*;

use std::ffi::{CStr, CString};

use ash::extensions::{khr, ext};
use ash::vk;
use std::sync::{Arc, Mutex};

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
    pub adapter: Arc<Adapter>,
    pub global_allocator: Arc<Mutex<gpu_allocator::vulkan::Allocator>>,
    pub global_queue: vk::Queue,
    pub queue_family_index: u32,
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

#[derive(Deref, DerefMut)]
pub struct SwapchainImage{
    #[deref]
    #[deref_mut]
    pub image: Arc<Image>,
    pub image_index: usize,
    pub acquire_semaphore: vk::Semaphore,
    pub rendering_finished_semaphore: vk::Semaphore,
}

// TODO: Implement
pub struct Image{
    pub image: vk::Image,
    pub desc: ImageDesc,
    pub views: Mutex<FxHashMap<ImageViewDesc, ImageView>>,
    pub device: Arc<Device>,
}

pub struct ImageSubresourceData<'a>{
    pub data: &'a [u8],
    pub row_pitch: usize,
    pub slice_pitch: usize,
}

#[derive(Deref, DerefMut, Clone)]
pub struct ImageView{
    #[deref]
    #[deref_mut]
    pub view: vk::ImageView,
    pub desc: ImageViewDesc,
    pub fb_attachment_desc: FramebufferAttachmentDesc,
    pub image_desc: ImageDesc,
}

pub const MAX_COLOR_ATTACHMENTS: usize = 8;


pub struct FramebufferCache{
    entries: Mutex<FxHashMap<FramebufferCacheKey, vk::Framebuffer>>,
    attachment_desc: ArrayVec<vk::AttachmentDescription, MAX_COLOR_ATTACHMENTS>,
    render_pass: vk::RenderPass,
    color_attachment_count: usize,
}

#[derive(Eq, PartialEq, Hash)]
pub struct FramebufferCacheKey{
    pub extent: vk::Extent2D,
    pub attachments: ArrayVec<FramebufferAttachmentDesc, MAX_COLOR_ATTACHMENTS>,
}

#[derive(Deref, DerefMut)]
pub struct RenderPass{
    #[deref]
    #[deref_mut]
    pub rpass: vk::RenderPass,
    pub framebuffer_cache: FramebufferCache,
    pub device: Arc<Device>,
}

pub struct CommandBuffer{
    pub buffer: vk::CommandBuffer,
    pub pool: vk::CommandPool,
    pub submit_done_fence: vk::Fence,
    pub device: Arc<Device>,
}
