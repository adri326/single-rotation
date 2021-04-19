use std::collections::HashMap;

/// The width and height of a "region", or partition of the 2D plane
pub const REGION_SIZE: usize = 16;
/// Same as REGION_SIZE, converted to i64
const R: i64 = REGION_SIZE as i64;

/// List of all of the adjacency vectors
pub const NEIGHBORS: [(i64, i64); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// A partition of the 2D plane: contains REGION_SIZE² tiles
#[derive(Debug, Clone)]
pub struct Region {
    pub x: i64,
    pub y: i64,
    pub cells: [[usize; REGION_SIZE]; REGION_SIZE],
    pub neighbors: [Option<usize>; 8],
    pub n_cells: usize,
}

/// Holds a grid of `Region`s and a list of cells
#[derive(Debug, Clone)]
pub struct RegionTree {
    pub regions: Vec<Region>,
    pub tree: HashMap<(i64, i64), usize>,
    pub cells: Vec<(i64, i64)>,
    pub colors: Vec<usize>,
    pub step: usize,
}

impl Region {
    /// Creates a new, empty region at `x`, `y`
    pub fn new(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            cells: [[0; REGION_SIZE]; REGION_SIZE],
            neighbors: [None; 8],
            n_cells: 0,
        }
    }
}

impl RegionTree {
    /// Creates a new, empty `RegionTree`
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            tree: HashMap::new(),
            step: 0,
            cells: vec![(0, 0)],
            colors: vec![0],
        }
    }

    /// Gets the cell at `x`, `y`
    pub fn get(&self, x: i64, y: i64) -> usize {
        let (nearest_x, nearest_y) = get_nearest(x, y);

        if let Some(&region) = self.tree.get(&(nearest_x, nearest_y)) {
            self.regions[region].cells[(y - nearest_y) as usize][(x - nearest_x) as usize]
        } else {
            0
        }
    }

    /// Inserts a cell at `x`, `y`.
    /// Does nothing if a cell already exists there
    pub fn insert(&mut self, x: i64, y: i64, color: usize) {
        let nearest = get_nearest(x, y);
        let region = if let Some(region) = self.tree.get(&nearest) {
            *region
        } else {
            self.insert_empty_region(nearest.0, nearest.1)
        };

        self.regions[region].n_cells += 1;
        if self.regions[region].cells[(y - nearest.1) as usize][(x - nearest.0) as usize] > 0 {
            return
        }
        self.regions[region].cells[(y - nearest.1) as usize][(x - nearest.0) as usize] =
            self.cells.len();
        self.cells.push((x, y));
        self.colors.push(color);

        self.update_regions();
    }

    /// Inserts an empty region at `x`, `y`. Does not verify that there already is an empty region there
    fn insert_empty_region(&mut self, x: i64, y: i64) -> usize {
        let r = self.regions.len();
        self.tree.insert((x, y), r);
        self.regions.push(Region::new(x, y));

        for (i, (dx, dy)) in NEIGHBORS.iter().enumerate() {
            if let Some(&n) = self.tree.get(&(x + R * dx, y + R * dy)) {
                self.regions[n].neighbors[(i + 4) % 8] = Some(r);
                self.regions[r].neighbors[i] = Some(n);
            }
        }

        r
    }

    /// Updates the region, pruning the empty ones and creating a border of empty regions
    pub fn update_regions(&mut self) {
        {
            let mut to_remove = Vec::new();
            for (index, region) in self.regions.iter().enumerate() {
                if region.n_cells == 0 {
                    let mut has_neighbor = false;
                    for neighbor in region.neighbors.iter() {
                        if let Some(n) = neighbor {
                            if self.regions[*n].n_cells > 0 {
                                has_neighbor = true;
                                break
                            }
                        }
                    }
                    if !has_neighbor {
                        to_remove.push(index);
                    }
                }
            }

            if to_remove.len() > 0 {
                let mut n = 0;

                for &index in &to_remove {
                    self.tree.remove(&(self.regions[index].x, self.regions[index].y));
                }
                self.regions.retain(|_| {
                    n += 1;
                    !to_remove.iter().any(|&x| x == n - 1)
                });

            }
        }

        for (index, region) in self.regions.iter_mut().enumerate() {
            self.tree.insert((region.x, region.y), index);
        }

        for region in self.regions.iter_mut() {
            region.neighbors = [
                self.tree.get(&(region.x, region.y - R)).map(|x| *x),
                self.tree.get(&(region.x + R, region.y - R)).map(|x| *x),
                self.tree.get(&(region.x + R, region.y)).map(|x| *x),
                self.tree.get(&(region.x + R, region.y + R)).map(|x| *x),
                self.tree.get(&(region.x, region.y + R)).map(|x| *x),
                self.tree.get(&(region.x - R, region.y + R)).map(|x| *x),
                self.tree.get(&(region.x - R, region.y)).map(|x| *x),
                self.tree.get(&(region.x - R, region.y - R)).map(|x| *x),
            ];
        }

        for region in 0..self.regions.len() {
            if self.regions[region].n_cells > 0 {
                for i in 0..8 {
                    if self.regions[region].neighbors[i].is_none() {
                        let dx = if i >= 1 && i <= 3 {
                            R
                        } else if i >= 5 {
                            -R
                        } else {
                            0
                        };
                        let dy = if i == 2 || i == 6 {
                            0
                        } else if i >= 3 && i <= 5 {
                            R
                        } else {
                            -R
                        };
                        self.insert_empty_region(self.regions[region].x + dx, self.regions[region].y + dy);
                    }
                }
            }
        }
    }

    /// Steps the simulation forward by one generation
    pub fn tick(&mut self) {
        if self.step % (REGION_SIZE - 2) == 0 {
            self.update_regions();
        }

        if self.step % 2 == 0 {
            // Easy
            for mut region in self.regions.iter_mut() {
                if region.n_cells == 0 {
                    continue
                }
                update_simple(&mut region, &mut self.cells, 0, REGION_SIZE / 2);
            }
        } else {
            // Do the easy ones first
            for mut region in self.regions.iter_mut() {
                if region.n_cells == 0 {
                    continue
                }
                update_simple(&mut region, &mut self.cells, 1, REGION_SIZE / 2 - 1);
            }
            // Now do the hard ones
            for i in 0..self.regions.len() {
                {
                    // Edges
                    let a = REGION_SIZE - 1;
                    let right = self.regions[i].neighbors[2];
                    let down = self.regions[i].neighbors[4];
                    for sb in 0..(REGION_SIZE / 2 - 1) {
                        let b = sb + sb + 1;
                        if let Some(right) = right {
                            self.update_single(a, b, i, right, right, i);
                        }
                        if let Some(down) = down {
                            self.update_single(b, a, i, i, down, down);
                        }
                    }
                }
                {
                    // Corner
                    let x = REGION_SIZE - 1;
                    let y = REGION_SIZE - 1;
                    let right = self.regions[i].neighbors[2];
                    let downright = self.regions[i].neighbors[3];
                    let down = self.regions[i].neighbors[4];
                    if let (Some(right), Some(downright), Some(down)) = (right, downright, down) {
                        self.update_single(x, y, i, right, downright, down);
                    }
                }
            }
        }

        self.step += 1;
    }

    /// Update a single 2x2 square, given the set of neighboring regions
    fn update_single(
        &mut self,
        x: usize,
        y: usize,
        a_i: usize,
        b_i: usize,
        c_i: usize,
        d_i: usize,
    ) {
        let a = self.regions[a_i].cells[y][x];
        let b = self.regions[b_i].cells[y][(x + 1) % REGION_SIZE];
        let c = self.regions[c_i].cells[(y + 1) % REGION_SIZE][(x + 1) % REGION_SIZE];
        let d = self.regions[d_i].cells[(y + 1) % REGION_SIZE][x];
        let n = (a > 0) as u8 + (b > 0) as u8 + (c > 0) as u8 + (d > 0) as u8;
        if n == 1 {
            // println!("~ {} {}: {} {} {} {}", x, y, a, b, c, d);
            self.regions[a_i].cells[y][x] = d;
            self.regions[b_i].cells[y][(x + 1) % REGION_SIZE] = a;
            self.regions[c_i].cells[(y + 1) % REGION_SIZE][(x + 1) % REGION_SIZE] = b;
            self.regions[d_i].cells[(y + 1) % REGION_SIZE][x] = c;

            self.regions[a_i].n_cells = self.regions[a_i].n_cells + (d > 0) as usize - (a > 0) as usize;
            self.regions[b_i].n_cells = self.regions[b_i].n_cells + (a > 0) as usize - (b > 0) as usize;
            self.regions[c_i].n_cells = self.regions[c_i].n_cells + (b > 0) as usize - (c > 0) as usize;
            self.regions[d_i].n_cells = self.regions[d_i].n_cells + (c > 0) as usize - (d > 0) as usize;

            self.cells[a] = (
                self.regions[a_i].x + x as i64 + 1,
                self.regions[a_i].y + y as i64,
            );
            self.cells[b] = (
                self.regions[a_i].x + x as i64 + 1,
                self.regions[a_i].y + y as i64 + 1,
            );
            self.cells[c] = (
                self.regions[a_i].x + x as i64,
                self.regions[a_i].y + y as i64 + 1,
            );
            self.cells[d] = (
                self.regions[a_i].x + x as i64,
                self.regions[a_i].y + y as i64,
            );
        }
    }
}

