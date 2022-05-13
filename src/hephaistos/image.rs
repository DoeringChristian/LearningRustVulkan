
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::borrow::BorrowMut;
use std::sync::Arc;

impl ImageView{
}

impl Image{
    pub fn get_view_create_info(&self, desc: &ImageViewDesc) -> vk::ImageViewCreateInfo{
        vk::ImageViewCreateInfo {
            format: self.desc.format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            view_type: desc.view_type.unwrap_or_else(|| {
                convert_image_type_to_view_type(self.desc.image_type)
            }),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: desc.aspect_mask,
                base_mip_level: desc.base_mip_level,
                level_count: desc
                    .level_count
                    .unwrap_or(self.desc.mip_levels as u32),
                    base_array_layer: 0,
                    layer_count: match self.desc.image_type {
                        ImageType::Cube | ImageType::CubeArray => 6,
                        _ => 1,
                    },
            },
            image: self.image,
            ..Default::default()
        }
    }
}

impl Drop for Image{
    fn drop(&mut self) {
        unsafe{
            for image_view in self.views.lock().unwrap().iter(){
                self.device.destroy_image_view(image_view.1.view, None);
            }
            self.device.destroy_image(self.image, None);
        }
    }
}

pub fn get_image_create_info(desc: &ImageDesc, initial_data: bool) -> vk::ImageCreateInfo {
    let (image_type, image_extent, image_layers) = match desc.image_type {
        ImageType::Tex1d => (
            vk::ImageType::TYPE_1D,
            vk::Extent3D {
                width: desc.extent.width,
                height: 1,
                depth: 1,
            },
            1,
        ),
        ImageType::Tex1dArray => (
            vk::ImageType::TYPE_1D,
            vk::Extent3D {
                width: desc.extent.width,
                height: 1,
                depth: 1,
            },
            desc.array_elements,
        ),
        ImageType::Tex2d => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            1,
        ),
        ImageType::Tex2dArray => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            desc.array_elements,
        ),
        ImageType::Tex3d => (
            vk::ImageType::TYPE_3D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: desc.extent.depth as u32,
            },
            1,
        ),
        ImageType::Cube => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            6,
        ),
        ImageType::CubeArray => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            6 * desc.array_elements,
        ),
    };

    let mut image_usage = desc.usage;

    if initial_data {
        image_usage |= vk::ImageUsageFlags::TRANSFER_DST;
    }

    vk::ImageCreateInfo {
        flags: desc.flags,
        image_type,
        format: desc.format,
        extent: image_extent,
        mip_levels: desc.mip_levels as u32,
        array_layers: image_layers as u32,
        samples: vk::SampleCountFlags::TYPE_1, // TODO: desc.sample_count
        tiling: desc.tiling,
        usage: image_usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        initial_layout: match initial_data {
            true => vk::ImageLayout::PREINITIALIZED,
            false => vk::ImageLayout::UNDEFINED,
        },
        ..Default::default()
    }
}
