mod aabb;
mod collision;
mod colliders;
mod grid;
mod voxel;
mod ray;
mod window;
mod render_graph;
mod render;

fn main() {
    env_logger::init();
    pollster::block_on(window::Window::new()).run();
}
