extern crate test;

use self::test::Bencher;
use std::sync;
use std::path::Path;
use cgmath::*;
use image;

use model;
use gl;
use shaders;
use color;
use utils;
use line;
use triangle;

const WINDOW_WIDTH: u32 = 512;
const WINDOW_HEIGHT: u32 = 512;

#[test]
fn test_lines() {
    let testmodelpath = Path::new("./content/monkey.obj");
    let testmodel = model::Model::load(testmodelpath).unwrap();
    let mut fb: Vec<u32> = vec![0; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];
    let fb_width = WINDOW_WIDTH as usize;
    let color = color::Color::red();
    for face in &testmodel.faces {
        for i in 0..3 {
            let (x0, y0) = utils::screen_to_image_space(
                face.verts[i % 3].pos.x,
                face.verts[i % 3].pos.y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            );
            let (x1, y1) = utils::screen_to_image_space(
                face.verts[(i + 1) % 3].pos.x,
                face.verts[(i + 1) % 3].pos.y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            );
            line::draw(x0, y0, x1, y1, color, &mut fb, fb_width);
        }
    }
    utils::save_buffer_as_image(
        Path::new("./test_output/test_lines_iter.png"),
        &fb,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
}

#[test]
fn test_lines_iter() {
    let testmodelpath = Path::new("./content/monkey.obj");
    let testmodel = model::Model::load(testmodelpath).unwrap();
    let mut fb: Vec<u32> = vec![0; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];
    let fb_width = WINDOW_WIDTH as usize;
    let color = color::Color::red();
    for face in &testmodel.faces {
        for i in 0..3 {
            let (x0, y0) = utils::screen_to_image_space(
                face.verts[i % 3].pos.x,
                face.verts[i % 3].pos.y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            );
            let (x1, y1) = utils::screen_to_image_space(
                face.verts[(i + 1) % 3].pos.x,
                face.verts[(i + 1) % 3].pos.y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            );
            let line = line::LineIterator::new(x0, y0, x1, y1);
            for point in line {
                fb[utils::xy(point.0, point.1, fb_width)] = color.bgra();
            }
        }
    }
    utils::save_buffer_as_image(
        Path::new("./test_output/test_lines_iter.png"),
        &fb,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
}

#[bench]
fn bench_line(b: &mut Bencher) {
    let mut fb: Vec<u32> = vec![0; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];
    let fb_width = WINDOW_WIDTH as usize;
    let color = color::Color::red();
    b.iter(|| {
        line::draw(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, color, &mut fb, fb_width)
    })
}

#[bench]
fn bench_line_iter(b: &mut Bencher) {
    let mut fb: Vec<u32> = vec![0; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];
    let fb_width = WINDOW_WIDTH as usize;
    let color = color::Color::red();
    b.iter(|| {
        let line = line::LineIterator::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
        for point in line {
            fb[utils::xy(point.0, point.1, fb_width)] = color.bgra();
        }
    })
}

#[bench]
fn bench_triangle(b: &mut Bencher) {
    let mut fb: Vec<u32> = vec![0; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];
    let fb_width = WINDOW_WIDTH as usize;
    let color = color::Color::red();
    let mut tri: Vec<Vector2<u32>> = Vec::with_capacity(3);
    tri.push(Vector2::<u32>::new(0, 0));
    tri.push(Vector2::<u32>::new(0, WINDOW_HEIGHT));
    tri.push(Vector2::<u32>::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    b.iter(|| triangle::draw(&tri, color, &mut fb, fb_width));
    utils::save_buffer_as_image(
        Path::new("./test_output/bench_triangle.png"),
        &fb,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
}

#[bench]
fn bench_triangle_iter(b: &mut Bencher) {
    let mut fb: Vec<u32> = vec![0; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];
    let fb_width = WINDOW_WIDTH as usize;
    let color = color::Color::red();
    let mut tri: Vec<Vector2<u32>> = Vec::with_capacity(3);
    tri.push(Vector2::<u32>::new(0, 0));
    tri.push(Vector2::<u32>::new(0, WINDOW_HEIGHT - 1));
    tri.push(Vector2::<u32>::new(WINDOW_WIDTH - 1, WINDOW_HEIGHT - 1));

    b.iter(|| {
        let triangle = triangle::TriangleIterator::new(&tri);
        for line in triangle {
            for point in line {
                fb[utils::xy(point.0, point.1, fb_width)] = color.bgra();
            }
        }
    });
    utils::save_buffer_as_image(
        Path::new("./test_output/bench_triangle_iter.png"),
        &fb,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );
}

#[test]
fn test_head() {
    let mut graphics: gl::Gl = gl::Gl::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    let camera: Vector3<f32> = Vector3::new(2.0, 0.0, 3.0);
    let camera_target: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
    let up: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    let light_pos = Vector3::new(0.0, 0.0, 1.0);

    let view = gl::view_matrix(camera, camera_target, up);
    let mut projection: Matrix4<f32> = Matrix4::identity();
    projection[2][3] = -0.5 / camera.z;

    let head_modelpath = Path::new("./content/african_head/african_head.obj");
    let head_model = model::Model::load(head_modelpath).unwrap();

    let head_diffuse_image = image::open("./content/african_head/african_head_diffuse.tga")
        .unwrap();
    let head_diffuse_tex = sync::Arc::new(head_diffuse_image);
    let head_normals_image = image::open("./content/african_head/african_head_nm.tga").unwrap();
    let head_normals_tex = sync::Arc::new(head_normals_image);
    let head_specular_image = image::open("./content/african_head/african_head_spec.tga").unwrap();
    let head_specular_tex = sync::Arc::new(head_specular_image);

    let mut vs_in: gl::VSInput = gl::VSInput::default();
    vs_in.view = view;
    vs_in.projection = projection;
    vs_in.camera = camera;
    vs_in.camera_target = camera_target;

    let mut ps_in: gl::PSInput = gl::PSInput::default();
    ps_in.textures.push(head_diffuse_tex.clone());
    ps_in.textures.push(head_normals_tex.clone());
    ps_in.textures.push(head_specular_tex.clone());
    ps_in.light_pos = light_pos;
    ps_in.cam_dir = camera - camera_target;

    graphics.draw(
        &head_model,
        shaders::simple_vertex,
        vs_in,
        shaders::spec_pixel,
        ps_in.clone(),
    );

    let ei_modelpath = Path::new("./content/african_head/african_head_eye_inner.obj");
    let ei_model = model::Model::load(ei_modelpath).unwrap();

    let ei_diffuse_image = image::open(
        "./content/african_head/african_head_eye_inner_diffuse\
                                        2.tga",
    ).unwrap();
    let ei_diffuse_tex = sync::Arc::new(ei_diffuse_image);
    let ei_normals_image = image::open("./content/african_head/african_head_eye_inner_nm.tga")
        .unwrap();
    let ei_normals_tex = sync::Arc::new(ei_normals_image);
    let ei_specular_image = image::open(
        "./content/african_head/african_head_eye_inner_spec.\
                                         tga",
    ).unwrap();
    let ei_specular_tex = sync::Arc::new(ei_specular_image);

    ps_in.textures.clear();
    ps_in.textures.push(ei_diffuse_tex);
    ps_in.textures.push(ei_normals_tex);
    ps_in.textures.push(ei_specular_tex);

    graphics.draw(
        &ei_model,
        shaders::simple_vertex,
        vs_in,
        shaders::spec_pixel,
        ps_in,
    );

    graphics.save_framebuffer_as_image(Path::new("./test_output/test_head.png"));
}

fn _test_monkey() {
    let mut graphics: gl::Gl = gl::Gl::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    let camera: Vector3<f32> = Vector3::new(2.0, 0.0, 3.0);
    let camera_target: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
    let up: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    let light_pos = Vector3::new(0.0, 0.0, 1.0);

    let view = gl::view_matrix(camera, camera_target, up);
    let mut projection: Matrix4<f32> = Matrix4::identity();
    projection[2][3] = -0.5 / camera.z;

    let modelpath = Path::new("./content/monkey.obj");
    let model = model::Model::load(modelpath).unwrap();

    let mut vs_in: gl::VSInput = gl::VSInput::default();
    vs_in.view = view;
    vs_in.projection = projection;
    vs_in.camera = camera;
    vs_in.camera_target = camera_target;

    let mut ps_in: gl::PSInput = gl::PSInput::default();
    ps_in.light_pos = light_pos;
    ps_in.cam_dir = camera - camera_target;

    graphics.draw(
        &model,
        shaders::simple_vertex,
        vs_in,
        shaders::simple_pixel,
        ps_in,
    );

    graphics.save_framebuffer_as_image(Path::new("./test_output/test_monkey.png"));
}
