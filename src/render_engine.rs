mod resource;
pub mod api;

use crate::render::Queue;
use wgpu::{
    Device
};

struct DeviceState {
    device: Device,
    queues: [Queue]
}

pub struct RenderEngine {

}
