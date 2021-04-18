use minifb::{Window, WindowOptions, ScaleMode, Scale};
use super::regions::RegionTree;
use raqote::{DrawTarget, SolidSource, Source, DrawOptions, PathBuilder, Point, Transform, StrokeStyle};

pub const WIDTH: usize = 600;
pub const HEIGHT: usize = 400;
pub const SCALE: f32 = 8.0;
pub const RADIUS: f32 = 3.0;
pub const CX: f32 = 300.0;
pub const CY: f32 = 200.0;

pub fn spawn() -> Window {
    Window::new("Single Rotation CA", WIDTH, HEIGHT, WindowOptions {
        ..WindowOptions::default()
    }).unwrap()
}

pub fn draw(window: &mut Window, tree: &RegionTree) {
    let size = window.get_size();
    let mut dt = DrawTarget::new(size.0 as i32, size.1 as i32);

    let mut pb = PathBuilder::new();

    for (i, cell) in tree.cells.iter().skip(1).enumerate() {
        let x = cell.0 as f32 * SCALE + CX;
        let y = cell.1 as f32 * SCALE + CY;
        pb.move_to(x, y);
        pb.arc(x, y, RADIUS, 0.0, std::f32::consts::PI * 2.0);
    }
    dt.fill(&pb.finish(), &Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, 0xff, 0xff, 0xff)), &DrawOptions::new());
    window.update_with_buffer(dt.get_data(), size.0, size.1).unwrap();
}
