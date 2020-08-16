use super::vertex::Vertex;

pub struct ValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub struct DeviceExtension {
    pub names: [&'static str; 1]
}

pub const DEVICE_EXTENSTIONS: DeviceExtension = DeviceExtension {
    names: ["VK_KHR_swapchain"],
};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub const VERTICES_DATA: [Vertex; 3] = [
    Vertex {
        pos: [0.0, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5,0.5],
        color: [0.0,1.0,0.0],
    },
    Vertex {
        pos: [-0.5,0.5],
        color: [0.0,0.0,1.0],
    },
];