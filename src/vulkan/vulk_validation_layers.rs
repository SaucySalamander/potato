use ash::extensions::ext::DebugUtils;
use ash::vk;
use ash::vk::{
    Bool32, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
    DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerEXT,
};
use ash::{Entry, Instance};
use log::debug;
use std::ffi::CStr;
use std::os::raw::c_void;
use super::constants::VALIDATION;

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> Bool32 {
    let severity = match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    debug!("{}, {}, {:?}", severity, types, message);

    ash::vk::FALSE
}

pub fn setup_debug_utils(
    entry: &Entry,
    instance: &Instance,
) -> (DebugUtils, DebugUtilsMessengerEXT) {
    let debug_util_loader = DebugUtils::new(entry, instance);

    if !VALIDATION.is_enable {
        (debug_util_loader, DebugUtilsMessengerEXT::null())
    } else {
        let messenger_ci = populate_debug_messenger_create_info();

        let utils_messenger = unsafe {
            debug_util_loader
                .create_debug_utils_messenger(&messenger_ci, None)
                .expect("Debug Utils Callback")
        };
        (debug_util_loader, utils_messenger)
    }
}

pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: std::ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            // Can uncomment for more verbosity form the validation layers
            // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: std::ptr::null_mut(),
    }
}
