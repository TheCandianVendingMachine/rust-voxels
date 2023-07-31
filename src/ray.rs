use cgmath::Vector2;

pub struct Ray {
    pub origin: Vector2<f64>,
    pub direction: Vector2<f64>,
    pub max_distance: Option<f64>
}
