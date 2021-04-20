use super::{Pattern, RegionTree, update_simple, REGION_SIZE};
use crate::pattern::get_patterns;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RegionHashState {
    Empty,
    Normal,
    Recording(u64), // ID of the ongoing recording
    Playing(usize),
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct HashedPattern {
    pub hash: u64,

    // Coordinates used for AABB
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

impl HashedPattern {
    pub fn hash(pattern: &Pattern) -> Self {
        let mut hasher = DefaultHasher::new();

        hasher.write_usize(pattern.width);
        hasher.write_usize(pattern.height);
        hasher.write_u8(pattern.x.rem_euclid(2) as u8 + ((pattern.y.rem_euclid(2)) as u8) * 2);
        pattern.contents.hash(&mut hasher);

        Self {
            hash: hasher.finish(),
            min_x: pattern.min_x,
            min_y: pattern.min_y,
            max_x: pattern.max_x,
            max_y: pattern.max_y,
        }
    }

    pub fn intersect(&self, other: &HashedPattern, margin: i64) -> bool {
        self.hash == other.hash
            && self.min_x <= (other.max_x + margin)
            && (self.max_x + margin) >= other.min_x
            && self.min_y <= (other.max_y + margin)
            && (self.max_y + margin) >= other.min_y
    }
}

pub struct Recording {
    pub dx: i64,
    pub dy: i64,

    pub min_x: i64,
    pub max_x: i64,
    pub min_y: i64,
    pub max_y: i64,

    pub states: Vec<(HashedPattern, Vec<usize>)>,
    pub cell_permutations: Vec<usize>,
}

struct OngoingRecording {
    pub reference: HashedPattern,
    pub period: usize,
    pub dx: i64,
    pub dy: i64,

    pub states: Vec<(HashedPattern, Vec<usize>)>,
    pub cells: Vec<usize>,

    pub second_phase: bool,
}

impl OngoingRecording {
    pub fn begin(pattern: Pattern, reference: HashedPattern, period: usize) -> Self {
        let dx = pattern.min_x - reference.min_x;
        let dy = pattern.min_y - reference.min_y;
        let cells = vec![0]
            .into_iter()
            .chain(pattern.cells.iter().filter(|x| **x > 0).map(|x| *x))
            .collect::<Vec<_>>();
        let mut states = Vec::with_capacity(period);
        let mut n = 0;
        states.push((
            reference,
            pattern
                .cells
                .iter()
                .map(|c| {
                    if *c == 0 {
                        0
                    } else {
                        n += 1;
                        n
                    }
                })
                .collect::<Vec<_>>(),
        ));

        Self {
            reference,
            period,
            cells,
            dx,
            dy,
            states,
            second_phase: false,
        }
    }
}

pub struct HashSimulator {
    tree: RegionTree,
    pub max_size: usize,
    pub max_period: usize,

    pub candidates: VecDeque<(HashedPattern, usize)>,
    pub recordings: Vec<Recording>,
    pub learned_patterns: HashMap<Pattern, usize>,
    pub current_patterns: Vec<(usize, usize, i64, i64)>, // recording ID, time start, start x, start y
    ongoing_recordings: Vec<OngoingRecording>,
}

impl HashSimulator {
    pub fn new(tree: RegionTree, max_size: usize) -> Self {
        Self {
            tree,
            max_size,
            ..Self::default()
        }
    }

    pub fn tick(&mut self) {
        if self.tree.step % (REGION_SIZE - 2) == 0 {
            self.update_regions();
        }

        // Update the normal regions (TODO: move this back in region.rs?)
        if self.tree.step % 2 == 0 {
            // Easy
            for mut region in self.tree.regions.iter_mut() {
                if region.state != RegionHashState::Normal {
                    continue
                }
                update_simple(&mut region, &mut self.tree.cells, 0, REGION_SIZE / 2);
            }
        } else {
            // Do the easy ones first
            for mut region in self.tree.regions.iter_mut() {
                if region.state != RegionHashState::Normal {
                    continue
                }
                update_simple(&mut region, &mut self.tree.cells, 1, REGION_SIZE / 2 - 1);
            }
            // Now do the hard ones
            for i in 0..self.tree.regions.len() {
                let right = self.tree.regions[i].neighbors[2];
                let down = self.tree.regions[i].neighbors[4];

                // Right edge
                if let Some(right) = right {
                    if
                        self.tree.regions[i].state == RegionHashState::Normal
                        || self.tree.regions[right].state == RegionHashState::Normal
                    {
                        let a = REGION_SIZE - 1;
                        for sb in 0..(REGION_SIZE / 2 - 1) {
                            let b = sb + sb + 1;
                            self.tree.update_single(a, b, i, right, right, i);
                        }
                    }
                }

                // Down edge
                if let Some(down) = down {
                    if
                        self.tree.regions[i].state == RegionHashState::Normal
                        || self.tree.regions[down].state == RegionHashState::Normal
                    {
                        let a = REGION_SIZE - 1;
                        for sb in 0..(REGION_SIZE / 2 - 1) {
                            let b = sb + sb + 1;
                            self.tree.update_single(b, a, i, i, down, down);
                        }
                    }
                }

                {
                    // Corner
                    let x = REGION_SIZE - 1;
                    let y = REGION_SIZE - 1;
                    let right = self.tree.regions[i].neighbors[2];
                    let downright = self.tree.regions[i].neighbors[3];
                    let down = self.tree.regions[i].neighbors[4];
                    if let (Some(right), Some(downright), Some(down)) = (right, downright, down) {
                        if
                            self.tree.regions[i].state == RegionHashState::Normal
                            || self.tree.regions[down].state == RegionHashState::Normal
                            || self.tree.regions[downright].state == RegionHashState::Normal
                            || self.tree.regions[right].state == RegionHashState::Normal
                        {
                            self.tree.update_single(x, y, i, right, downright, down);
                        }
                    }
                }
            }
        }

        // TODO: do the stuff with the other regions

        if self.tree.step % 2 == 0 {
            self.match_patterns()
        }
    }

    pub fn update_regions(&mut self) {
        self.tree.update_regions();

        // TODO: look for newly-created regions
        // unimplemented!();
    }

    pub fn match_patterns(&mut self) {
        unimplemented!();
        // let patterns = get_patterns(&self.tree, self.max_size);

        // let mut unmatched_patterns = Vec::new();

        // 'a: for pattern in patterns {
        //     let hashed = HashedPattern::hash(&pattern);
        //     for (index, t, sx, sy) in &self.current_patterns {
        //         let min_x = self.recordings[*index].min_x + sx;
        //         let max_x = self.recordings[*index].max_x + sx;
        //         let min_y = self.recordings[*index].min_y + sy;
        //         let max_y = self.recordings[*index].max_y + sy;
        //         if min_x <= pattern.max_x
        //             && max_x >= pattern.min_x
        //             && min_y <= pattern.max_y
        //             && max_y >= pattern.min_y
        //         {
        //             let expected = &self.recordings[*index].states[self.tree.step - t].0;
        //             if expected.hash != hashed.hash {
        //                 // Broken pattern!
        //                 unimplemented!();
        //             }
        //             continue 'a;
        //         }
        //     }
        //     let margin = 4;
        //     for recording in &self.ongoing_recordings {
        //         if pattern.min_x <= (recording.reference.max_x + margin)
        //             && (pattern.max_x + margin) >= recording.reference.min_x
        //             && pattern.min_y <= (recording.reference.max_y + margin)
        //             && (pattern.max_y + margin) >= recording.reference.min_y
        //         {
        //             continue 'a
        //         }
        //     }
        //     for (candidate, t) in &self.candidates {
        //         if candidate.intersect(&hashed, 4) {
        //             self.ongoing_recordings.push(OngoingRecording::begin(
        //                 pattern,
        //                 *candidate,
        //                 self.tree.step - t,
        //             ));
        //             continue 'a;
        //         }
        //     }
        //     unmatched_patterns.push(pattern);
        // }
    }
}

impl Default for HashSimulator {
    fn default() -> Self {
        Self {
            tree: RegionTree::new(),
            max_size: 0,
            max_period: 16,

            candidates: VecDeque::new(),
            recordings: Vec::new(),
            learned_patterns: HashMap::new(),
            ongoing_recordings: Vec::new(),
            current_patterns: Vec::new(),
        }
    }
}

