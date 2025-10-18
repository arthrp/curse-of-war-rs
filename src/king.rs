use crate::common::*;
use crate::grid::*;
use rand::Rng;

#[derive(Copy, Clone, Debug, Default)]
pub struct Country {
    pub gold: i64,
}

pub const PRICE_VILLAGE: i64 = 160;
pub const PRICE_TOWN: i64 = 240;
pub const PRICE_CASTLE: i64 = 320;

#[derive(Copy, Clone, Debug)]
pub enum Strategy {
    None,
    AggrGreedy,
    OneGreedy,
    PersistentGreedy,
    Opportunist,
    Noble,
    Midas,
}

#[derive(Clone, Debug, Copy)]
pub struct King {
    pub value: [[i32; MAX_HEIGHT]; MAX_WIDTH],
    pub pl: i32,
    pub strategy: Strategy,
}

pub fn build(g: &mut Grid, c: &mut Country, pl: i32, i: i32, j: i32) -> i32 {
    if i >= 0 && i < g.width && j >= 0 && j < g.height && g.tiles[i as usize][j as usize].pl == pl {
        let (price, cl) = match g.tiles[i as usize][j as usize].cl {
            TileClass::Grassland => (PRICE_VILLAGE, TileClass::Village),
            TileClass::Village => (PRICE_TOWN, TileClass::Town),
            TileClass::Town => (PRICE_CASTLE, TileClass::Castle),
            _ => return -1,
        };
        if c.gold >= price {
            g.tiles[i as usize][j as usize].cl = cl;
            c.gold -= price;
            return 0;
        }
    }
    -1
}

pub fn degrade(g: &mut Grid, i: i32, j: i32) -> i32 {
    if i >= 0 && i < g.width && j >= 0 && j < g.height {
        let cl = match g.tiles[i as usize][j as usize].cl {
            TileClass::Village => TileClass::Grassland,
            TileClass::Town => TileClass::Village,
            TileClass::Castle => TileClass::Town,
            _ => return -1,
        };
        g.tiles[i as usize][j as usize].cl = cl;
        return 0;
    }
    -1
}

pub fn king_evaluate_map(k: &mut King, g: &Grid, dif: ConfigDif) {
    let mut u = [[0i32; MAX_HEIGHT]; MAX_WIDTH];
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            u[i][j] = 0;
            k.value[i][j] = 0;
        }
    }
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            if is_inhabitable(g.tiles[i][j].cl) {
                k.value[i][j] += 1;
            }
            match k.strategy {
                Strategy::PersistentGreedy => {
                    if is_inhabitable(g.tiles[i][j].cl) {
                        k.value[i][j] += 1;
                    }
                }
                _ => {}
            }
            match g.tiles[i][j].cl {
                TileClass::Castle => {
                    if matches!(k.strategy, Strategy::Noble) {
                        spread(g, &mut u, &mut k.value, i as i32, j as i32, 32, 1);
                    } else {
                        spread(g, &mut u, &mut k.value, i as i32, j as i32, 16, 1);
                    }
                    even(g, &mut k.value, i as i32, j as i32, 0);
                }
                TileClass::Town => {
                    spread(g, &mut u, &mut k.value, i as i32, j as i32, 8, 1);
                    even(g, &mut k.value, i as i32, j as i32, 0);
                }
                TileClass::Village => {
                    if matches!(k.strategy, Strategy::Noble) {
                        spread(g, &mut u, &mut k.value, i as i32, j as i32, 2, 1);
                    } else {
                        spread(g, &mut u, &mut k.value, i as i32, j as i32, 4, 1);
                    }
                    even(g, &mut k.value, i as i32, j as i32, 0);
                }
                TileClass::Mine => {
                    for d in 0..DIRECTIONS {
                        let ii = i as i32 + DIRS[d].i;
                        let jj = j as i32 + DIRS[d].j;
                        if matches!(k.strategy, Strategy::Midas) {
                            spread(g, &mut u, &mut k.value, ii, jj, 8, 1);
                        } else {
                            spread(g, &mut u, &mut k.value, ii, jj, 4, 1);
                        }
                        even(g, &mut k.value, ii, jj, 0);
                    }
                }
                _ => {}
            }
        }
    }
    // dumb down kings
    let mut rng = rand::thread_rng();
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            match dif {
                ConfigDif::Easiest => {
                    let mut x = k.value[i][j] / 4;
                    x = x + (rng.gen_range(0..7) as i32) - 3;
                    k.value[i][j] = max_i32(0, x);
                }
                ConfigDif::Easy => {
                    let mut x = k.value[i][j] / 2;
                    x = x + (rng.gen_range(0..3) as i32) - 1;
                    k.value[i][j] = max_i32(0, x);
                }
                _ => {}
            }
        }
    }
}

