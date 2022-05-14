
use super::*;

pub trait BeginFrame{
    fn begin_frame(&self) -> Arc<DeviceFrame>;
    fn finish_frame(&self, frame: Arc<DeviceFrame>);
    fn submit_frame(&self, submits: &[vk::SubmitInfo], frame: &DeviceFrame);
}

impl BeginFrame for RenderDevice{
    fn begin_frame(&self) -> Arc<DeviceFrame> {
        let mut frame0 = self.frames[0].lock().unwrap();
        unsafe{
            let frame0 = Arc::get_mut(&mut frame0).unwrap();

            self.raw.wait_for_fences(
                &[frame0.main_cb.submit_done_fence],
                true,
                std::u64::MAX,
            ).expect("Could not wait for fence");

            self.raw.reset_fences(
                &[frame0.main_cb.submit_done_fence],
            ).expect("Could not reset fence");
        }
        frame0.clone()
    }

    fn finish_frame(&self, frame: Arc<DeviceFrame>) {
        drop(frame);

        let mut frame0 = self.frames[0].lock().unwrap();

        let frame0 = Arc::get_mut(&mut frame0).expect("Could not acquire frame0");

        {
            let mut frame1 = self.frames[1].lock().unwrap();
            let frame1 = Arc::get_mut(&mut frame1).expect("Could not acquire frame1");

            std::mem::swap(frame0, frame1);
        }
    }

    fn submit_frame(&self, submits: &[vk::SubmitInfo], frame: &DeviceFrame) {
        unsafe{
            self.shared.raw.queue_submit(self.shared.global_queue, submits, frame.main_cb.submit_done_fence).unwrap();
        }
    }
}
