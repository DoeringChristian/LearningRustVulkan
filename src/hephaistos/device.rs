use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::AllocationCreateDesc;
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

impl Device {
    pub fn with_commandbuffer_wait_idle(
        &self,
        commandbuffer: &CommandBuffer,
        callback: impl FnOnce(vk::CommandBuffer),
    ){
        unsafe{
            self.raw.begin_command_buffer(
                commandbuffer.raw,
                &vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            ).unwrap();
        }

        callback(commandbuffer.raw);

        unsafe{
            self.raw.end_command_buffer(commandbuffer.raw).unwrap();
            
            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&[commandbuffer.raw])
                .build();

            self.raw.queue_submit(self.global_queue, &[submit_info], vk::Fence::null())
                .expect("Could not submit queue.");

            self.raw.device_wait_idle();
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

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
            println!("device")
        }
    }
}
