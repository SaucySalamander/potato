use super::device::create_logical_device;
use super::physical_device::select_physical_device;
use super::utilities::{conver_str_vec_to_c_str_ptr_vec, vk_to_string};
use super::vulk_validation_layers::{
    populate_debug_messenger_create_info, setup_debug_utils, VALIDATION,
};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, XlibSurface};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::{
    make_version, ApplicationInfo, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT,
    InstanceCreateFlags, InstanceCreateInfo, PhysicalDevice, Queue, StructureType,
};
use ash::Device;
use ash::Entry;
use ash::Instance;
use log::info;
use std::ffi::CString;
use std::os::raw::c_void;

pub struct VulkanApiObjects {
    _entry: Entry,
    instance: Instance,
    debug_utils_loader: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,
    _physical_device: PhysicalDevice,
    device: Device,
    _graphics_queue: Queue,
}

impl VulkanApiObjects {
    pub fn init() -> VulkanApiObjects {
        info!("Initializing VulkanApiObjects");
        let entry = Entry::new().unwrap();
        let instance = VulkanApiObjects::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) = setup_debug_utils(&entry, &instance);
        let physical_device = select_physical_device(&instance);
        let (logical_device, graphics_queue) = create_logical_device(&instance, physical_device);

        VulkanApiObjects {
            _entry: entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
            device: logical_device,
            _graphics_queue: graphics_queue,
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

        let debug_utils_create_info = populate_debug_messenger_create_info();

        let extension_names = vec![
            Surface::name().as_ptr(),
            XlibSurface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ];

        let (cstring_vec, enable_layer_names) =
            conver_str_vec_to_c_str_ptr_vec(VALIDATION.required_validation_layers.to_vec());

        let create_info = InstanceCreateInfo {
            s_type: StructureType::INSTANCE_CREATE_INFO,
            p_next: if VALIDATION.is_enable {
                &debug_utils_create_info as *const DebugUtilsMessengerCreateInfoEXT as *const c_void
            } else {
                std::ptr::null()
            },
            flags: InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if VALIDATION.is_enable {
                enable_layer_names.as_ptr()
            } else {
                std::ptr::null()
            },
            enabled_layer_count: get_enabled_layers_len(),
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
            self.device.destroy_device(None);
            if VALIDATION.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

fn check_validation_layer_support(entry: &Entry) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate the Instance Layers Properties!");

    // info!("{:?}", layer_properties);
    VALIDATION
        .required_validation_layers
        .iter()
        .map(|layers| {
            layer_properties
                .iter()
                .any(|v| vk_to_string(&v.layer_name) == *layers)
        })
        .any(|b| b)
}

fn get_enabled_layers_len() -> u32 {
    if VALIDATION.is_enable {
        VALIDATION.required_validation_layers.iter().len() as u32
    } else {
        0 as u32
    }
}
