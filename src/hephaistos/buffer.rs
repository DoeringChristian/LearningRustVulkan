
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocator, AllocationCreateDesc};
use std::borrow::BorrowMut;
use std::sync::{Arc, Weak};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct BufferDesc<'a>{
    pub label: Option<&'a str>,
    pub size: usize,
    pub usage: vk::BufferUsageFlags,
    pub memory_location: MemoryLocation,
}

impl<'a> From<BufferDesc<'a>> for BufferDescInt{
    fn from(src: BufferDesc<'a>) -> Self {
        BufferDescInt{
            label: src.label.map(|s| String::from(s)),
            size: src.size,
            usage: src.usage,
            memory_location: src.memory_location,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct BufferDescInt{
    pub label: Option<String>,
    pub size: usize,
    pub usage: vk::BufferUsageFlags,
    pub memory_location: MemoryLocation,
}

pub struct Buffer{
    pub buffer: vk::Buffer,
    pub desc: BufferDescInt,
    pub device: Arc<Device>,
}

pub trait CreateBuffer{
    fn create_buffer_alloc(&self, allocator: &mut Allocator, desc: &BufferDesc) -> Buffer;
    fn create_buffer(&self, desc: &BufferDesc) -> Buffer;
}

impl CreateBuffer for Arc<Device>{
    fn create_buffer_alloc(&self, allocator: &mut Allocator, desc: &BufferDesc) -> Buffer {
        let buffer_info = vk::BufferCreateInfo{
            size: desc.size as u64,
            usage: desc.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe{
            self.device.create_buffer(&buffer_info, None).expect("Buffer creation error.")
        };
        let mut requirements = unsafe{self.device.get_buffer_memory_requirements(buffer)};

        if desc.usage.contains(vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR){
            requirements.alignment = requirements.alignment.max(64);
        }

        let allocation = allocator.allocate(&AllocationCreateDesc{
            name: desc.label.unwrap_or(""),
            requirements,
            location: desc.memory_location,
            linear: true,
        }).expect("Could not allocate memory on GPU");

        unsafe{
            self.device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()).expect("Could not bind buffer memory.");
        }

        Buffer{
            buffer,
            desc: (*desc).into(),
            device: self.clone(),
        }
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Buffer {
        let mut allocator = self.global_allocator.lock().unwrap();
        self.create_buffer_alloc(&mut allocator, desc)
    }
}


