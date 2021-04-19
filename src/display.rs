use minifb::{Window, WindowOptions};
use super::lanczos::LanczosInterpolator;
use raqote::{DrawTarget, SolidSource, Source, DrawOptions, PathBuilder};
use std::time::Duration;

pub const WIDTH: usize = 1366;
pub const HEIGHT: usize = 768;
pub const SCALE: f32 = 16.0;
pub const RADIUS: f32 = 6.0;

pub fn spawn() -> Window {
    Window::new("Single Rotation CA", WIDTH, HEIGHT, WindowOptions {
        ..WindowOptions::default()
    }).unwrap()
}

pub fn draw(window: &mut Window, tree: &mut LanczosInterpolator, dt: Duration) {
    let size = window.get_size();
    let mut target = DrawTarget::new(size.0 as i32, size.1 as i32);

    let mut pb = PathBuilder::new();

    let cx = size.0 as f32 / 2.0;
    let cy = size.1 as f32 / 2.0;

    for cell in tree.get(dt) {
        let x = cell.0 * SCALE + cx;
        let y = cell.1 * SCALE + cy;
        pb.move_to(x, y);
        pb.arc(x, y, RADIUS, 0.0, std::f32::consts::PI * 2.0);
    }
    target.fill(&pb.finish(), &Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, 0xff, 0xff, 0xff)), &DrawOptions::new());
    window.update_with_buffer(target.get_data(), size.0, size.1).unwrap();
}
