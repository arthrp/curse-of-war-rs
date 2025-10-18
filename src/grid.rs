use rand::Rng;

use crate::common::*;

#[derive(Copy, Clone, Debug)]
pub struct Tile {
    pub cl: TileClass,
    pub pl: i32,
    pub units: [[i32; MAX_CLASS]; MAX_PLAYER],
}

#[derive(Clone, Debug)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
    pub tiles: [[Tile; MAX_HEIGHT]; MAX_WIDTH],
}

#[derive(Clone, Debug, Copy)]
pub struct FlagGrid {
    pub width: i32,
    pub height: i32,
    pub flag: [[i32; MAX_HEIGHT]; MAX_WIDTH],
    pub call: [[i32; MAX_HEIGHT]; MAX_WIDTH],
}

#[inline]
pub fn is_a_city(t: TileClass) -> bool {
    matches!(t, TileClass::Village | TileClass::Town | TileClass::Castle)
}

#[inline]
pub fn is_inhabitable(t: TileClass) -> bool {
    !matches!(t, TileClass::Abyss | TileClass::Mountain | TileClass::Mine)
}

#[inline]
pub fn is_visible(t: TileClass) -> bool {
    !matches!(t, TileClass::Abyss)
}

pub fn stencil_avlbl_loc_num(st: Stencil) -> i32 {
    match st {
        Stencil::Rhombus => 4,
        Stencil::Rect => 4,
        Stencil::Hex => 6,
    }
}

pub fn grid_init(g: &mut Grid, w: i32, h: i32) {
    g.width = min_i32(w, MAX_WIDTH as i32);
    g.height = min_i32(h, MAX_HEIGHT as i32);
    let mut rng = rand::thread_rng();
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            let mut cl = TileClass::Grassland;
            let x = rand::Rng::gen_range(&mut rng, 0..20);
            if x == 0 {
                let y = rand::Rng::gen_range(&mut rng, 0..6);
                cl = match y {
                    0 => TileClass::Castle,
                    1 | 2 => TileClass::Town,
                    _ => TileClass::Village,
                };
            }
            if x > 0 && x < 5 {
                if rand::Rng::gen_range(&mut rng, 0..10) == 0 {
                    cl = TileClass::Mine;
                } else {
                    cl = TileClass::Mountain;
                }
            }

            let mut pl = NEUTRAL as i32;
            if !matches!(cl, TileClass::Mountain | TileClass::Mine) {
                let x2 = 1 + rand::Rng::gen_range(&mut rng, 0..(MAX_PLAYER as i32 - 1));
                if x2 < MAX_PLAYER as i32 {
                    pl = x2;
                } else {
                    pl = NEUTRAL as i32;
                }
            }

            let mut tile = Tile { cl, pl, units: [[0; MAX_CLASS]; MAX_PLAYER] };
            if is_a_city(cl) {
                let owner = tile.pl as usize;
                tile.units[owner][UnitClass::Citizen as usize] = 10;
            }
            g.tiles[i][j] = tile;
        }
    }
}

fn x_of_ij(i: i32, j: i32) -> f32 { 0.5 * (j as f32) + (i as f32) }
fn y_of_ij(_i: i32, j: i32) -> f32 { j as f32 }

fn stencil_rhombus(g: &mut Grid, d: i32, loc: &mut [Loc; MAX_AVLBL_LOC]) {
    let xs = [d, g.width - 1 - d, d, g.width - 1 - d];
    let ys = [d, g.height - 1 - d, g.height - 1 - d, d];
    for k in 0..4 {
        loc[k] = Loc { i: xs[k], j: ys[k] };
    }
}

fn stencil_rect(g: &mut Grid, d: i32, loc: &mut [Loc; MAX_AVLBL_LOC]) {
    let epsilon = 0.1f32;
    let x0 = x_of_ij(0, g.height - 1) - epsilon;
    let y0 = y_of_ij(0, 0) - epsilon;
    let x1 = x_of_ij(g.width - 1, 0) + epsilon;
    let y1 = y_of_ij(0, g.height - 1) + epsilon;
    for i in 0..g.width {
        for j in 0..g.height {
            let x = x_of_ij(i, j);
            let y = y_of_ij(i, j);
            if x < x0 || x > x1 || y < y0 || y > y1 {
                g.tiles[i as usize][j as usize].cl = TileClass::Abyss;
            }
        }
    }
    let loc_num = 4;
    let dx = g.height / 2;
    let temp_loc = [
        Loc { i: dx + d - 1, j: d },
        Loc { i: g.width - dx - 1 - d + 1, j: g.height - 1 - d },
        Loc { i: d + 1, j: g.height - 1 - d },
        Loc { i: g.width - 1 - d - 1, j: d },
    ];
    for k in 0..loc_num { loc[k] = temp_loc[k]; }
}

