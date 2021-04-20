use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use super::{Pattern, RegionTree};

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
}

pub struct PatternRecording {
    pub dx: i64,
    pub dy: i64,

    pub states: Vec<(HashedPattern, Vec<usize>)>,
    pub cell_permutations: Vec<usize>,
}

struct OngoingRecording {
    pub reference: HashedPattern,
}

pub struct HashSimulator {
    tree: RegionTree,
    pub max_size: usize,
    pub max_period: usize,

    pub candidates: VecDeque<(HashedPattern, usize)>,
    pub recordings: Vec<PatternRecording>,
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
        self.tree.tick();

        if self.tree.step % 2 == 0 {
            self.match_patterns()
        }
    }

    pub fn match_patterns(&mut self) {
        unimplemented!()
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
