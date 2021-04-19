use super::{nearest_region, RegionTree, REGION_SIZE, NEIGHBORS};

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

#[test]
fn test_get_island() {
    let mut tree = RegionTree::new();
    tree.insert(0, 0, 0);
    assert_eq!(get_island(&tree, 0, 0, 0), vec![(0, 0)]);
    tree.insert(REGION_SIZE as i64, 0, 0);
    assert_eq!(get_island(&tree, 0, 0, 0), vec![(0, 0), (REGION_SIZE as i64, 0)]);
    assert_eq!(get_island(&tree, 0, 0, 1), vec![(0, 0)]);
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ship {
    contents: Vec<bool>,
    width: usize,
    height: usize,
    x: i64,
    y: i64,
}

pub fn get_ship(tree: &RegionTree, x: i64, y: i64, max_size: usize) -> Option<Ship> {
    let regions = get_island(tree, x, y, max_size).into_iter().map(|i| &tree.regions[tree.hashmap[&i]]).collect::<Vec<_>>();
    let mut min_x = x;
    let mut max_x = x;
    let mut min_y = y;
    let mut max_y = y;

    if regions.len() == 0 {
        return None
    }

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

    Some(Ship {
        contents,
        width,
        height,
        x: min_x,
        y: min_y
    })
}

#[test]
fn test_get_ship() {
    let mut tree = RegionTree::new();
    tree.insert(0, 0, 0);

    assert_eq!(get_ship(&tree, 0, 0, 0), Some(Ship {
        contents: vec![true],
        width: 1,
        height: 1,
        x: 0,
        y: 0
    }));

    tree.insert(REGION_SIZE as i64, 0, 0);

    {
        let mut contents = Vec::new();
        contents.push(true);
        for _ in 1..REGION_SIZE {
            contents.push(false);
        }
        contents.push(true);
        assert_eq!(get_ship(&tree, 0, 0, 0), Some(Ship {
            contents: contents,
            width: 1 + REGION_SIZE,
            height: 1,
            x: 0,
            y: 0
        }));
    }
}
