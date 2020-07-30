use super::vulk_validation_layers::{setup_debug_utils, VALIDATION};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::extensions::khr::XlibSurface;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk::{
    make_version, ApplicationInfo, DebugUtilsMessengerEXT, InstanceCreateFlags, InstanceCreateInfo,
    StructureType,
};
use ash::Entry;
use ash::Instance;
use log::info;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

pub struct VulkanApiObjects {
    _entry: Entry,
    instance: Instance,
    debug_utils_loader: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,
}

impl VulkanApiObjects {
    pub fn init() -> VulkanApiObjects {
        info!("Initializing VulkanApiObjects");
        let entry = Entry::new().unwrap();
        let instance = VulkanApiObjects::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) = setup_debug_utils(&entry, &instance);

        VulkanApiObjects {
            _entry: entry,
            instance,
            debug_utils_loader,
            debug_messenger,
        }
    }

    fn create_instance(entry: &Entry) -> Instance {
        if VALIDATION.is_enable && !check_validation_layer_support(entry) {
            panic!("Validation layers requested but not supported.");
        }

        let app_name = CString::new("Test").unwrap();
        let engine_name = CString::new("Potato").unwrap();
        let app_info = ApplicationInfo {
            s_type: StructureType::APPLICATION_INFO,
            p_next: std::ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: make_version(0, 0, 1),
            p_engine_name: engine_name.as_ptr(),
            engine_version: make_version(0, 0, 1),
            api_version: make_version(1, 2, 148),
        };

        let extension_names = vec![
            Surface::name().as_ptr(),
            XlibSurface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ];

        let create_info = InstanceCreateInfo {
            s_type: StructureType::INSTANCE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: std::ptr::null(),
            enabled_layer_count: 0,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
        };

        info!("Creating Instance with {:?}", create_info);
        let instance: Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };
        instance
    }


}

impl Drop for VulkanApiObjects {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

fn check_validation_layer_support(entry: &Entry) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate the Instance Layers Properties!");

    VALIDATION.required_validation_layers.iter()
        .map(|layers| layer_properties.iter()
                                      .any(|v| vk_to_string(&v.layer_name) == *layers)).any(|b| b)
}

fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}