/// Update all of the 2x2 square fully enclosed within a region
#[inline]
fn update_simple(region: &mut Region, cells: &mut Vec<(i64, i64)>, offset: usize, len: usize) {
    for sy in 0..len {
        let y = sy + sy + offset;
        for sx in 0..len {
            let x = sx + sx + offset;
            let a = region.cells[y][x];
            let b = region.cells[y][x + 1];
            let c = region.cells[y + 1][x + 1];
            let d = region.cells[y + 1][x];
            let n = (a > 0) as u8 + (b > 0) as u8 + (c > 0) as u8 + (d > 0) as u8;
            if n == 1 {
                // println!("{} {}: {} {} {} {}", x, y, a, b, c, d);
                region.cells[y][x] = d;
                region.cells[y][x + 1] = a;
                region.cells[y + 1][x + 1] = b;
                region.cells[y + 1][x] = c;
                cells[a] = (region.x + x as i64 + 1, region.y + y as i64);
                cells[b] = (region.x + x as i64 + 1, region.y + y as i64 + 1);
                cells[c] = (region.x + x as i64, region.y + y as i64 + 1);
                cells[d] = (region.x + x as i64, region.y + y as i64);
            }
        }
    }
}

/// Gets the region coordinate of the nearest region (that where the `(x, y)` belongs)
fn get_nearest(x: i64, y: i64) -> (i64, i64) {
    (
        x.div_euclid(R) * R,
        y.div_euclid(R) * R
    )
}
