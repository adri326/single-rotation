use crate::regions::RegionTree;
use std::collections::VecDeque;

pub const LANCZOS_WIDTH: f32 = 4.0;

/// The "sinc" mathematical function
pub fn sinc(x: f32) -> f32 {
    let a = x.abs();

    if a > 1.0e-8 {
        x.sin() / x
    } else {
        1.0 - x * x / 6.0 // o(x³)
    }
}

/// Returns the kernel for the Lanczos filter
pub fn lanczos_kernel(order: usize, points: usize) -> Vec<f32> {
    let order_f32 = order as f32;
    let points_f32 = points as f32;

    let mut res = Vec::with_capacity(points * 2 + 1);

    for ix in -(order as isize * points as isize)..=(order as isize * points as isize) {
        let x = ix as f32 / points_f32 * std::f32::consts::PI;
        res.push(sinc(x) * sinc(x / order_f32));
    }

    res
}

/// A wrapper around RegionTree that interpolates the points based on the lanczos filter
pub struct LanczosInterpolator {
    pub kernel: Vec<f32>,
    pub order: usize,
    pub timesteps: usize,
    pub time: usize,
    pub smoothing: usize,
    pub step_delta: usize,

    pub states: VecDeque<Vec<(i64, i64)>>,
    pub tree: RegionTree,
}

impl LanczosInterpolator {
    pub fn new(tree: RegionTree, order: usize, timesteps: usize, smoothing: usize, step_delta: usize) -> Self {
        Self {
            kernel: lanczos_kernel(order, timesteps * smoothing),
            order,
            timesteps,
            time: 0,
            smoothing,
            step_delta,

            states: VecDeque::with_capacity(2 * order * smoothing),
            tree
        }
    }

    pub fn required_states(&self) -> usize {
        // self.kernel.len()
        2 * self.order * self.smoothing
    }

    pub fn get(&mut self) -> Vec<(f32, f32)> {
        self.time += 1;

        while self.time >= self.timesteps {
            self.time -= self.timesteps;
            self.states.pop_front();
        }

        while self.states.len() < self.required_states() {
            self.states.push_back(self.tree.cells.clone());
            for _ in 0..self.step_delta {
                self.tree.tick();
            }
        }

        self.interpolate(self.timesteps - self.time - 1)
    }

    fn interpolate(&self, offset: usize) -> Vec<(f32, f32)> {
        if self.states.len() == 0 {
            return vec![]
        }
        let mut res = vec![(0.0, 0.0); self.states[0].len() - 1];

        for (i, state) in self.states.iter().enumerate() {
            let k = self.kernel[i * self.timesteps + offset] / self.smoothing as f32;
            for ((ref mut x, ref mut y), (sx, sy)) in res.iter_mut().zip(state.iter().skip(1)) {
                *x += *sx as f32 * k;
                *y += *sy as f32 * k;
            }
        }

        res
    }
}
