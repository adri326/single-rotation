use std::time::{Duration, Instant};
use std::io::BufRead;
use std::str::FromStr;
use rand::Rng;

pub mod regions;
use regions::*;

const STEPS: usize = 8 * 7;

fn main() {
    let mut tree = RegionTree::new();
    parse_rle(&mut tree);
    // let mut rng = rand::thread_rng();
    let mut total_duration = Duration::new(0, 0);
    loop {
        let offset = (tree.step as i64 / 12) * 2;
        for y in -4..=14 {
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
        for _ in 0..STEPS {
            tree.tick();
        }
        total_duration += start.elapsed();
        let sps = (tree.step as f64 / total_duration.as_micros() as f64) * 1.0e6;
        print!("\x1b[0K");
        print!("   {:?} steps/s", sps);
        print!("\x1b[20F");
        if let Some(duration) = Duration::new(0, 1000_000_000).checked_sub(start.elapsed()) {
            std::thread::sleep(duration);
        }
    }
}

fn parse_rle(tree: &mut RegionTree) {
    while let Some(Ok(rle)) = std::io::stdin().lock().lines().next() {
        let mut count = String::new();
        let mut x = 0;
        let mut sx = 0;
        let mut y = 0;
        let mut input_x = false;
        let mut input_y = false;
        for c in rle.chars() {
            if c == 'x' {
                input_x = true;
            } if c == 'y' {
                input_y = true;
            } else if c >= '0' && c <= '9' {
                count.push(c);
            } else if input_x {
                if count.len() > 0 {
                    sx = count.parse::<i64>().unwrap();
                    x = sx;
                    count = String::new();
                    input_x = false;
                }
            } else if input_y {
                if count.len() > 0 {
                    y = count.parse::<i64>().unwrap();
                    count = String::new();
                    input_y = false;
                }
            } else if c == 'o' {
                if count.len() > 0 {
                    for _ in 0..usize::from_str(&count).unwrap() {
                        tree.insert(x, y);
                        x += 1;
                    }
                    count = String::new();
                } else {
                    tree.insert(x, y);
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
        }
    }
}
