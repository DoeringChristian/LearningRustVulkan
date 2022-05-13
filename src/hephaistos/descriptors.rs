
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageViewDesc{
    pub view_type: Option<vk::ImageViewType>,
    pub format: Option<vk::Format>,
    pub aspect_mask: vk::ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: Option<u32>,
}

/*
#[derive(Default, Copy, Clone)]
pub struct RenderPassAttachmentDesc{
    pub flags:            vk::AttachmentDescriptionFlags,
    pub format:           vk::Format,
    pub samples:          vk::SampleCountFlags,
    pub load_op:          vk::AttachmentLoadOp,
    pub store_op:         vk::AttachmentStoreOp,
    pub stencil_load_op:  vk::AttachmentLoadOp,
    pub stencil_store_op: vk::AttachmentStoreOp,
    pub initial_layout:   vk::ImageLayout,
    pub final_layout:     vk::ImageLayout,
}

impl RenderPassAttachmentDesc{
    pub fn to_vk(
        self, 
        initial_layout: vk::ImageLayout,
        final_layout: vk::ImageLayout,
    ) -> vk::AttachmentDescription{
        vk::AttachmentDescription{
            format: self.format,
            samples: self.samples,
            load_op: self.load_op,
            initial_layout,
            final_layout,
            ..Default::default()
        }
    }
}
*/

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
