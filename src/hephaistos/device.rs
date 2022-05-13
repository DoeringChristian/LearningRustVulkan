use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::MemoryLocation;
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub trait SharedDevice {
    fn create_render_pass(&self, desc: &RenderPassDesc<'_>) -> Arc<RenderPass>;
    fn create_image(&self, desc: &ImageDesc, data: Vec<ImageSubresourceData>) -> Image;
}

impl SharedDevice for Arc<Device> {
    fn create_render_pass(&self, desc: &RenderPassDesc<'_>) -> Arc<RenderPass> {
        let renderpass_attachments = desc
            .color_attachments
            .iter()
            .map(|a| {
                a.to_vk(
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                )
            })
            .chain(desc.depth_attachment.as_ref().map(|a| {
                a.to_vk(
                    vk::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
                    vk::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
                )
            }))
            .collect::<Vec<_>>();

        let color_attachment_refs = (0..desc.color_attachments.len() as u32)
            .map(|attachment| vk::AttachmentReference {
                attachment,
                layout: desc.color_attachments[attachment as usize].layout,
            })
            .collect::<Vec<_>>();

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: desc.color_attachments.len() as u32,
            layout: desc.depth_attachment.unwrap().layout,
        };

        let mut subpass_description = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        if desc.depth_attachment.is_some() {
            subpass_description =
                subpass_description.depth_stencil_attachment(&depth_attachment_ref);
        }
        let subpass_description = subpass_description.build();

        let subpasses = [subpass_description];
        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(&subpasses);

        let render_pass = unsafe {
            self.device
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        Arc::new(RenderPass {
            rpass: render_pass,
            framebuffer_cache: FramebufferCache::new(
                render_pass,
                desc.color_attachments,
                desc.depth_attachment,
            ),
            device: self.clone(),
        })
    }
    fn create_image(&self, desc: &ImageDesc, data: Vec<ImageSubresourceData>) -> Image{
        unsafe{
            let create_info = get_image_create_info(desc, !data.is_empty());
            
            let image = self.device.create_image(&create_info, None).expect("Image Creation Error");

            let requirements = self.device.get_image_memory_requirements(image);

            let allocation = self.global_allocator.lock().unwrap()
                .allocate(&AllocationCreateDesc{
                    name: "image",
                    requirements,
                    location: MemoryLocation::GpuOnly,
                    linear: false,
                }).expect("Alloocation Error");

            self.device.bind_image_memory(image, allocation.memory(), allocation.offset())
                .expect("Image bind error");

            // TODO: load image into memory.

            Image{
                image,
                desc: *desc,
                views: Mutex::new(FxHashMap::default()),
                device: self.clone(),
            }
        }
    }
}

impl Device {

    pub fn create_image_view(&self, image: &Image, desc: &ImageViewDesc) -> ImageView {
        unsafe {
            let mut views = image.views.lock().unwrap();
            views
                .entry(*desc)
                .or_insert(ImageView {
                    view: self
                        .device
                        .create_image_view(
                            &vk::ImageViewCreateInfo {
                                format: image.desc.format,
                                components: vk::ComponentMapping {
                                    r: vk::ComponentSwizzle::R,
                                    g: vk::ComponentSwizzle::G,
                                    b: vk::ComponentSwizzle::B,
                                    a: vk::ComponentSwizzle::A,
                                },
                                view_type: desc.view_type.unwrap_or_else(|| {
                                    convert_image_type_to_view_type(image.desc.image_type)
                                }),
                                subresource_range: vk::ImageSubresourceRange {
                                    aspect_mask: desc.aspect_mask,
                                    base_mip_level: desc.base_mip_level,
                                    level_count: desc
                                        .level_count
                                        .unwrap_or(image.desc.mip_levels as u32),
                                        base_array_layer: 0,
                                        layer_count: match image.desc.image_type {
                                            ImageType::Cube | ImageType::CubeArray => 6,
                                            _ => 1,
                                        },
                                },
                                image: image.image,
                                ..Default::default()
                            },
                            None,
                            )
                                .unwrap(),
                                desc: *desc,
                                image_desc: image.desc,
                })
            .clone()
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
            self.device.destroy_device(None);
            println!("device")
        }
    }
}
