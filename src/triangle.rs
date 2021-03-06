use cgmath::*;
use line;
use color;
use utils;


/// Returns barycentric coordinates of point `point` in triangle `tri`.
/// Triangle vertices positions are taken as Vector3 even though the function operates only in
/// 2 dimensions for compatibility with rendering loops.
pub fn barycentric(point: Vector2<f32>, tri: &[Vector3<f32>]) -> Option<Vector3<f32>> {
    let u: Vector3<f32> =
        Vector3::new(tri[2].x - tri[0].x, tri[1].x - tri[0].x, tri[0].x - point.x)
            .cross(Vector3::new(
                tri[2].y - tri[0].y,
                tri[1].y - tri[0].y,
                tri[0].y - point.y,
            ));
    if u.z.abs() < 1.0 {
        None
    } else {
        let result = Vector3::<f32>::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z);
        if result.x < 0.0 || result.y < 0.0 || result.z < 0.0 {
            None
        } else {
            Some(result)
        }
    }
}


/// Returns bounding box as tuple `(min_x, min_y, max_x, max_y)`
/// # Panics
/// * Not being able to find max or min value.
/// * Or anything else really, full of unwrap.
fn bounding_box(positions: &[Vector2<u32>]) -> (u32, u32, u32, u32) {
    let min_x = positions.iter().map(|pos| pos.x).min().unwrap();
    let min_y = positions.iter().map(|pos| pos.y).min().unwrap();
    let max_x = positions.iter().map(|pos| pos.x).max().unwrap();
    let max_y = positions.iter().map(|pos| pos.y).max().unwrap();
    (min_x, min_y, max_x, max_y)
}


fn naive_point_in_triangle(point: (usize, usize), triangle: &[Vector2<u32>]) -> bool {
    let p0 = Vector2::<f32>::new(triangle[0].x as f32, triangle[0].y as f32);
    let p1 = Vector2::<f32>::new(triangle[1].x as f32, triangle[1].y as f32);
    let p2 = Vector2::<f32>::new(triangle[2].x as f32, triangle[2].y as f32);
    let p = Vector2::<f32>::new(point.0 as f32, point.1 as f32);

    let c0 = Vector3::<f32>::new(p2.x - p0.x, p1.x - p0.x, p0.x - p.x);
    let c1 = Vector3::<f32>::new(p2.y - p0.y, p1.y - p0.y, p0.y - p.y);
    let u = c0.cross(c1);

    if u.z.abs() < 1.0 {
        return false;
    }

    let r = Vector3::<f32>::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z);

    r.x > 0.0 && r.y > 0.0 && r.z > 0.0
}

const EPSILON: f32 = 0.01;
const EPSILON_SQUARE: f32 = EPSILON * EPSILON;

fn point_in_triangle_bounding_box(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    point: (f32, f32),
) -> bool {
    let x = point.0;
    let y = point.1;
    let x_min: f32 = x1.min(x2.min(x3)) - EPSILON;
    let x_max: f32 = x1.max(x2.max(x3)) + EPSILON;
    let y_min: f32 = y1.min(y2.min(y3)) - EPSILON;
    let y_max: f32 = y1.max(y2.max(y3)) + EPSILON;

    !(x < x_min || x_max < x || y < y_min || y_max < y)
}

fn distance_square_point_to_segment(x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) -> f32 {
    let p1_p2_square_length: f32 = (x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1);
    let dot_product: f32 = ((x - x1) * (x2 - x1) + (y - y1) * (y2 - y1)) / p1_p2_square_length;
    if dot_product < 0.0 {
        (x - x1) * (x - x1) + (y - y1) * (y - y1)
    } else if dot_product <= 1.0 {
        let p_p1_square_length: f32 = (x1 - x) * (x1 - x) + (y1 - y) * (y1 - y);
        p_p1_square_length - dot_product * dot_product * p1_p2_square_length
    } else {
        (x - x2) * (x - x2) + (y - y2) * (y - y2)
    }
}

