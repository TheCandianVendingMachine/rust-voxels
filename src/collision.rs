use cgmath::Vector2;
use crate::ray::Ray;
use crate::aabb::AABB;

pub struct IntersectInfo {
    pub position: Vector2<f64>,
}

pub trait Collidable<T> {
    type IntersectReturn;
    type CollisionReturn;

    fn does_intersect(&self, other: &T) -> Self::IntersectReturn;
    fn does_contain(&self, other: &T) -> bool;
    fn does_collide(&self, other: &T) -> Self::CollisionReturn;
}

impl Collidable<Ray> for AABB {
    type IntersectReturn = Option<IntersectInfo>;
    type CollisionReturn = Self::IntersectReturn;

    fn does_intersect(&self, ray: &Ray) -> Self::IntersectReturn {
        const EPSILON: f64 = 0.00001;
        if ray.direction.x.abs() <= EPSILON {
            if ray.origin.x < self.min().x || ray.origin.x > self.max().x {
                return None
            }
        }
        if ray.direction.y.abs() <= EPSILON {
            if ray.origin.y < self.min().y || ray.origin.y > self.max().y {
                return None
            }
        }

        let mut tmin = 0.0_f64;
        let mut tmax = ray.max_distance.unwrap_or(f64::MAX);

        let check_slab = &mut |p: f64, d: f64, min: f64, max: f64| -> bool {
            let ood = 1.0 / d;
            let (t1, t2) = {
                let t1 = (min - p) * ood;
                let t2 = (max - p) * ood;

                if t1 > t2 {
                    (t2, t1)
                } else {
                    (t1, t2)
                }
            };

            tmin = tmin.max(t1);
            tmax = tmax.max(t2);

            tmin <= tmax
        };

        if !check_slab(ray.origin.x, ray.direction.x, self.min().x, self.max().x) {
            return None
        }

        if !check_slab(ray.origin.y, ray.direction.y, self.min().y, self.max().y) {
            return None
        }

        Some(IntersectInfo { position: ray.origin + ray.direction * tmin })
    }

    fn does_contain(&self, ray: &Ray) -> bool {
        if ray.max_distance.is_none() {
            return false
        }
        let relative_pos = ray.origin - self.position;
        let end = ray.direction * ray.max_distance.unwrap();
        relative_pos.x >= 0.0 && relative_pos.x + end.x <= self.size.x &&
        relative_pos.y >= 0.0 && relative_pos.y + end.y <= self.size.y
    }

    fn does_collide(&self, ray: &Ray) -> Self::CollisionReturn {
        self.does_intersect(ray)
    }
}

impl Collidable<Vector2<f64>> for AABB {
    type IntersectReturn = ();
    type CollisionReturn = bool;

    fn does_intersect(&self, _point: &Vector2<f64>) -> Self::IntersectReturn {
        panic!("Cannot test an intersection against a point and AABB")
    }

    fn does_contain(&self, point: &Vector2<f64>) -> bool {
        point.x >= self.position.x && point.x < self.position.x + self.size.x &&
        point.y >= self.position.y && point.y < self.position.y + self.size.y
    }

    fn does_collide(&self, point: &Vector2<f64>) -> Self::CollisionReturn {
        self.does_contain(point)
    }
}

impl Collidable<AABB> for AABB {
    type IntersectReturn = bool;
    type CollisionReturn = bool;

    fn does_intersect(&self, other: &AABB) -> Self::IntersectReturn {
        let relative_pos = other.position - self.position;
        /*
                +-------+
                |       |
           +----|---+   |
           |    +-------+
           |        |
           +--------+
        */
        ( // test x position
            (relative_pos.x >= 0.0 && relative_pos.x < self.size.x) || 
            (relative_pos.x + other.size.x >= 0.0 && relative_pos.x + other.size.x < self.size.x)
        ) &&
        ( // test y position
            (relative_pos.y >= 0.0 && relative_pos.y < self.size.y) || 
            (relative_pos.y + other.size.y >= 0.0 && relative_pos.y + other.size.y < self.size.y)
        )
    }

    fn does_contain(&self, other: &AABB) -> bool {
        /*
           +--------+
           | +---+  |
           | +---+  |
           +--------+
        */
        self.does_intersect(other) && !other.does_intersect(self)
    }

    fn does_collide(&self, other: &AABB) -> Self::CollisionReturn {
        self.does_intersect(other) || other.does_intersect(self)
    }
}
