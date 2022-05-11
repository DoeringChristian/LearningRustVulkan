
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub trait SharedAdapter{
    fn request_device(&self) -> (Arc<Device>, Arc<Queue>);
}

impl SharedAdapter for Arc<Adapter>{
    fn request_device(&self) -> (Arc<Device>, Arc<Queue>) {
        unsafe{


            let device_extension_names_raw = [khr::Swapchain::name().as_ptr()];
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
                .enabled_features(&features);

            let device: ash::Device = self.instance.instance
                .create_device(self.pdevice, &device_create_info, None)
                .unwrap();

            let queue = device.get_device_queue(self.queue_family_index as u32, 0);

            let device = Arc::new(Device{
                device,
                instance: self.instance.clone(),
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

