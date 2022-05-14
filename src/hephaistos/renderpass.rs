
use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub trait CreateRenderPass{
    fn create_render_pass(&self, desc: &RenderPassDesc<'_>) -> Arc<RenderPass>;
}

impl CreateRenderPass for Arc<Device>{
    fn create_render_pass(&self, desc: &RenderPassDesc<'_>) -> Arc<RenderPass> {
        let renderpass_attachments = desc
            .color_attachments
            .iter()
            .map(|a| {
                *a
            })
            .chain(desc.depth_attachment.as_ref().map(|a| {
                *a
            }))
            .collect::<Vec<_>>();

        let color_attachment_refs = (0..desc.color_attachments.len() as u32)
            .map(|attachment| vk::AttachmentReference {
                attachment,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                //layout: desc.color_attachments[attachment as usize].initial_layout,
            })
            .collect::<Vec<_>>();

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: desc.color_attachments.len() as u32,
            layout: vk::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
            //layout: desc.depth_attachment.unwrap().initial_layout,
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
            self.raw
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        Arc::new(RenderPass {
            raw: render_pass,
            framebuffer_cache: FramebufferCache::new(
                render_pass,
                desc.color_attachments,
                desc.depth_attachment,
            ),
            device: self.clone(),
        })
    }
}

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
                v.raw
            }).collect::<ArrayVec<vk::ImageView, MAX_COLOR_ATTACHMENTS>>();

        let mut pass_attachment_desc = vk::RenderPassAttachmentBeginInfoKHR::builder()
            .attachments(&image_attachments);

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.raw)
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
            self.device.destroy_render_pass(self.raw, None)
        };
    }
}