fn stencil_hex(g: &mut Grid, d: i32, loc: &mut [Loc; MAX_AVLBL_LOC]) {
    let dx = g.height / 2;
    for i in 0..g.width {
        for j in 0..g.height {
            if i + j < dx || i + j > g.width - 1 + g.height - 1 - dx {
                g.tiles[i as usize][j as usize].cl = TileClass::Abyss;
            }
        }
    }
    let loc_num = 6;
    let temp_loc = [
        Loc { i: dx + d - 2, j: d },
        Loc { i: d, j: g.height - 1 - d },
        Loc { i: g.width - 1 - d, j: dx },
        Loc { i: d, j: dx },
        Loc { i: g.width - 1 - d - 2 + 2, j: d },
        Loc { i: g.width - 1 - dx - d + 2, j: g.height - 1 - d },
    ];
    for k in 0..loc_num { loc[k] = temp_loc[k]; }
}

pub fn apply_stencil(st: Stencil, g: &mut Grid, d: i32, loc: &mut [Loc; MAX_AVLBL_LOC], avlbl_loc_num: &mut i32) {
    *avlbl_loc_num = stencil_avlbl_loc_num(st);
    match st {
        Stencil::Rhombus => stencil_rhombus(g, d, loc),
        Stencil::Rect => stencil_rect(g, d, loc),
        Stencil::Hex => stencil_hex(g, d, loc),
    }
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            if g.tiles[i][j].cl == TileClass::Abyss {
                for p in 0..MAX_PLAYER {
                    g.tiles[i][j].units[p][UnitClass::Citizen as usize] = 0;
                    g.tiles[i][j].pl = NEUTRAL as i32;
                }
            }
        }
    }
}

// Helpers for conflict evaluation
fn floodfill_closest(g: &Grid, u: &mut [[i32; MAX_HEIGHT]; MAX_WIDTH], d: &mut [[i32; MAX_HEIGHT]; MAX_WIDTH], x: i32, y: i32, val: i32, dist: i32) {
    if x < 0 || x >= g.width || y < 0 || y >= g.height || !is_inhabitable(g.tiles[x as usize][y as usize].cl) || d[x as usize][y as usize] <= dist { return; }
    u[x as usize][y as usize] = val; d[x as usize][y as usize] = dist;
    for k in 0..DIRECTIONS { floodfill_closest(g, u, d, x + DIRS[k].i, y + DIRS[k].j, val, dist + 1); }
}

fn eval_locations(g: &Grid, loc: &[Loc], result: &mut [i32], len: usize) {
    let mut u = [[0i32; MAX_HEIGHT]; MAX_WIDTH];
    let mut d = [[(MAX_WIDTH * MAX_HEIGHT + 1) as i32; MAX_HEIGHT]; MAX_WIDTH];
    for i in 0..g.width as usize { for j in 0..g.height as usize { d[i][j] = (MAX_WIDTH * MAX_HEIGHT + 1) as i32; u[i][j] = -1; } }
    for k in 0..len { floodfill_closest(g, &mut u, &mut d, loc[k].i, loc[k].j, k as i32, 0); }
    for i in 0..g.width as usize { for j in 0..g.height as usize {
        if g.tiles[i][j].cl == TileClass::Mine {
            let mut single_owner = -1; let mut max_dist = 0; let mut min_dist = (MAX_WIDTH * MAX_HEIGHT + 1) as i32;
            for k in 0..DIRECTIONS { let x = i as i32 + DIRS[k].i; let y = j as i32 + DIRS[k].j; if x<0 || x>=g.width || y<0 || y>=g.height || !is_inhabitable(g.tiles[x as usize][y as usize].cl) { continue; }
                if single_owner == -1 { single_owner = u[x as usize][y as usize]; max_dist = d[x as usize][y as usize]; min_dist = d[x as usize][y as usize]; }
                else {
                    if u[x as usize][y as usize] == single_owner { max_dist = max_i32(max_dist, d[x as usize][y as usize]); min_dist = min_i32(min_dist, d[x as usize][y as usize]); }
                    else if u[x as usize][y as usize] != -1 { single_owner = -2; }
                }
            }
            if single_owner != -2 && single_owner != -1 {
                result[single_owner as usize] += (100.0 * (MAX_WIDTH + MAX_HEIGHT) as f32 * (-10.0 * (max_dist as f32) * (min_dist as f32) / ((MAX_WIDTH * MAX_HEIGHT) as f32)).exp()) as i32;
            }
        }
    }}
}

