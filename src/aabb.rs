use cgmath::Vector2;

pub struct AABB {
    pub position: Vector2<f64>,
    pub size: Vector2<f64>
}

impl AABB {
    pub fn new() -> AABB {
        AABB {
            position: Vector2 { x: 0.0, y: 0.0 },
            size: Vector2 { x: 0.0, y: 0.0 }
        }
    }

    pub fn from_position_and_size(position: Vector2<f64>, size: Vector2<f64>) -> AABB {
        AABB {
            position,
            size,
        }
    }

    pub fn min(&self) -> Vector2<f64> {
        self.position
    }

    pub fn max(&self) -> Vector2<f64> {
        self.position + self.size
    }

    pub fn mid(&self) -> Vector2<f64> {
        self.position + self.size * 0.5
    }
}

