
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

impl RenderPass{
    pub fn begin(&self, desc: &RenderPassBeginnDesc, draw_command_buffer: vk::CommandBuffer){

        //let framebuffer = self.framebuffer_cache.get_or_create(&self.device, key).unwrap();
        let color_attachment_descs = desc.color_attachments.iter().map(|a|{
            a.fb_attachment_desc
        }).collect::<Vec<_>>();
        let framebuffer_key = FramebufferCacheKey::new(
            desc.area.extent,
            color_attachment_descs.iter(),
            desc.depth_attachment.map(|a|{
                &a.fb_attachment_desc
            })
        );

        let image_attachments = desc.color_attachments.iter()
            .chain(desc.depth_attachment.as_ref().into_iter())
            .map(|v|{
                v.view
            }).collect::<ArrayVec<vk::ImageView, MAX_COLOR_ATTACHMENTS>>();

        let mut pass_attachment_desc = vk::RenderPassAttachmentBeginInfoKHR::builder()
            .attachments(&image_attachments);

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.rpass)
            .framebuffer(self.framebuffer_cache.get_or_create(&self.device, framebuffer_key).unwrap())
            .render_area(desc.area)
            .clear_values(desc.clear_values)
            .push_next(&mut pass_attachment_desc);

        unsafe{
            self.device.cmd_begin_render_pass(
                draw_command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }
    pub fn end(&self, draw_command_buffer: vk::CommandBuffer){
        unsafe{
            self.device.cmd_end_render_pass(draw_command_buffer);
        }
    }
}

impl Drop for RenderPass{
    fn drop(&mut self) {
        self.framebuffer_cache.destroy_cache(&self.device);
        unsafe{
            self.device.destroy_render_pass(self.rpass, None)
        };
    }
}
