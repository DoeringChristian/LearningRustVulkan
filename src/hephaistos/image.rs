use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::vulkan::AllocationCreateDesc;
use gpu_allocator::MemoryLocation;
use raw_window_handle::HasRawWindowHandle;
use std::borrow::BorrowMut;
use std::sync::Arc;

pub trait CreateImage {
    fn create_image(&self, desc: &ImageDesc, data: Vec<ImageSubresourceData>) -> Image;
}

impl CreateImage for Arc<Device> {
    fn create_image(&self, desc: &ImageDesc, data: Vec<ImageSubresourceData>) -> Image {
        unsafe {
            let create_info = get_image_create_info(desc, !data.is_empty());

            let image = self
                .raw
                .create_image(&create_info, None)
                .expect("Image Creation Error");

            let requirements = self.raw.get_image_memory_requirements(image);

            let allocation = self
                .global_allocator
                .lock()
                .unwrap()
                .allocate(&AllocationCreateDesc {
                    name: "image",
                    requirements,
                    location: MemoryLocation::GpuOnly,
                    linear: false,
                })
                .expect("Alloocation Error");

            self.raw
                .bind_image_memory(image, allocation.memory(), allocation.offset())
                .expect("Image bind error");

            // TODO: load image into memory.
            if !data.is_empty(){
                let data_bytes: usize = data.iter().map(|d| d.data.len()).sum();

                let mut buffer = self.create_buffer(BufferDesc{
                    label: Some("Image Staging Buffer"),
                    size: data_bytes,
                    usage: vk::BufferUsageFlags::TRANSFER_SRC,
                    memory_location: gpu_allocator::MemoryLocation::CpuToGpu,
                }, None);
                let mapped_slice_mut = buffer.allocation.mapped_slice_mut().unwrap();
                let mut offset = 0;

                let buffer_copy_regions = data
                    .into_iter()
                    .enumerate()
                    .map(|(level, sub)| {
                        mapped_slice_mut[offset..offset + sub.data.len()].copy_from_slice(sub.data);

                        let region = vk::BufferImageCopy::builder()
                            .buffer_offset(offset as _)
                            .image_subresource(
                                vk::ImageSubresourceLayers::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1)
                                .mip_level(level as _)
                                .build(),
                            )
                            .image_extent(vk::Extent3D {
                                width: (desc.extent.width >> level).max(1),
                                height: (desc.extent.height >> level).max(1),
                                depth: (desc.extent.depth >> level).max(1),
                            });

                        offset += sub.data.len();
                        region.build()
                    })
                .collect::<Vec<_>>();

                let setup_cb = self.create_command_buffer();

                self.with_commandbuffer_wait_idle(&setup_cb, |cb| {
                    let barrier = vk::ImageMemoryBarrier{
                        dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                        new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        image,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: vk::REMAINING_MIP_LEVELS,
                            base_array_layer: 0,
                            layer_count: vk::REMAINING_ARRAY_LAYERS,
                        },
                        ..Default::default()
                    };

                    self.raw.cmd_pipeline_barrier(
                        cb,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[barrier],
                    );

                    self.raw.cmd_copy_buffer_to_image(
                        cb,
                        buffer.raw,
                        image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &buffer_copy_regions,
                    );

                    let barrier = vk::ImageMemoryBarrier{
                        src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                        dst_access_mask: vk::AccessFlags::SHADER_READ,
                        new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        image,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: vk::REMAINING_MIP_LEVELS,
                            base_array_layer: 0,
                            layer_count: vk::REMAINING_ARRAY_LAYERS,
                        },
                        ..Default::default()
                    };

                    self.raw.cmd_pipeline_barrier(
                        cb,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[barrier],
                    );
                });
            }

            Image {
                raw: image,
                desc: *desc,
                views: Mutex::new(FxHashMap::default()),
                device: self.clone(),
            }
        }
    }
}

