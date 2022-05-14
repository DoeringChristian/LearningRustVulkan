use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::vulkan::{AllocationCreateDesc, Allocator};
use gpu_allocator::MemoryLocation;
use std::borrow::BorrowMut;
use std::sync::{Arc, Weak};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct BufferDesc<'a> {
    pub label: Option<&'a str>,
    pub size: usize,
    pub usage: vk::BufferUsageFlags,
    pub memory_location: MemoryLocation,
}

impl<'a> From<BufferDesc<'a>> for BufferDescInt {
    fn from(src: BufferDesc<'a>) -> Self {
        BufferDescInt {
            label: src.label.map(|s| String::from(s)),
            size: src.size,
            usage: src.usage,
            memory_location: src.memory_location,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct BufferDescInt {
    pub label: Option<String>,
    pub size: usize,
    pub usage: vk::BufferUsageFlags,
    pub memory_location: MemoryLocation,
}

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub desc: BufferDescInt,
    pub allocation: gpu_allocator::vulkan::Allocation,
    pub device: Arc<Device>,
}

pub trait CreateBuffer {
    fn create_buffer_alloc(&self, allocator: &mut Allocator, desc: BufferDesc) -> Buffer;
    fn create_buffer(&self, desc: BufferDesc, data: Option<&[u8]>) -> Buffer;
}

impl CreateBuffer for Arc<Device> {
    fn create_buffer_alloc(&self, allocator: &mut Allocator, desc: BufferDesc) -> Buffer {
        let buffer_info = vk::BufferCreateInfo {
            size: desc.size as u64,
            usage: desc.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .expect("Buffer creation error.")
        };
        let mut requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

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
            self.device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("Could not bind buffer memory.");
        }

        Buffer {
            buffer,
            desc: (desc).into(),
            device: self.clone(),
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

            let setup_command_buffer = self.create_command_buffer();

            self.with_commandbuffer_wait_idle(&setup_command_buffer, |cb| unsafe {
                self.device.cmd_copy_buffer(
                    cb,
                    scratch_buffer.buffer,
                    buffer.buffer,
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
