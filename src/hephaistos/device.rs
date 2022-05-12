
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

impl Device{
    pub fn create_image(){
        todo!()
    }

    pub fn create_image_view(&self, image: Arc<Image>, desc: &ImageViewDesc) -> vk::ImageView{
        unsafe{
            let mut views = image.views.lock().unwrap();
            views.entry(*desc).or_insert(self.device.create_image_view(&vk::ImageViewCreateInfo{
                format: image.desc.format,
                components: vk::ComponentMapping{
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                view_type: desc.view_type.unwrap_or_else(|| convert_image_type_to_view_type(image.desc.image_type)),
                subresource_range: vk::ImageSubresourceRange{
                    aspect_mask: desc.aspect_mask,
                    base_mip_level: desc.base_mip_level,
                    level_count: desc.level_count.unwrap_or(image.desc.mip_levels as u32),
                    base_array_layer: 0,
                    layer_count: match image.desc.image_type{
                        ImageType::Cube | ImageType::CubeArray => 6,
                        _ => 1,
                    },
                },
                image: image.image,
                ..Default::default()
            }, None).unwrap()
            ).clone()
        }
    }
}

pub fn convert_image_type_to_view_type(image_type: ImageType) -> vk::ImageViewType {
    match image_type {
        ImageType::Tex1d => vk::ImageViewType::TYPE_1D,
        ImageType::Tex1dArray => vk::ImageViewType::TYPE_1D_ARRAY,
        ImageType::Tex2d => vk::ImageViewType::TYPE_2D,
        ImageType::Tex2dArray => vk::ImageViewType::TYPE_2D_ARRAY,
        ImageType::Tex3d => vk::ImageViewType::TYPE_3D,
        ImageType::Cube => vk::ImageViewType::CUBE,
        ImageType::CubeArray => vk::ImageViewType::CUBE_ARRAY,
    }
}

impl Drop for Device{
    fn drop(&mut self) {
        unsafe{
            self.device.destroy_device(None);
            println!("device")
        }
    }
}