pub fn king_init(k: &mut King, pl: i32, strat: Strategy, _g: &Grid, _dif: ConfigDif) {
    k.pl = pl;
    k.strategy = strat;
}

pub fn builder_default(k: &King, c: &mut Country, g: &mut Grid, _fg: &mut FlagGrid) -> i32 {
    let mut i_best = 0i32;
    let mut j_best = 0i32;
    let mut v_best: f32 = 0.0;
    for i in 0..g.width {
        for j in 0..g.height {
            let t = &g.tiles[i as usize][j as usize];
            let ok = t.pl == k.pl && is_inhabitable(t.cl) && {
                let mut all_mine = true;
                for n in 0..DIRECTIONS {
                    let di = DIRS[n].i;
                    let dj = DIRS[n].j;
                    let ni = i + di;
                    let nj = j + dj;
                    if ni >= 0
                        && ni < g.width
                        && nj >= 0
                        && nj < g.height
                        && is_inhabitable(g.tiles[ni as usize][nj as usize].cl)
                    {
                        all_mine = all_mine && (g.tiles[ni as usize][nj as usize].pl == k.pl);
                    }
                }
                all_mine
            };
            let army =
                g.tiles[i as usize][j as usize].units[k.pl as usize][UnitClass::Citizen as usize];
            let mut enemy = 0;
            for p in 0..MAX_PLAYER {
                if p as i32 != k.pl {
                    enemy += g.tiles[i as usize][j as usize].units[p][UnitClass::Citizen as usize];
                }
            }
            let base = match g.tiles[i as usize][j as usize].cl {
                TileClass::Grassland => 1.0,
                TileClass::Village => 8.0,
                TileClass::Town => 32.0,
                _ => 0.0,
            };
            let mut base = base;
            if matches!(k.strategy, Strategy::Midas) {
                base *= (k.value[i as usize][j as usize] + 10) as f32;
            }
            let mut v = if ok {
                base * (MAX_POP as f32 - army as f32)
            } else {
                0.0
            };
            if (army as i32) < (MAX_POP / 10) {
                v = 0.0;
            }
            if v > 0.0 && v > v_best {
                i_best = i;
                j_best = j;
                v_best = v;
            }
        }
    }
    if v_best > 0.0 {
        build(g, c, k.pl, i_best, j_best)
    } else {
        -1
    }
}

fn action_aggr_greedy(k: &King, g: &Grid, fg: &mut FlagGrid) {
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            if fg.flag[i][j] == FLAG_ON {
                remove_flag(g, fg, i as i32, j as i32, FLAG_POWER);
            }
            let army = g.tiles[i][j].units[k.pl as usize][UnitClass::Citizen as usize];
            let mut enemy = 0;
            for p in 0..MAX_PLAYER {
                if p as i32 != k.pl {
                    enemy += g.tiles[i][j].units[p][UnitClass::Citizen as usize];
                }
            }
            let v = (k.value[i][j] as f32)
                * (2.0 * enemy as f32 - army as f32)
                * (army as f32).powf(0.5);
            if v > 5000.0 {
                add_flag(g, fg, i as i32, j as i32, FLAG_POWER);
            }
        }
    }
}

fn action_one_greedy(k: &King, g: &Grid, fg: &mut FlagGrid) {
    let mut i_best = 0usize;
    let mut j_best = 0usize;
    let mut v_best = -1.0f32;
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            if fg.flag[i][j] == FLAG_ON {
                remove_flag(g, fg, i as i32, j as i32, FLAG_POWER);
            }
            let army = g.tiles[i][j].units[k.pl as usize][UnitClass::Citizen as usize];
            let mut enemy = 0;
            for p in 0..MAX_PLAYER {
                if p as i32 != k.pl {
                    enemy += g.tiles[i][j].units[p][UnitClass::Citizen as usize];
                }
            }
            let v = (k.value[i][j] as f32)
                * (5.0 * enemy as f32 - army as f32)
                * (army as f32).powf(0.5);
            if v > v_best && v > 5000.0 {
                v_best = v;
                i_best = i;
                j_best = j;
            }
        }
    }
    if v_best > 0.0 {
        add_flag(g, fg, i_best as i32, j_best as i32, FLAG_POWER);
    }
}

