pub mod api;
mod texture;
mod window;

use crate::render::Queue;
use crate::resource::{ ResourceManager, ResourceMetaData, ResourceLifetime };
use window::Window;
use wgpu::{
    Device, Adapter
};

pub struct DeviceState {
    device: Device,
    adapter: Adapter,
    queues: Box<[Queue]>
}

impl DeviceState {
    async fn new(instance: &wgpu::Instance, surface: &wgpu::Surface) -> DeviceState {
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None
        ).await.unwrap();

        DeviceState {
            device,
            adapter,
            queues: Box::new([Queue::Render(queue)])
        }
    }
}

pub struct RenderEngine<'engine> {
    instance: wgpu::Instance,
    textures: ResourceManager<'engine, texture::Texture>,
    window: Window
}

impl RenderEngine<'_> {
    pub fn new<'engine>(device: &DeviceState, texture_handler: &'engine mut texture::TextureHandler) -> RenderEngine<'engine> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default()
        });

        let window = Window::new(&instance);
        let surface_caps = window.surface.get_capabilities(&device.adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };
        window.surface.configure(&device.device, &config);

        let surface_uuid = texture_handler.set_surface(&window.surface);
        let mut textures = ResourceManager::new::<1024>(texture_handler);
        textures.create(&ResourceMetaData {
            uuid: surface_uuid,
            lifetime: ResourceLifetime::Forever,
            name: Some(std::borrow::Cow::Owned("Window Surface".to_string())),
            path: None
        });

        RenderEngine {
            instance,
            textures,
            window
        }
    }
}
