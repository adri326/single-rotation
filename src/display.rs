use minifb::{Window, WindowOptions};
use super::lanczos::LanczosInterpolator;
use raqote::{DrawTarget, SolidSource, Source, DrawOptions, PathBuilder};
use std::time::Duration;

pub const WIDTH: usize = 1366;
pub const HEIGHT: usize = 768;
pub const SCALE: f32 = 16.0;
pub const RADIUS: f32 = 6.0;

pub const COLORS: [(u8, u8, u8, u8); 5] = [
    (0xff, 0xff, 0xff, 0xff),
    (0xff, 0x9d, 0xbe, 0xb9),
    (0xff, 0xff, 0xc2, 0xb4),
    (0xff, 0xff, 0x88, 0x82),
    (0xff, 0x19, 0x43, 0x50),
];

pub fn spawn() -> Window {
    Window::new("Single Rotation CA", WIDTH, HEIGHT, WindowOptions {
        ..WindowOptions::default()
    }).unwrap()
}

pub fn draw(window: &mut Window, tree: &mut LanczosInterpolator, dt: Duration) {
    let size = window.get_size();
    let mut target = DrawTarget::new(size.0 as i32, size.1 as i32);

    let mut pbs = Vec::new();
    for _ in 0..COLORS.len() {
        pbs.push(PathBuilder::new());
    }

    let cx = size.0 as f32 / 2.0;
    let cy = size.1 as f32 / 2.0;

    for (i, cell) in tree.get(dt).into_iter().enumerate() {
        let x = cell.0 * SCALE + cx;
        let y = cell.1 * SCALE + cy;
        pbs[tree.tree.colors[i + 1]].move_to(x, y);
        pbs[tree.tree.colors[i + 1]].arc(x, y, RADIUS, 0.0, std::f32::consts::PI * 2.0);
    }

    for (i, pb) in pbs.into_iter().enumerate() {
        let color = COLORS[i];
        let f = pb.finish();
        target.fill(
            &f,
            &Source::Solid(SolidSource::from_unpremultiplied_argb(color.0, color.1, color.2, color.3)),
            &DrawOptions::new()
        );
    }
    window.update_with_buffer(target.get_data(), size.0, size.1).unwrap();
}
