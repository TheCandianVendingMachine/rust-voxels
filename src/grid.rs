use crate::voxel::Voxel;
use crate::colliders::*;
use cgmath::{ Vector2, InnerSpace };
use std::hash::{ Hash, Hasher };

const VOXEL_COUNT_X: usize = 10;
const VOXEL_COUNT_Y: usize = 10;
const VOXEL_COUNT: usize = VOXEL_COUNT_X * VOXEL_COUNT_Y;

pub struct Grid {
    elements: [Option<Voxel>; VOXEL_COUNT],
    hash: u128
}

impl Grid {
    pub fn new() -> Grid {
        let elements = [None; VOXEL_COUNT];
        Grid {
            hash: elements.iter().enumerate().map(|(i, v)| {
                let (x, y) = Grid::get_coords_from_index(i);
                Grid::hash_for_voxel(x, y, v.unwrap_or(Voxel::default()).element_id)
            }).sum(),
            elements,
        }
    }

    const fn get_index_from_coords(x: u64, y: u64) -> usize {
        (x + y * VOXEL_COUNT_X as u64) as usize
    }

    const fn get_coords_from_index(index: usize) -> (u64, u64) {
        ((index % VOXEL_COUNT_X) as u64, (index / VOXEL_COUNT_X) as u64)
    }

    const fn hash_for_voxel(x: u64, y: u64, element_id: u16) -> u128 {
        const P1: u128 = 963726515729;
        const P2: u128 = 318083817907;
        const P3: u128 = 222334565193649;

        (x as u128 * P1) ^ (y as u128 * P2) ^ (element_id as u128 * P3)
    }

    pub fn set(&mut self, x: u64, y: u64, voxel: Voxel) {
        let previous_element = self.elements[Grid::get_index_from_coords(x, y)].unwrap_or(Default::default());
        let previous_hash = Grid::hash_for_voxel(x, y, previous_element.element_id);
        let new_hash = Grid::hash_for_voxel(x, y, voxel.element_id);

        self.elements[Grid::get_index_from_coords(x, y)] = Some(voxel);
        self.hash = self.hash - previous_hash + new_hash
    }

    pub fn get_all_orientation_hashes(&self) -> [u128; 4] {
        let mut hashes = [0; 4];

        hashes[0] = self.hash;
        hashes[1] = self.elements.iter()
            .enumerate()
            .map(|(i, v)| { (Grid::get_coords_from_index(i), v.unwrap_or(Voxel::default()).element_id) })
            .map(|((x, y), e)| {
                (VOXEL_COUNT_X as u64 - x, y, e)
            })
            .map(|(x, y, e)| Grid::hash_for_voxel(x, y, e))
            .sum();

        hashes[2] = self.elements.iter()
            .enumerate()
            .map(|(i, v)| { (Grid::get_coords_from_index(i), v.unwrap_or(Voxel::default()).element_id) })
            .map(|((x, y), e)| {
                (x, VOXEL_COUNT_Y as u64 - y, e)
            })
            .map(|(x, y, e)| Grid::hash_for_voxel(x, y, e))
            .sum();

        hashes[3] = self.elements.iter()
            .enumerate()
            .map(|(i, v)| { (Grid::get_coords_from_index(i), v.unwrap_or(Voxel::default()).element_id) })
            .map(|((x, y), e)| {
                (VOXEL_COUNT_X as u64 - x, VOXEL_COUNT_Y as u64 - y, e)
            })
            .map(|(x, y, e)| Grid::hash_for_voxel(x, y, e))
            .sum();

        hashes
    }

    pub fn is_orientation_of(&self, other: &Grid) -> bool {
        other.get_all_orientation_hashes().iter().any(|h| *h == self.hash)
    }
}

pub struct SpatialGrid {
    pub grid: Grid,
    /// Origin of grid: based in top left corner
    pub origin: Vector2<f64>,
    pub voxel_side_length: f64,
}

pub enum IntersectType {
    First,
    All
}

impl SpatialGrid {
    pub fn new(voxel_side_length: f64) -> SpatialGrid {
        SpatialGrid {
            grid: Grid::new(),
            origin: Vector2::new(0.0, 0.0),
            voxel_side_length
        }
    }

    pub fn bounds(&self) -> AABB {
        AABB::from_position_and_size(self.origin, Vector2 {
            x: VOXEL_COUNT_X as f64 * self.voxel_side_length,
            y: VOXEL_COUNT_Y as f64 * self.voxel_side_length
        })
    }

    pub fn walk_grid_across_ray(&self, ray: Ray, on_voxel_hit: &mut dyn FnMut(Voxel) -> bool) {
        let ray = Ray {
            origin: {
                let grid_aabb = self.bounds();
                let intersect_pos = if grid_aabb.does_contain(&ray.origin) {
                    ray.origin
                } else if let Some(intersect) = grid_aabb.does_intersect(&ray) {
                    intersect.position + ray.direction * 0.001
                } else {
                    return
                };

                intersect_pos - self.origin
            },
            direction: ray.direction,
            max_distance: ray.max_distance
        };

        let step = Vector2 {
            x: (ray.direction.x >= 0.0) as i64 * 2 - 1,
            y: (ray.direction.y >= 0.0) as i64 * 2 - 1
        };

        let t_delta = self.voxel_side_length * {
            let magnitude = ray.direction.magnitude();
            Vector2 { 
                x: magnitude / ray.direction.x,
                y: magnitude / ray.direction.y
            }
        };
        let mut t_max = {
            let min = self.voxel_side_length * Vector2 {
                x: (ray.origin.x / self.voxel_side_length).floor(),
                y: (ray.origin.y / self.voxel_side_length).floor()
            };
            let max = min + Vector2::new(self.voxel_side_length, self.voxel_side_length);

            let scalar = {
                let x = if ray.direction.x >= 0.0 {
                    max.x - ray.origin.x
                } else {
                    ray.origin.x - min.x
                };

                let y = if ray.direction.y >= 0.0 {
                    max.y - ray.origin.y
                } else {
                    ray.origin.y - min.y
                };

                Vector2{ x, y }
            };

            Vector2 {
                x: scalar.x * t_delta.x,
                y: scalar.y * t_delta.y
            }
        };

        let mut grid_pos = Vector2 {
            x: ray.origin.x.floor() as i64,
            y: ray.origin.y.floor() as i64,
        };

        loop {
            let voxel = self.grid.elements[Grid::get_index_from_coords(grid_pos.x as u64, grid_pos.y as u64)];
            if let Some(v) = voxel {
                on_voxel_hit(v);
            }

            if t_max.x < t_max.y {
                t_max.x += t_delta.x;
                grid_pos.x += step.x;
                if grid_pos.x < 0 || grid_pos.x as usize >= VOXEL_COUNT_X {
                    break;
                }
            } else {
                t_max.y += t_delta.y;
                grid_pos.y += step.y;
                if grid_pos.y < 0 || grid_pos.y as usize >= VOXEL_COUNT_Y {
                    break;
                }
            }
        }
    }

    pub fn get_intersections(&self, ray: Ray, intersect: IntersectType) -> Vec<Voxel> {
        let mut voxels_hit = Vec::new();
        if let IntersectType::First = intersect {
            self.walk_grid_across_ray(ray, &mut |v| {
                voxels_hit.push(v);
                false
            });
        } else {
            self.walk_grid_across_ray(ray, &mut |v| {
                voxels_hit.push(v);
                true
            });
        }
        voxels_hit
    }
}

impl PartialEq for Grid {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Grid {}

impl Hash for Grid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}
