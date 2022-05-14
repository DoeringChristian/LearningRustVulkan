use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::AllocationCreateDesc;
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

impl From<Arc<SharedDevice>> for RenderDevice{
    fn from(src: Arc<SharedDevice>) -> Self {
        Self{
            shared: src.clone(),
            frames: [
                Mutex::new(Arc::new(DeviceFrame{
                    main_cb: src.create_command_buffer(),
                })),
                Mutex::new(Arc::new(DeviceFrame{
                    main_cb: src.create_command_buffer(),
                })),
            ],
            setup_cb: src.create_command_buffer(),
        }
    }
}

impl RenderDevice{
    pub fn with_setup_cb(&self, callback: impl FnOnce(vk::CommandBuffer)){
        self.shared.with_commandbuffer_wait_idle(&self.setup_cb, callback)
    }
}

pub trait RequestDevice{
    fn request_device(&self) -> Arc<RenderDevice>;
}

impl RequestDevice for Arc<Adapter>{
    fn request_device(&self) -> Arc<RenderDevice>{
        unsafe{
            let device_extension_names_raw = [
                khr::Swapchain::name().as_ptr(),
                vk::KhrImagelessFramebufferFn::name().as_ptr(),
                vk::KhrBufferDeviceAddressFn::name().as_ptr(),
            ];

            let mut buffer_device_address_feature = vk::PhysicalDeviceBufferDeviceAddressFeatures::default();
            let mut features2 = vk::PhysicalDeviceFeatures2::builder()
                .push_next(&mut buffer_device_address_feature)
                .build();

            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(self.queue_family_index)
                .queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                //.enabled_features(&features)
                .push_next(&mut features2);

            let device: ash::Device = self.instance.raw
                .create_device(self.pdevice, &device_create_info, None)
                .unwrap();

            let queue = device.get_device_queue(self.queue_family_index as u32, 0);

            let global_allocator = gpu_allocator::vulkan::Allocator::new(&gpu_allocator::vulkan::AllocatorCreateDesc{
                instance: self.instance.raw.clone(),
                device: device.clone(),
                physical_device: self.pdevice,
                debug_settings: gpu_allocator::AllocatorDebugSettings{
                    log_leaks_on_shutdown: false,
                    log_memory_information: true,
                    log_allocations: true,
                    ..Default::default()
                },
                buffer_device_address: true,
            }).unwrap();

            let global_allocator = Arc::new(Mutex::new(global_allocator));

            let shared = Arc::new(SharedDevice{
                global_allocator,
                raw: device,
                instance: self.instance.clone(),
                adapter: self.clone(),
                global_queue: queue,
                queue_family_index: self.queue_family_index,
            });

            Arc::new(RenderDevice{
                shared: shared.clone(),
                frames: [
                    Mutex::new(Arc::new(DeviceFrame{
                        main_cb: shared.create_command_buffer(),
                    })),
                    Mutex::new(Arc::new(DeviceFrame{
                        main_cb: shared.create_command_buffer(),
                    })),
                ],
                setup_cb: shared.create_command_buffer(),
            })
        }
    }
}

impl SharedDevice {
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

impl Drop for SharedDevice {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
            println!("device")
        }
    }
}
