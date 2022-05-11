
use super::*;
use ash::vk;

pub struct AdapterDescriptor<'a>{
    pub compatible_surface: Option<&'a Surface>,
    pub queue_flags: vk::QueueFlags,
    //pub limits: vk::PhysicalDeviceLimits,
}

pub enum ImageType{
    Tex2d,
}

pub struct ImageDescriptor{
    pub image_type: vk::ImageType,
    pub usage: vk::ImageUsageFlags,
    pub flags: vk::ImageCreateFlags,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub tiling: vk::ImageTiling,
    pub mip_levels: u32,
    pub array_elements: u32,
}
