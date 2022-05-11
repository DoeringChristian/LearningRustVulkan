use std::os::raw::c_char;

use super::*;
use ash::extensions::khr;
use ash::{extensions::ext::DebugUtils, vk};
use raw_window_handle::HasRawWindowHandle;

impl Instance {
    pub fn init(compatible_window: Option<&dyn HasRawWindowHandle>) -> Self {
        unsafe {
            let entry = match ash::Entry::load() {
                Ok(entry) => entry,
                Err(err) => {
                    panic!("Could not load Vulkan: {:?}", err);
                }
            };

            let driver_api_version = match entry.try_enumerate_instance_version() {
                Ok(Some(version)) => version,
                Ok(None) => vk::API_VERSION_1_0,
                Err(e) => {
                    panic!("try_enumerate_instance_version: {:?}", e);
                }
            };

            let appname = CString::new("test app").unwrap();
            let appinfo = vk::ApplicationInfo::builder()
                .application_name(appname.as_c_str())
                .application_version(0)
                .engine_name(CStr::from_bytes_with_nul(b"hephaistos\0").unwrap())
                .api_version(if driver_api_version < vk::API_VERSION_1_1 {
                    vk::API_VERSION_1_0
                } else {
                    vk::HEADER_VERSION_COMPLETE
                });

            let mut extension_names = match compatible_window {
                Some(window) => ash_window::enumerate_required_extensions(window)
                    .unwrap()
                    .to_vec(),
                None => Vec::new(),
            };

            extension_names.push(DebugUtils::name().as_ptr());

            let layer_names = [CStr::from_bytes_with_nul_unchecked(
                b"VK_LAYER_KHRONOS_validation\0",
            )];
            let layers_names_raw: Vec<*const c_char> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&appinfo)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names);

            let instance = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(super::utils::vulkan_debug_callback));
            let debug_utils_loader = DebugUtils::new(&entry, &instance);
            let debug_call_back = debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap();

            Self { 
                instance, 
                entry,
                debug_utils_loader: Some(debug_utils_loader),
                debug_call_back: Some(debug_call_back),
            }
        }
    }

    pub fn create_surface(&self, window_handle: &dyn HasRawWindowHandle) -> Surface {
        unsafe {
            let surface =
                ash_window::create_surface(&self.entry, &self.instance, window_handle, None)
                    .unwrap();

            let surface_loader = khr::Surface::new(&self.entry, &self.instance);

            Surface {
                surface,
                surface_loader,
            }
        }
    }
}

impl Drop for Instance{
    fn drop(&mut self) {
        unsafe{
            self.debug_utils_loader.as_ref().and_then(|d| {
                d.destroy_debug_utils_messenger(self.debug_call_back.unwrap(), None);
                Some(())
            });
            self.instance.destroy_instance(None);
        }
    }
}
