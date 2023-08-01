mod aabb;
mod collision;
mod colliders;
mod grid;
mod voxel;
mod ray;
mod window;

fn main() {
    env_logger::init();
    pollster::block_on(window::Window::new()).run();
}
