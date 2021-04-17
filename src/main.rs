use std::time::{Duration, Instant};
use rand::Rng;

pub mod regions;
use regions::*;

const STEPS: usize = 72;

fn main() {
    let mut tree = RegionTree::new();
    // let mut rng = rand::thread_rng();
    let mut total_duration = Duration::new(0, 0);
    tree.tick();
    tree.insert(1, 0);
    tree.insert(1, 1);
    tree.insert(1, 2);
    tree.insert(4, 1);
    tree.insert(5, 1);
    tree.insert(3, 2);
    // for _ in 0..500 {
    //     tree.insert(rng.gen_range(0..100), rng.gen_range(0..100));
    // }
    // for x in -1..101 {
    //     tree.insert(-1, x);
    //     tree.insert(-2, x);
    //     tree.insert(101, x);
    //     tree.insert(102, x);
    //     tree.insert(x, -1);
    //     tree.insert(x, -2);
    //     tree.insert(x, 101);
    //     tree.insert(x, 102);
    // }
    loop {
        let offset = (tree.step as i64 / 12) * 2;
        for y in -4..=14 {
            // for x in (offset - 4)..=(offset + 4) {
            for x in -4..=96 {
                let n = tree.get(x, y);
                if n > 0 {
                    print!("{}", n);
                } else {
                    print!(" ");
                }
            }
            print!("\n");
        }
        println!("Step: {}", tree.step);

        let start = Instant::now();
        for _ in 0..STEPS {
            tree.tick();
        }
        total_duration += start.elapsed();
        let sps = (tree.step as f64 / total_duration.as_micros() as f64) * 1.0e6;
        print!("\x1b[0K");
        print!("   {:?} steps/s", sps);
        print!("\x1b[20F");
        if let Some(duration) = Duration::new(0, 100_000_000).checked_sub(start.elapsed()) {
            std::thread::sleep(duration);
        }
    }
}
