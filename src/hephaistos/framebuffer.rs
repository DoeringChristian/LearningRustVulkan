// Copied from https://github.com/EmbarkStudios/kajiya/blob/main/crates/lib/kajiya-backend/src/vulkan/shader.rs
/*
Copyright (c) 2019 Embark Studios

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
 */

use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::borrow::BorrowMut;
use std::sync::Arc;

impl FramebufferCacheKey {
    pub fn new<'a>(
        extent: vk::Extent2D,
        color_attachments: impl Iterator<Item = &'a FramebufferAttachmentDesc>,
        depth_stencil_attachment: Option<&'a FramebufferAttachmentDesc>,
    ) -> Self {
        let color_attachments = color_attachments
            .chain(depth_stencil_attachment.into_iter())
            .copied()
            .map(|attachment| attachment)
            .collect();

        Self {
            extent,
            attachments: color_attachments,
        }
    }
}
/*
impl<'a> From<RenderPassBeginnDesc<'a>> for FramebufferCacheKey{
    fn from(src: RenderPassBeginnDesc) -> Self {
        let attachments = src.color_attachments.iter()
            .chain(src.depth_attachment.iter())
            .map(|a|{
                (a.image_desc.usage, a.image_desc.flags)
            }).collect::<ArrayVec<_, MAX_COLOR_ATTACHMENTS>>();
        FramebufferCacheKey{
            extent: src.area.extent,
            attachments,
        }
    }
}
*/

impl FramebufferCache {
    pub fn new(
        render_pass: vk::RenderPass,
        color_attachments: &[vk::AttachmentDescription],
        depth_attachment: Option<vk::AttachmentDescription>,
    ) -> Self {
        let mut attachment_desc = ArrayVec::new();

        attachment_desc
            .try_extend_from_slice(color_attachments)
            .unwrap();

        if let Some(depth_attachment) = depth_attachment {
            attachment_desc.push(depth_attachment)
        }

        Self {
            entries: Default::default(),
            attachment_desc,
            render_pass,
            color_attachment_count: color_attachments.len(),
        }
    }
    pub fn get_or_create(
        &self,
        device: &ash::Device,
        key: FramebufferCacheKey,
    ) -> Option<vk::Framebuffer> {
        let mut entries = self.entries.lock().unwrap();

        if let Some(entry) = entries.get(&key) {
            Some(*entry)
        } else {
            let entry = {
                let attachments: ArrayVec<_, MAX_COLOR_ATTACHMENTS> = self
                    .attachment_desc
                    .iter()
                    .zip(key.attachments.iter())
                    .map(|(desc, attachemnt_desc)| {
                        vk::FramebufferAttachmentImageInfoKHR::builder()
                            .width(key.extent.width)
                            .height(key.extent.height)
                            .flags(attachemnt_desc.flgas)
                            .view_formats(&[desc.format])
                            .usage(attachemnt_desc.usage)
                            .layer_count(attachemnt_desc.layer_count)
                            .build()
                    })
                    .collect();

                let mut imageless_desc = vk::FramebufferAttachmentsCreateInfo::builder()
                    .attachment_image_infos(&attachments);
                let mut create_info = vk::FramebufferCreateInfo::builder()
                    .flags(vk::FramebufferCreateFlags::IMAGELESS_KHR)
                    .render_pass(self.render_pass)
                    .width(key.extent.width)
                    .height(key.extent.height)
                    .layers(1)
                    .push_next(&mut imageless_desc);

                create_info.attachment_count = attachments.len() as u32;

                unsafe { device.create_framebuffer(&create_info, None).unwrap() }
            };

            entries.insert(key, entry);
            Some(entry)
        }
    }
    pub fn destroy_cache(&self, device: &ash::Device){
        for (_, item) in self.entries.lock().unwrap().iter(){
            unsafe{device.destroy_framebuffer(*item, None)};
        }
        self.entries.lock().unwrap().clear();
    }
}