fn shuffle(arr: &mut [i32]) { let mut rng = rand::thread_rng(); for _ in 0..arr.len() { let i = rand::Rng::gen_range(&mut rng, 0..arr.len()); let j = rand::Rng::gen_range(&mut rng, 0..arr.len()); arr.swap(i,j); } }

fn sort_increasing(val: &mut [i32], item: &mut [i32], len: usize) {
    for i in 0..(len.saturating_sub(1)) { let mut k = i; for j in (i+1)..len { if val[j] < val[k] { k = j; } } val.swap(i,k); item.swap(i,k); }
}

pub fn conflict(g: &mut Grid, loc_arr: &[Loc], available_loc_num: i32, players: &[i32], players_num: i32, locations_num: i32, ui_players: &[i32], ui_players_num: i32, conditions: i32, ineq: i32) -> i32 {
    // remove all cities and reset
    for i in 0..g.width as usize { for j in 0..g.height as usize { for p in 0..MAX_PLAYER { for c in 0..MAX_CLASS { g.tiles[i][j].units[p][c] = 0; g.tiles[i][j].pl = NEUTRAL as i32; if is_a_city(g.tiles[i][j].cl) { g.tiles[i][j].cl = TileClass::Grassland; } } } } }
    let locations_num = in_segment(locations_num, 2, available_loc_num) as usize;
    let num = min_i32(locations_num as i32, (players_num + ui_players_num) as i32) as usize;
    let di = rand::Rng::gen_range(&mut rand::thread_rng(), 0..available_loc_num.max(1) as usize);
    let mut chosen_loc = [Loc{ i: 0, j: 0}; MAX_AVLBL_LOC];
    let mut i = 0usize; while i < num { let ii = (i + di) % (available_loc_num as usize); let x = loc_arr[ii].i; let y = loc_arr[ii].j; chosen_loc[i] = Loc { i: x, j: y }; g.tiles[x as usize][y as usize].cl = TileClass::Castle; let dir = rand::Rng::gen_range(&mut rand::thread_rng(), 0..DIRECTIONS); let ri = DIRS[dir].i; let rj = DIRS[dir].j; let m = 1; let mut mine_i = x + m*ri; let mut mine_j = y + m*rj; g.tiles[mine_i as usize][mine_j as usize].cl = TileClass::Mine; g.tiles[mine_i as usize][mine_j as usize].pl = NEUTRAL as i32; mine_i = x - 2*m*ri; mine_j = y - 2*m*rj; g.tiles[mine_i as usize][mine_j as usize].cl = TileClass::Mine; g.tiles[mine_i as usize][mine_j as usize].pl = NEUTRAL as i32; mine_i = x - m*ri; mine_j = y - m*rj; g.tiles[mine_i as usize][mine_j as usize].cl = TileClass::Grassland; g.tiles[mine_i as usize][mine_j as usize].pl = NEUTRAL as i32; i += 1; }
    let mut eval_result = [0i32; MAX_AVLBL_LOC]; let mut loc_index = [0i32,1,2,3,4,5,6]; eval_locations(g, &chosen_loc, &mut eval_result, num); sort_increasing(&mut eval_result, &mut loc_index, num);
    if ineq != RANDOM_INEQUALITY {
        let mut avg = 0f32; for i in 0..num { avg += eval_result[i] as f32; } avg /= num as f32; let mut var = 0f32; for i in 0..num { var += (eval_result[i] as f32 - avg).powi(2); } var /= num as f32; let diff = var.sqrt(); let x = diff * 1000.0 / avg; let ok = match ineq { 0 => x <= 50.0, 1 => x > 50.0 && x <= 100.0, 2 => x > 100.0 && x <= 250.0, 3 => x > 250.0 && x <= 500.0, 4 => x > 500.0, _ => true }; if !ok { return -1; }
    }
    let mut sh_players_comp = players.to_vec(); shuffle(&mut sh_players_comp);
    let mut sh_players = vec![0i32; num]; let mut idx = 0usize; while idx < ui_players_num as usize { sh_players[idx] = ui_players[idx]; idx += 1; }
    let dplayer = rand::Rng::gen_range(&mut rand::thread_rng(), 0..players_num.max(1) as usize);
    while idx < num { sh_players[idx] = sh_players_comp[(idx - ui_players_num as usize + dplayer) % (players_num as usize)]; idx += 1; }
    shuffle(&mut sh_players);
    let mut ihuman = rand::Rng::gen_range(&mut rand::thread_rng(), 0..num);
    if conditions > 0 { let select = in_segment(num as i32 - conditions, 0, num as i32 - 1) as usize; ihuman = loc_index[select] as usize; }
    for i in 0..num { let ii = loc_index[i] as usize; let x = chosen_loc[ii].i as usize; let y = chosen_loc[ii].j as usize; if ui_players_num > 1 { g.tiles[x][y].pl = sh_players[i]; } else { if ii == ihuman { g.tiles[x][y].pl = ui_players[0]; } else { g.tiles[x][y].pl = sh_players_comp[i]; } } g.tiles[x][y].units[g.tiles[x][y].pl as usize][UnitClass::Citizen as usize] = 10; }
    0
}

