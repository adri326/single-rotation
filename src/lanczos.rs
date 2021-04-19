use crate::regions::RegionTree;
use std::collections::VecDeque;
use std::time::Duration;

pub const LANCZOS_WIDTH: f32 = 4.0;
pub type Prec = f64;

/// The "sinc" mathematical function
pub fn sinc(x: Prec) -> Prec {
    let a = x.abs();

    if a > 1.0e-8 {
        x.sin() / x
    } else {
        1.0 - x * x / 6.0 // o(x³)
    }
}

/// Returns the kernel for the Lanczos filter
pub fn lanczos_kernel(order: usize, points: usize) -> Vec<Prec> {
    let order_float = order as Prec;
    let points_float = points as Prec;

    let mut res = Vec::with_capacity(points * 2 + 2);

    for ix in -(order as isize * points as isize)..=(order as isize * points as isize + 1) {
        let x = ix as Prec / points_float * std::f64::consts::PI as Prec;
        res.push(sinc(x) * sinc(x / order_float));
    }

    res
}

/// A wrapper around RegionTree that interpolates the points based on the lanczos filter
pub struct LanczosInterpolator {
    pub kernel: Vec<Prec>,
    pub order: usize,
    pub timesteps: usize,
    pub time: f32,
    pub smoothing: usize,
    pub interval: u32,
    pub step_delta: usize,

    pub states: VecDeque<Vec<(i64, i64)>>,
    pub tree: RegionTree,
}

impl LanczosInterpolator {
    pub fn new(tree: RegionTree, order: usize, timesteps: usize, smoothing: usize, interval: u32, step_delta: usize) -> Self {
        Self {
            kernel: lanczos_kernel(order, timesteps * smoothing),
            order,
            timesteps,
            time: 0.0,
            smoothing,
            interval,
            step_delta,

            states: VecDeque::with_capacity(2 * order * smoothing),
            tree
        }
    }

    pub fn required_states(&self) -> usize {
        // self.kernel.len()
        2 * self.order * self.smoothing
    }

    pub fn get(&mut self, dt: Duration) -> Vec<(f32, f32)> {
        let dt = dt.as_millis() as f32 / self.interval as f32;
        self.time += dt;

        while self.time.floor() >= 1.0 {
            self.time -= 1.0;
            self.states.pop_front();
        }

        while self.states.len() < self.required_states() {
            self.states.push_back(self.tree.cells.clone());
            for _ in 0..self.step_delta {
                self.tree.tick();
            }
        }

        self.interpolate()
    }

    fn interpolate(&self) -> Vec<(f32, f32)> {
        let offset = (1.0 - self.time) * self.timesteps as f32;
        let offset_int = offset.floor() as usize;
        let offset_frac = offset.fract() as Prec;

        if self.states.len() == 0 {
            return vec![]
        }

        let mut res = vec![(0.0, 0.0); self.states[0].len() - 1];
        let reference = &self.states[self.required_states() / 2];

        for (i, state) in self.states.iter().enumerate() {
            let k = (
                self.kernel[i * self.timesteps + offset_int] * (1.0 - offset_frac)
                + self.kernel[i * self.timesteps + offset_int + 1] * offset_frac
             ) / (self.smoothing as Prec);
            for (j, ((ref mut x, ref mut y), (sx, sy))) in res.iter_mut().zip(state.iter().skip(1)).enumerate() {
                *x += (*sx - reference[j + 1].0) as Prec * k;
                *y += (*sy - reference[j + 1].1) as Prec * k;
            }
        }

        res.into_iter().enumerate().map(|(i, (x, y))| (x as f32 + reference[i + 1].0 as f32, y as f32 + reference[i + 1].1 as f32)).collect()
    }
}
