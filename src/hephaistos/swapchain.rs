
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::borrow::BorrowMut;
use std::sync::Arc;

impl Swapchain{
    pub fn acquire_next_image(&self) -> Option<SwapchainImage>{
        unsafe{
            let next_semaphore = self.next_semaphore.lock().unwrap().clone();
            let acquire_semaphore = self.acquire_semaphores[next_semaphore];
            let rendering_finished_semaphore = self.rendering_finished_semaphores[next_semaphore];

            let present_index = self.swapchain_loader.acquire_next_image(
                self.swapchain, 
                std::u64::MAX,
                acquire_semaphore,
                vk::Fence::null())
                .map(|(val, _)| val as usize);

            match present_index{
                Ok(present_index) => {
                    *self.next_semaphore.lock().unwrap() = (next_semaphore + 1) % self.images.len();

                    Some(SwapchainImage{
                        image: self.images[present_index].clone(),
                        image_index: present_index,
                        acquire_semaphore,
                        rendering_finished_semaphore,
                    })
                },
                Err(err)
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR
                        || err == vk::Result::SUBOPTIMAL_KHR => 
                    {
                        None
                    },
                        err => {
                            panic!("Could not acquire next image in Swapchain: {:?}", err);
                    }
            }
        }
    }
    pub fn present_image(&self, image: SwapchainImage){
        unsafe{
            let wait_semaphors = [image.rendering_finished_semaphore];
            let swapchains = [self.swapchain];
            let image_indices = [image.image_index as u32];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphors)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain_loader
                .queue_present(self.device.global_queue, &present_info)
                .unwrap();

            }
    }
}

impl Drop for Swapchain{
    fn drop(&mut self) {
        unsafe{
            println!("Swapchain::drop");
            self.acquire_semaphores.iter_mut().for_each(|s|{
                self.device.destroy_semaphore(*s, None);
            });
            self.rendering_finished_semaphores.iter_mut().for_each(|s|{
                self.device.destroy_semaphore(*s, None);
            })
        }
    }
}
