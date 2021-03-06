
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub trait SharedSurface{
    fn create_swapchain(&mut self, device: &RenderDevice, adapter: Arc<Adapter>);
    fn acquire_next_image(&self) -> Option<SwapchainImage>;
    fn present_image(&self, image: SwapchainImage);
}

impl SharedSurface for Arc<Surface>{
    fn create_swapchain(&mut self, device: &RenderDevice, adapter: Arc<Adapter>) {
        unsafe{
            let surface_format = self.loader
                .get_physical_device_surface_formats(adapter.pdevice, self.raw)
                .unwrap()[0];

            let surface_capabilities = self.loader
                .get_physical_device_surface_capabilities(adapter.pdevice, self.raw)
                .unwrap();
            let mut desired_image_count = 3.max(surface_capabilities.min_image_count);
            if surface_capabilities.max_image_count > 0
                //&& desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = desired_image_count.min(surface_capabilities.max_image_count);
            }
            let extent = match surface_capabilities.current_extent.width {
                std::u32::MAX => panic!("Could not get surface_resolution"),
                _ => surface_capabilities.current_extent,
            };
            let pre_transform = if surface_capabilities
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capabilities.current_transform
            };
            let present_modes = self.loader
                .get_physical_device_surface_present_modes(adapter.pdevice, self.raw)
                .unwrap();
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);
            let swapchain_loader = khr::Swapchain::new(&self.instance.raw, &device);

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(self.raw)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(extent)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();

            let images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
            let images: Vec<Arc<Image>> = images.into_iter().enumerate().map(|(i, vk_image)|{
                Arc::new(Image{
                    raw: vk_image,
                    desc: ImageDesc{
                        image_type: ImageType::Tex2d,
                        usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                        flags: vk::ImageCreateFlags::empty(),
                        format: vk::Format::B8G8R8A8_SRGB,
                        extent: vk::Extent3D{
                            width: extent.width,
                            height: extent.height,
                            depth: 0,
                        },
                        tiling: vk::ImageTiling::OPTIMAL,
                        mip_levels: 1,
                        array_elements: 1,
                    },
                    views: Mutex::new(FxHashMap::default()),
                    device: device.shared.clone(),
                },
                )
            }).collect();

            let acquire_semaphores: Vec<vk::Semaphore> = images.iter().map(|_|{
                device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap()
            }).collect();
            let present_complete_semaphores: Vec<vk::Semaphore> = images.iter().map(|_|{
                device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap()
            }).collect();

            //assert_eq!(desired_image_count, images.len() as u32);

            let surface = Arc::get_mut(self).unwrap();
            surface.swapchain = Some(Swapchain{
                raw: swapchain,
                loader: swapchain_loader,
                surface_format,
                images,
                acquire_semaphores,
                rendering_finished_semaphores: present_complete_semaphores,
                extent,
                device: device.shared.clone(),
                next_semaphore: Mutex::new(0),
            })
        }
    }

    fn acquire_next_image(&self) -> Option<SwapchainImage> {
        self.swapchain.as_ref().unwrap().acquire_next_image()
    }
    fn present_image(&self, image: SwapchainImage) {
        self.swapchain.as_ref().unwrap().present_image(image)
    }
}

