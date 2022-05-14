use super::*;

pub trait CreateCommandBuffer {
    fn create_command_buffer(&self) -> CommandBuffer;
}

impl CreateCommandBuffer for Arc<Device> {
    fn create_command_buffer(&self) -> CommandBuffer {
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(self.queue_family_index);

        let pool = unsafe {
            self.raw
                .create_command_pool(&pool_create_info, None)
                .expect("Could not create command pool")
        };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffer = unsafe {
            self.raw
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Could not allocate command_buffer.")[0]
        };

        let submit_done_fence = unsafe {
            self.raw
                .create_fence(
                    &vk::FenceCreateInfo::builder()
                        .flags(vk::FenceCreateFlags::SIGNALED)
                        .build(),
                    None,
                )
                .expect("Could not create submit_done_fence for CommandBuffer")
        };

        CommandBuffer {
            raw: command_buffer,
            pool,
            submit_done_fence,
            device: self.clone(),
        }
    }
}
