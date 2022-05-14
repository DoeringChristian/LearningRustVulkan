use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::vulkan::{AllocationCreateDesc, Allocator};
use gpu_allocator::MemoryLocation;
use std::borrow::BorrowMut;
use std::sync::{Arc, Weak};

pub trait CreateBuffer {
    fn create_buffer_alloc(&self, allocator: &mut Allocator, desc: BufferDesc) -> Buffer;
    fn create_buffer(&self, desc: BufferDesc, data: Option<&[u8]>) -> Buffer;
}

impl CreateBuffer for RenderDevice{
    fn create_buffer_alloc(&self, allocator: &mut Allocator, desc: BufferDesc) -> Buffer {
        let buffer_info = vk::BufferCreateInfo {
            size: desc.size as u64,
            usage: desc.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            self.raw
                .create_buffer(&buffer_info, None)
                .expect("Buffer creation error.")
        };
        let mut requirements = unsafe { self.raw.get_buffer_memory_requirements(buffer) };

        if desc
            .usage
            .contains(vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR)
        {
            requirements.alignment = requirements.alignment.max(64);
        }

        let allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: desc.label.unwrap_or(""),
                requirements,
                location: desc.memory_location,
                linear: true,
            })
            .expect("Could not allocate memory on GPU");

        unsafe {
            self.raw
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("Could not bind buffer memory.");
        }

        Buffer {
            raw: buffer,
            desc: (desc).into(),
            device: self.shared.clone(),
            allocation,
        }
    }

    fn create_buffer(&self, mut desc: BufferDesc, data: Option<&[u8]>) -> Buffer {
        if data.is_some() {
            desc.usage |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        let mut allocator = self.global_allocator.lock().unwrap();
        let buffer = self.create_buffer_alloc(&mut allocator, desc);

        if let Some(data) = data {
            let mut scratch_buffer = self.create_buffer_alloc(
                &mut self.global_allocator.lock().unwrap(),
                BufferDesc {
                    size: desc.size,
                    usage: vk::BufferUsageFlags::TRANSFER_SRC,
                    memory_location: gpu_allocator::MemoryLocation::CpuToGpu,
                    label: None,
                },
            );

            scratch_buffer.allocation.mapped_slice_mut().unwrap()[0..data.len()]
                .copy_from_slice(data);

            self.with_setup_cb(|cb| unsafe {
                self.raw.cmd_copy_buffer(
                    cb,
                    scratch_buffer.raw,
                    buffer.raw,
                    &[vk::BufferCopy::builder()
                        .dst_offset(0)
                        .src_offset(0)
                        .size(desc.size as u64)
                        .build()],
                );
            });
        }

        buffer
    }
}

impl Drop for Buffer{
    fn drop(&mut self) {
        unsafe{
            self.device.destroy_buffer(self.raw, None);
        }
    }
}
