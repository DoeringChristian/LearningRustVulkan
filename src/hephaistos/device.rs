use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::AllocationCreateDesc;
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

impl Device {
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

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            println!("device")
        }
    }
}