impl Image {
    pub fn get_view_create_info(&self, desc: &ImageViewDesc) -> vk::ImageViewCreateInfo {
        vk::ImageViewCreateInfo {
            format: self.desc.format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            view_type: desc
                .view_type
                .unwrap_or_else(|| convert_image_type_to_view_type(self.desc.image_type)),
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: desc.aspect_mask,
                    base_mip_level: desc.base_mip_level,
                    level_count: desc.level_count.unwrap_or(self.desc.mip_levels as u32),
                    base_array_layer: 0,
                    layer_count: match self.desc.image_type {
                        ImageType::Cube | ImageType::CubeArray => 6,
                        _ => 1,
                    },
                },
                image: self.raw,
                ..Default::default()
        }
    }
    pub fn view(&self, desc: ImageViewDesc) -> ImageView {
        unsafe {
            let mut views = self.views.lock().unwrap();
            let fb_attachment_desc = FramebufferAttachmentDesc {
                flgas: self.desc.flags,
                usage: self.desc.usage,
                layer_count: desc.level_count.unwrap_or(self.desc.mip_levels as u32),
            };
            views
                .entry(desc)
                .or_insert_with(|| {
                    //println!("New View Created");
                    ImageView {
                        fb_attachment_desc,
                        raw: self
                            .device
                            .raw
                            .create_image_view(
                                &vk::ImageViewCreateInfo {
                                    format: self.desc.format,
                                    components: vk::ComponentMapping {
                                        r: vk::ComponentSwizzle::R,
                                        g: vk::ComponentSwizzle::G,
                                        b: vk::ComponentSwizzle::B,
                                        a: vk::ComponentSwizzle::A,
                                    },
                                    view_type: desc.view_type.unwrap_or_else(|| {
                                        convert_image_type_to_view_type(self.desc.image_type)
                                    }),
                                    subresource_range: vk::ImageSubresourceRange {
                                        aspect_mask: desc.aspect_mask,
                                        base_mip_level: desc.base_mip_level,
                                        level_count: desc
                                            .level_count
                                            .unwrap_or(self.desc.mip_levels as u32),
                                            base_array_layer: 0,
                                            layer_count: match self.desc.image_type {
                                                ImageType::Cube | ImageType::CubeArray => 6,
                                                _ => 1,
                                            },
                                    },
                                    image: self.raw,
                                    ..Default::default()
                                },
                                None,
                                )
                                    .unwrap(),
                                    desc,
                                    image_desc: self.desc,
                    }
                })
            .clone()
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            for (_, image_view) in self.views.lock().unwrap().iter() {
                println!("destroy_image_view");
                self.device.destroy_image_view(image_view.raw, None);
            }
            println!("destroy_image");
            self.device.destroy_image(self.raw, None);
        }
    }
}

pub fn get_image_create_info(desc: &ImageDesc, initial_data: bool) -> vk::ImageCreateInfo {
    let (image_type, image_extent, image_layers) = match desc.image_type {
        ImageType::Tex1d => (
            vk::ImageType::TYPE_1D,
            vk::Extent3D {
                width: desc.extent.width,
                height: 1,
                depth: 1,
            },
            1,
        ),
        ImageType::Tex1dArray => (
            vk::ImageType::TYPE_1D,
            vk::Extent3D {
                width: desc.extent.width,
                height: 1,
                depth: 1,
            },
            desc.array_elements,
        ),
        ImageType::Tex2d => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            1,
        ),
        ImageType::Tex2dArray => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            desc.array_elements,
        ),
        ImageType::Tex3d => (
            vk::ImageType::TYPE_3D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: desc.extent.depth as u32,
            },
            1,
        ),
        ImageType::Cube => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            6,
        ),
        ImageType::CubeArray => (
            vk::ImageType::TYPE_2D,
            vk::Extent3D {
                width: desc.extent.width,
                height: desc.extent.height,
                depth: 1,
            },
            6 * desc.array_elements,
        ),
    };

    let mut image_usage = desc.usage;

    if initial_data {
        image_usage |= vk::ImageUsageFlags::TRANSFER_DST;
    }

    vk::ImageCreateInfo {
        flags: desc.flags,
        image_type,
        format: desc.format,
        extent: image_extent,
        mip_levels: desc.mip_levels as u32,
        array_layers: image_layers as u32,
        samples: vk::SampleCountFlags::TYPE_1, // TODO: desc.sample_count
        tiling: desc.tiling,
        usage: image_usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        initial_layout: match initial_data {
            true => vk::ImageLayout::PREINITIALIZED,
            false => vk::ImageLayout::UNDEFINED,
        },
        ..Default::default()
    }
}