fn action_persistent_greedy(k: &King, g: &Grid, fg: &mut FlagGrid) {
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            let army = g.tiles[i][j].units[k.pl as usize][UnitClass::Citizen as usize];
            let mut enemy = 0;
            for p in 0..MAX_PLAYER {
                if p as i32 != k.pl {
                    enemy += g.tiles[i][j].units[p][UnitClass::Citizen as usize];
                }
            }
            let v1 = (k.value[i][j] as f32)
                * (2.5 * enemy as f32 - army as f32)
                * (army as f32).powf(0.7);
            let mut v2 = (k.value[i][j] as f32)
                * (MAX_POP as f32 - (enemy as f32 - army as f32))
                * (army as f32).powf(0.7)
                * 0.5;
            if enemy <= army {
                v2 = -10000.0;
            }
            let v = v1.max(v2);
            if fg.flag[i][j] == FLAG_ON {
                if v < 1000.0 {
                    remove_flag(g, fg, i as i32, j as i32, FLAG_POWER);
                }
            } else {
                if v > 9000.0 {
                    add_flag(g, fg, i as i32, j as i32, FLAG_POWER);
                }
            }
        }
    }
}

fn action_opportunist(k: &King, g: &Grid, fg: &mut FlagGrid) {
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            if fg.flag[i][j] == FLAG_ON {
                remove_flag(g, fg, i as i32, j as i32, FLAG_POWER);
            }
            let army = g.tiles[i][j].units[k.pl as usize][UnitClass::Citizen as usize];
            let mut enemy = 0;
            for p in 0..MAX_PLAYER {
                if p as i32 != k.pl {
                    enemy += g.tiles[i][j].units[p][UnitClass::Citizen as usize];
                }
            }
            let v = (k.value[i][j] as f32)
                * (MAX_POP as f32 - (enemy as f32 - army as f32))
                * (army as f32).powf(0.5);
            if enemy > army && v > 7000.0 {
                add_flag(g, fg, i as i32, j as i32, FLAG_POWER);
            }
        }
    }
}

fn action_noble(k: &King, g: &Grid, fg: &mut FlagGrid) {
    const LOCVAL_LEN: usize = 5;
    let mut loc: [Loc; MAX_AVLBL_LOC] = [Loc { i: -1, j: -1 }; MAX_AVLBL_LOC];
    let mut val: [i32; MAX_AVLBL_LOC] = [-1; MAX_AVLBL_LOC];
    for i in 0..LOCVAL_LEN {
        loc[i] = Loc { i: -1, j: -1 };
        val[i] = -1;
    }
    let mut insert = |lx: Loc, vx: i32| {
        let mut idx = 0usize;
        while idx < LOCVAL_LEN && idx < MAX_AVLBL_LOC && val[idx] >= vx {
            idx += 1;
        }
        if idx < LOCVAL_LEN && idx < MAX_AVLBL_LOC {
            for j in (idx + 1..LOCVAL_LEN).rev() {
                loc[j] = loc[j - 1];
                val[j] = val[j - 1];
            }
            loc[idx] = lx;
            val[idx] = vx;
        }
    };
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            if fg.flag[i][j] == FLAG_ON {
                remove_flag(g, fg, i as i32, j as i32, FLAG_POWER);
            }
            let army = g.tiles[i][j].units[k.pl as usize][UnitClass::Citizen as usize];
            let mut enemy = 0;
            for p in 0..MAX_PLAYER {
                if p as i32 != k.pl {
                    enemy += g.tiles[i][j].units[p][UnitClass::Citizen as usize];
                }
            }
            let v = (k.value[i][j] as f32)
                * (MAX_POP as f32 - (enemy as f32 - army as f32))
                * (army as f32).powf(0.5);
            if enemy > army && v > 7000.0 {
                insert(
                    Loc {
                        i: i as i32,
                        j: j as i32,
                    },
                    v as i32,
                );
            }
        }
    }
    for idx in 0..LOCVAL_LEN {
        if val[idx] > 0 {
            add_flag(g, fg, loc[idx].i, loc[idx].j, FLAG_POWER);
        }
    }
}

pub fn place_flags(k: &King, g: &Grid, fg: &mut FlagGrid) {
    match k.strategy {
        Strategy::AggrGreedy => action_aggr_greedy(k, g, fg),
        Strategy::OneGreedy => action_one_greedy(k, g, fg),
        Strategy::PersistentGreedy => action_persistent_greedy(k, g, fg),
        Strategy::Opportunist => action_opportunist(k, g, fg),
        Strategy::Noble => action_noble(k, g, fg),
        Strategy::Midas => {}
        Strategy::None => {}
    }
}
