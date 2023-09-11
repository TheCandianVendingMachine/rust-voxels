use winit::{
    event_loop::EventLoop,
    window::{ self, WindowBuilder }
};

pub struct Window {
    size: winit::dpi::PhysicalSize<u32>,
    event_loop: Option<EventLoop<()>>,
    pub window: window::Window,
    pub surface: wgpu::Surface
}

impl Window {
    pub fn new(instance: &wgpu::Instance) -> Window {
        let event_loop = Some(EventLoop::new());
        let window = WindowBuilder::new().build(&event_loop.as_ref().unwrap()).unwrap();
        let size = window.inner_size();

        /* # Safety
         *
         * The surface only needs to live as long as the window, and the window lasts as 
         * long as the surface so this will remain valid
         */
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        Window {
            size,
            event_loop,
            window,
            surface
        }
    }
}
