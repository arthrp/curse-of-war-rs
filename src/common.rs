// Shared constants, enums, and helpers

pub const MAX_PLAYER: usize = 8; // number of players (countries)
pub const NEUTRAL: usize = 0; // neutral player
pub const MAX_CLASS: usize = 1; // classes of units. only one exists.
pub const MAX_WIDTH: usize = 40; // max map width
pub const MAX_HEIGHT: usize = 29; // max map height
pub const DIRECTIONS: usize = 6; // number of neighbors on the grid (hex)
pub const MAX_POP: i32 = 499; // maximum population at a tile (for each player)
pub const MAX_TIMELINE_MARK: usize = 72;

pub const FLAG_ON: i32 = 1;
pub const FLAG_OFF: i32 = 0;
pub const FLAG_POWER: i32 = 8;
pub const RANDOM_INEQUALITY: i32 = -1;
pub const MAX_AVLBL_LOC: usize = 7;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConfigSpeed {
    Pause,
    Slowest,
    Slower,
    Slow,
    Normal,
    Fast,
    Faster,
    Fastest,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConfigDif {
    Easiest,
    Easy,
    Normal,
    Hard,
    Hardest,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnitClass {
    Citizen = 0,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TileClass {
    Abyss = 0,
    Mountain = 1,
    Mine = 2,
    Grassland = 3,
    Village = 4,
    Town = 5,
    Castle = 6,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Stencil {
    Rhombus,
    Rect,
    Hex,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Loc {
    pub i: i32,
    pub j: i32,
}

pub const DIRS: [Loc; DIRECTIONS] = [
    Loc { i: -1, j: 0 },
    Loc { i: 1, j: 0 },
    Loc { i: 0, j: -1 },
    Loc { i: 0, j: 1 },
    Loc { i: 1, j: -1 },
    Loc { i: -1, j: 1 },
];

#[inline]
pub fn min_i32(x: i32, y: i32) -> i32 {
    if x < y { x } else { y }
}

#[inline]
pub fn max_i32(x: i32, y: i32) -> i32 {
    if x < y { y } else { x }
}

#[inline]
pub fn in_segment(x: i32, l: i32, r: i32) -> i32 {
    if x < l { l } else if x > r { r } else { x }
}


