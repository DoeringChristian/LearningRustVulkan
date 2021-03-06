
use super::*;
use ash::vk;

pub struct AdapterDesc<'a>{
    pub compatible_surface: Option<&'a Surface>,
    pub queue_flags: vk::QueueFlags,
    //pub limits: vk::PhysicalDeviceLimits,
}

#[derive(Copy, Clone)]
pub enum ImageType{
    Tex1d,
    Tex1dArray,
    Tex2d,
    Tex2dArray,
    Tex3d,
    Cube,
    CubeArray,
}

#[derive(Clone, Copy)]
pub struct ImageDesc{
    pub image_type: ImageType,
    pub usage: vk::ImageUsageFlags,
    pub flags: vk::ImageCreateFlags,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub tiling: vk::ImageTiling,
    pub mip_levels: u32,
    pub array_elements: u32,
}

impl Default for ImageDesc{
    fn default() -> Self {
        Self{
            image_type: ImageType::Tex2d,
            usage: Default::default(),
            flags: Default::default(),
            format: Default::default(),
            extent: Default::default(),
            tiling: Default::default(),
            mip_levels: 1,
            array_elements: 1,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageViewDesc{
    pub view_type: Option<vk::ImageViewType>,
    pub format: Option<vk::Format>,
    pub aspect_mask: vk::ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: Option<u32>,
}

pub struct RenderPassDesc<'a>{
    pub color_attachments: &'a [vk::AttachmentDescription],
    pub depth_attachment: Option<vk::AttachmentDescription>,
}

pub struct RenderPassBeginnDesc<'a>{
    pub color_attachments: &'a [&'a ImageView],
    pub depth_attachment: Option<&'a ImageView>,
    pub area: vk::Rect2D,
    pub clear_values: &'a [vk::ClearValue],
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FramebufferAttachmentDesc{
    pub flgas: vk::ImageCreateFlags,
    pub usage: vk::ImageUsageFlags,
    pub layer_count: u32,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct BufferDesc<'a> {
    pub label: Option<&'a str>,
    pub size: usize,
    pub usage: vk::BufferUsageFlags,
    pub memory_location: gpu_allocator::MemoryLocation,
}

impl<'a> From<BufferDesc<'a>> for BufferDescInt {
    fn from(src: BufferDesc<'a>) -> Self {
        BufferDescInt {
            label: src.label.map(|s| String::from(s)),
            size: src.size,
            usage: src.usage,
            memory_location: src.memory_location,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct BufferDescInt {
    pub label: Option<String>,
    pub size: usize,
    pub usage: vk::BufferUsageFlags,
    pub memory_location: gpu_allocator::MemoryLocation,
}