fn point_in_triangle(point: (usize, usize), triangle: &[Vector2<u32>]) -> bool {
    let x1 = triangle[0].x as f32;
    let y1 = triangle[0].y as f32;
    let x2 = triangle[1].x as f32;
    let y2 = triangle[1].y as f32;
    let x3 = triangle[2].x as f32;
    let y3 = triangle[2].y as f32;
    let x = point.0 as f32;
    let y = point.1 as f32;

    if !point_in_triangle_bounding_box(x1, y1, x2, y2, x3, y3, (x, y)) {
        return false;
    }

    if naive_point_in_triangle(point, triangle) {
        return true;
    }
    if distance_square_point_to_segment(x1, y1, x2, y2, x, y) <= EPSILON_SQUARE {
        return true;
    }
    if distance_square_point_to_segment(x2, y2, x3, y3, x, y) <= EPSILON_SQUARE {
        return true;
    }
    if distance_square_point_to_segment(x3, y3, x1, y1, x, y) <= EPSILON_SQUARE {
        return true;
    }
    false
}

#[test]
fn test_point_in_triangle() {
    let mut tri: Vec<Vector2<u32>> = Vec::with_capacity(3);
    tri.push(Vector2::<u32>::new(245, 391));
    tri.push(Vector2::<u32>::new(115, 200));
    tri.push(Vector2::<u32>::new(306, 438));

    let mut point = (234, 357);
    assert!(point_in_triangle(point, tri.as_ref()));
    point = (236, 277);
    assert!(!point_in_triangle(point, tri.as_ref()));

    tri.clear();
    tri.push(Vector2::<u32>::new(375, 186));
    tri.push(Vector2::<u32>::new(2, 257));
    tri.push(Vector2::<u32>::new(483, 5));

    point = (340, 110);
    assert!(point_in_triangle(point, tri.as_ref()));
    point = (288, 82);
    assert!(!point_in_triangle(point, tri.as_ref()));
    point = (375, 186);
    assert!(point_in_triangle(point, tri.as_ref()));
}


/// Draw triangle from given vertex positions.
pub fn draw(
    triangle: &[Vector2<u32>],
    color: color::Color,
    buffer: &mut [u32],
    buffer_width: usize,
) {

    let (bb_min_x, bb_min_y, bb_max_x, bb_max_y) = bounding_box(triangle);

    for y in bb_min_y..(bb_max_y) {
        let line = line::LineIterator::new(bb_min_x, y, bb_max_x, y);
        for point in line.filter(|p| point_in_triangle(*p, triangle)) {
            buffer[utils::xy(point.0, point.1, buffer_width)] = color.bgra();
        }
    }
}


pub struct TriangleIterator<'a> {
    bb_min_x: u32,
    bb_max_x: u32,
    bb_max_y: u32,
    triangle: &'a [Vector2<u32>],
    y: u32,
}

impl<'a> TriangleIterator<'a> {
    pub fn new(triangle: &'a [Vector2<u32>]) -> TriangleIterator {
        let (bb_min_x, bb_min_y, bb_max_x, bb_max_y) = bounding_box(triangle);
        TriangleIterator {
            bb_min_x: bb_min_x,
            bb_max_x: bb_max_x,
            bb_max_y: bb_max_y,
            triangle: triangle,
            y: bb_min_y,
        }
    }
}

impl<'a> Iterator for TriangleIterator<'a> {
    type Item = Vec<(usize, usize)>;

    fn next(&mut self) -> Option<Vec<(usize, usize)>> {
        if self.y > self.bb_max_y {
            return None;
        }
        self.y += 1;
        Some(
            line::LineIterator::new(self.bb_min_x, self.y, self.bb_max_x, self.y)
                .filter(|p| point_in_triangle(*p, self.triangle))
                .collect(),
        )
    }
}
