use std::hash::{Hash, Hasher};
use super::{nearest_region, Region, RegionTree, REGION_SIZE, NEIGHBORS};

pub fn get_island(tree: &RegionTree, x: i64, y: i64, max_size: usize) -> Vec<(i64, i64)> {
    let (x, y) = nearest_region(x, y);
    const R: i64 = REGION_SIZE as i64;
    let mut stack = vec![(x, y)];
    let mut res = Vec::new();
    let mut explored = Vec::new();

    while let Some((x, y)) = stack.pop() {
        if explored.iter().any(|(x2, y2)| *x2 == x && *y2 == y) {
            continue
        }
        explored.push((x, y));
        if let Some(region) = tree.hashmap.get(&(x, y)) {
            let region = &tree.regions[*region];
            if region.n_cells > 0 {
                res.push((x, y));
                if max_size > 0 && res.len() >= max_size {
                    break
                }
                for (dx, dy) in NEIGHBORS.iter() {
                    let nx = x + dx * R;
                    let ny = y + dy * R;
                    stack.push((nx, ny));
                }
            }
        }
    }

    res
}


pub fn get_islands(tree: &RegionTree, max_size: usize) -> Vec<Vec<(i64, i64)>> {
    let mut islands: Vec<Vec<(i64, i64)>> = Vec::new();

    for region in &tree.regions {
        if region.n_cells == 0
            || islands.iter().any(|island| island.iter().any(|(x, y)| *x == region.x && *y == region.y)) {
            continue
        }

        let island = get_island(tree, region.x, region.y, max_size);
        if (max_size == 0 || island.len() < max_size) && island.len() != 0 {
            islands.push(island);
        }
    }

    islands
}

#[test]
fn test_get_island() {
    let mut tree = RegionTree::new();
    tree.insert(0, 0, 0);
    assert_eq!(get_island(&tree, 0, 0, 0), vec![(0, 0)]);
    assert_eq!(get_islands(&tree, 0), vec![vec![(0, 0)]]);
    tree.insert(REGION_SIZE as i64, 0, 0);
    assert_eq!(get_island(&tree, 0, 0, 0), vec![(0, 0), (REGION_SIZE as i64, 0)]);
    assert_eq!(get_islands(&tree, 0), vec![vec![(0, 0), (REGION_SIZE as i64, 0)]]);
    assert_eq!(get_island(&tree, 0, 0, 1), vec![(0, 0)]);
    assert_eq!(get_islands(&tree, 1).len(), 0);
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pattern {
    pub contents: Vec<bool>,
    pub width: usize,
    pub height: usize,
    pub x: i64,
    pub y: i64,

    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

pub fn get_pattern(tree: &RegionTree, x: i64, y: i64, max_size: usize) -> Option<Pattern> {
    let regions = get_island(tree, x, y, max_size).into_iter().map(|i| &tree.regions[tree.hashmap[&i]]).collect::<Vec<_>>();

    if regions.len() == 0 {
        return None
    }

    Some(isolate_pattern(x, y, regions))
}

pub fn get_patterns(tree: &RegionTree, max_size: usize) -> Vec<Pattern> {
    let islands = get_islands(tree, max_size);
    let mut res = Vec::with_capacity(islands.len());

    for island in islands {
        let (x, y) = island[0];
        let regions = island.into_iter().map(|i| &tree.regions[tree.hashmap[&i]]).collect::<Vec<_>>();
        res.push(isolate_pattern(x, y, regions));
    }

    res
}

pub fn isolate_pattern(x: i64, y: i64, regions: Vec<&Region>) -> Pattern {
    let mut min_x = x;
    let mut max_x = x;
    let mut min_y = y;
    let mut max_y = y;

    // get width and height
    for region in &regions {
        for (y, row) in region.cells.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell > 0 {
                    min_x = min_x.min(x as i64 + region.x);
                    max_x = max_x.max(x as i64 + region.x);
                    min_y = min_y.min(y as i64 + region.y);
                    max_y = max_y.max(y as i64 + region.y);
                }
            }
        }
    }

    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;

    let mut contents = vec![false; width * height];

    for region in &regions {
        for (y, row) in region.cells.iter().enumerate() {
            let y = region.y + y as i64;
            if y >= min_y && y <= max_y {
                for (x, &cell) in row.iter().enumerate() {
                    let x = region.x + x as i64;
                    if cell > 0 && x >= min_x && x <= max_x {
                        contents[(x - min_x) as usize + ((y - min_y) as usize) * width] = true;
                    }
                }
            }
        }
    }

    println!("{:?}", contents);

    Pattern {
        contents,
        width,
        height,
        x: min_x,
        y: min_y,
        min_x,
        max_x,
        min_y,
        max_y,
    }
}

#[test]
fn test_get_pattern() {
    let mut tree = RegionTree::new();
    tree.insert(0, 0, 0);

    assert_eq!(get_pattern(&tree, 0, 0, 0), Some(Pattern {
        contents: vec![true],
        width: 1,
        height: 1,
        x: 0,
        y: 0
    }));
    assert_eq!(get_patterns(&tree, 0), vec![Pattern {
        contents: vec![true],
        width: 1,
        height: 1,
        x: 0,
        y: 0
    }]);

    tree.insert(REGION_SIZE as i64, 0, 0);

    {
        let mut contents = Vec::new();
        contents.push(true);
        for _ in 1..REGION_SIZE {
            contents.push(false);
        }
        contents.push(true);
        let pattern = Pattern {
            contents: contents,
            width: 1 + REGION_SIZE,
            height: 1,
            x: 0,
            y: 0
        };
        assert_eq!(get_pattern(&tree, 0, 0, 0), Some(pattern.clone()));

        assert_eq!(get_patterns(&tree, 0), vec![pattern]);
        assert_eq!(get_patterns(&tree, 1), vec![]);
    }
}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
        (self.x.rem_euclid(2) as u8 + ((self.y.rem_euclid(2)) as u8) * 2).hash(state);
        self.contents.hash(state);
    }
}
