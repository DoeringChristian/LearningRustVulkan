
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::AllocatorDebugSettings;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub trait SharedAdapter{
    fn request_device(&self) -> (Arc<Device>, Arc<Queue>);
}

impl SharedAdapter for Arc<Adapter>{
    fn request_device(&self) -> (Arc<Device>, Arc<Queue>) {
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

            let device: ash::Device = self.instance.instance
                .create_device(self.pdevice, &device_create_info, None)
                .unwrap();

            let queue = device.get_device_queue(self.queue_family_index as u32, 0);

            let global_allocator = Allocator::new(&AllocatorCreateDesc{
                instance: self.instance.instance.clone(),
                device: device.clone(),
                physical_device: self.pdevice,
                debug_settings: AllocatorDebugSettings{
                    log_leaks_on_shutdown: false,
                    log_memory_information: true,
                    log_allocations: true,
                    ..Default::default()
                },
                buffer_device_address: true,
            }).unwrap();

            let global_allocator = Arc::new(Mutex::new(global_allocator));

            let device = Arc::new(Device{
                global_allocator,
                device,
                instance: self.instance.clone(),
                adapter: self.clone(),
            });

            let queue = Arc::new(Queue{
                queue,
                device: device.clone(),
                family_index: self.queue_family_index,
            });

            (device, queue)
        }
    }
}

