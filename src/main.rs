use std::time::{Duration, Instant};
use std::io::BufRead;
use std::str::FromStr;
// use rand::Rng;

pub mod regions;
use regions::*;

pub mod display;
pub mod lanczos;
pub mod pattern;
pub mod hash;
use pattern::Pattern;

fn main() {
    let graphical = std::env::args().any(|arg| arg == "-g");
    let mut tree = RegionTree::new();
    let mut steps: usize = 1;
    let mut interval: u32 = 100;
    let mut smoothing: usize = 4;
    parse_rle(&mut tree, &mut steps, &mut interval, &mut smoothing);

    let fps = 1000 / interval;

    // let mut rng = rand::thread_rng();
    let mut total_duration = Duration::new(0, 0);

    if graphical {
        let mut window = display::spawn();
        let mut interpolator = lanczos::LanczosInterpolator::new(tree, 3, (50 / fps as usize).max(1), smoothing, interval, steps);
        println!("");
        let mut previous_time = Instant::now();
        while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
            let start = Instant::now();
            display::draw(&mut window, &mut interpolator, previous_time.elapsed());
            previous_time = start;
            print!("\x1b[1F");
            println!("Step: {}", interpolator.tree.step);
            if let Some(duration) = Duration::new(0, 20_000_000).checked_sub(start.elapsed()) {
                std::thread::sleep(duration);
            }
        }
        return
    }

    loop {
        // let offset = (tree.step as i64 / 12) * 2;

        for y in -4..=24 {
            // for x in (offset - 4)..=(offset + 4) {
            for x in -4..=96 {
                let n = tree.get(x, y);
                if n > 0 {
                    if tree.cells.len() <= 10 {
                        print!("{}", n);
                    } else {
                        print!("#");
                    }
                } else {
                    print!("·");
                }
            }
            print!("\n");
        }

        println!("Step: {}", tree.step);

        let start = Instant::now();
        for _ in 0..steps {
            tree.tick();
        }
        total_duration += start.elapsed();
        let sps = (tree.step as f64 / total_duration.as_micros() as f64) * 1.0e6;
        print!("\x1b[0K");
        print!("   {:?} steps/s", sps);
        print!("\x1b[30F");
        if let Some(duration) = Duration::new(0, interval * 1_000_000).checked_sub(start.elapsed()) {
            std::thread::sleep(duration);
        }
    }
}

fn parse_rle(tree: &mut RegionTree, steps: &mut usize, interval: &mut u32, smoothing: &mut usize) {
    let mut x = 0;
    let mut sx = 0;
    let mut y = 0;
    let mut color = 0;
    while let Some(Ok(rle)) = std::io::stdin().lock().lines().next() {
        let mut count = String::new();
        let mut input_x = false;
        let mut input_y = false;
        let mut input_steps = false;
        let mut input_interval = false;
        let mut input_smoothing = false;
        let mut sign = 1;
        for c in rle.chars() {
            if c == 'x' {
                input_x = true;
            } else if c == 'y' {
                input_y = true;
            } else if c == 's' {
                input_steps = true;
            } else if c == 'i' {
                input_interval = true;
            } else if c == 'u' {
                input_smoothing = true;
            } else if c >= '0' && c <= '9' {
                count.push(c);
            } else if c == '-' {
                sign = -1;
            } else if input_x {
                if count.len() > 0 {
                    sx = count.parse::<i64>().unwrap() * sign as i64;
                    x = sx;
                    sign = 1;
                    count = String::new();
                    input_x = false;
                }
            } else if input_y {
                if count.len() > 0 {
                    y = count.parse::<i64>().unwrap() * sign as i64;
                    sign = 1;
                    count = String::new();
                    input_y = false;
                }
            } else if input_steps {
                if count.len() > 0 {
                    *steps = count.parse::<usize>().unwrap();
                    count = String::new();
                    input_steps = false;
                }
            } else if input_interval {
                if count.len() > 0 {
                    *interval = count.parse::<u32>().unwrap();
                    count = String::new();
                    input_interval = false;
                }
            } else if input_smoothing {
                if count.len() > 0 {
                    *smoothing = count.parse::<usize>().unwrap();
                    count = String::new();
                    input_smoothing = false;
                }
            } else if c == 'o' {
                if count.len() > 0 {
                    for _ in 0..usize::from_str(&count).unwrap() {
                        tree.insert(x, y, color);
                        x += 1;
                    }
                    count = String::new();
                } else {
                    tree.insert(x, y, color);
                    x += 1;
                }
            } else if c == 'b' {
                if count.len() > 0 {
                    x += count.parse::<i64>().unwrap();
                    count = String::new();
                } else {
                    x += 1;
                }
            } else if c == '$' {
                if count.len() > 0 {
                    y += count.parse::<i64>().unwrap();
                    x = sx;
                    count = String::new();
                } else {
                    y += 1;
                    x = sx;
                }
            } else if c == 'c' {
                if count.len() > 0 {
                    color = count.parse::<usize>().unwrap();
                    count = String::new();
                } else {
                    color = 0;
                }
            } else if c == '!' {
                return
            }
        }
        if input_x {
            if count.len() > 0 {
                x = count.parse::<i64>().unwrap();
            }
        } else if input_y {
            if count.len() > 0 {
                y = count.parse::<i64>().unwrap();
            }
        } else if input_steps {
            if count.len() > 0 {
                *steps = count.parse::<usize>().unwrap();
            }
        } else if input_interval {
            if count.len() > 0 {
                *interval = count.parse::<u32>().unwrap();
            }
        } else if input_smoothing {
            if count.len() > 0 {
                *smoothing = count.parse::<usize>().unwrap();
            }
        }
    }
}