fn floodfill(g: &Grid, u: &mut [[i32; MAX_HEIGHT]; MAX_WIDTH], x: i32, y: i32, val: i32) {
    if x < 0 || x >= g.width || y < 0 || y >= g.height || !is_inhabitable(g.tiles[x as usize][y as usize].cl) || u[x as usize][y as usize] == val { return; }
    u[x as usize][y as usize] = val; for k in 0..DIRECTIONS { floodfill(g, u, x + DIRS[k].i, y + DIRS[k].j, val); }
}

pub fn is_connected(g: &Grid) -> bool {
    let mut m = [[0i32; MAX_HEIGHT]; MAX_WIDTH]; let mut colored = false;
    for i in 0..g.width as usize { for j in 0..g.height as usize { if g.tiles[i][j].pl != NEUTRAL as i32 { if colored && m[i][j] == 0 { return false; } colored = true; floodfill(g, &mut m, i as i32, j as i32, 1); } } }
    true
}

pub fn flag_grid_init(fg: &mut FlagGrid, w: i32, h: i32) {
    fg.width = min_i32(w, MAX_WIDTH as i32);
    fg.height = min_i32(h, MAX_HEIGHT as i32);
    for i in 0..fg.width as usize {
        for j in 0..fg.height as usize {
            fg.flag[i][j] = FLAG_OFF;
            fg.call[i][j] = 0;
        }
    }
}

pub fn spread(g: &Grid, u: &mut [[i32; MAX_HEIGHT]; MAX_WIDTH], v: &mut [[i32; MAX_HEIGHT]; MAX_WIDTH], x: i32, y: i32, val: i32, factor: i32) {
    if x < 0 || x >= g.width || y < 0 || y >= g.height || !is_inhabitable(g.tiles[x as usize][y as usize].cl) { return; }
    let d = val - u[x as usize][y as usize];
    if d > 0 {
        v[x as usize][y as usize] = max_i32(0, v[x as usize][y as usize] + d * factor);
        u[x as usize][y as usize] += d;
        for k in 0..DIRECTIONS {
            let nx = x + DIRS[k].i;
            let ny = y + DIRS[k].j;
            spread(g, u, v, nx, ny, val / 2, factor);
        }
    }
}

pub fn even(g: &Grid, v: &mut [[i32; MAX_HEIGHT]; MAX_WIDTH], x: i32, y: i32, val: i32) {
    if x < 0 || x >= g.width || y < 0 || y >= g.height || v[x as usize][y as usize] == val { return; }
    v[x as usize][y as usize] = val;
    for k in 0..DIRECTIONS {
        let nx = x + DIRS[k].i;
        let ny = y + DIRS[k].j;
        even(g, v, nx, ny, val);
    }
}

pub fn add_flag(g: &Grid, fg: &mut FlagGrid, x: i32, y: i32, val: i32) {
    if x < 0 || x >= g.width || y < 0 || y >= g.height || !is_inhabitable(g.tiles[x as usize][y as usize].cl) || fg.flag[x as usize][y as usize] == FLAG_ON { return; }
    let mut u = [[0i32; MAX_HEIGHT]; MAX_WIDTH];
    fg.flag[x as usize][y as usize] = FLAG_ON;
    spread(g, &mut u, &mut fg.call, x, y, val, 1);
}

pub fn remove_flag(g: &Grid, fg: &mut FlagGrid, x: i32, y: i32, val: i32) {
    if x < 0 || x >= g.width || y < 0 || y >= g.height || !is_inhabitable(g.tiles[x as usize][y as usize].cl) || fg.flag[x as usize][y as usize] == FLAG_OFF { return; }
    let mut u = [[0i32; MAX_HEIGHT]; MAX_WIDTH];
    fg.flag[x as usize][y as usize] = FLAG_OFF;
    spread(g, &mut u, &mut fg.call, x, y, val, -1);
}

pub fn remove_flags_with_prob(g: &Grid, fg: &mut FlagGrid, prob: f32) {
    let mut rng = rand::thread_rng();
    for i in 0..g.width as usize {
        for j in 0..g.height as usize {
            if fg.flag[i][j] == FLAG_ON && rand::Rng::r#gen::<f32>(&mut rng) <= prob {
                remove_flag(g, fg, i as i32, j as i32, FLAG_POWER);
            }
        }
    }
}


