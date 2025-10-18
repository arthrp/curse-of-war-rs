use rand::Rng;

use crate::common::*;
use crate::grid::*;
use crate::king::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct UI {
    pub cursor: Loc,
    pub xskip: i32,
    pub xlength: i32,
}

#[derive(Clone, Debug)]
pub struct Timeline {
    pub data: [[f32; MAX_TIMELINE_MARK]; MAX_PLAYER],
    pub time: [u64; MAX_TIMELINE_MARK],
    pub mark: i32,
}

#[derive(Clone, Debug)]
pub struct BasicOptions {
    pub keep_random_flag: i32,
    pub dif: ConfigDif,
    pub speed: ConfigSpeed,
    pub w: i32,
    pub h: i32,
    pub loc_num: i32,
    pub map_seed: u32,
    pub conditions: i32,
    pub timeline_flag: i32,
    pub inequality: i32,
    pub shape: Stencil,
}

#[derive(Clone, Debug)]
pub struct State {
    pub grid: Grid,
    pub fg: [FlagGrid; MAX_PLAYER],
    pub king: [King; MAX_PLAYER],
    pub kings_num: i32,
    pub timeline: Timeline,
    pub show_timeline: i32,
    pub country: [Country; MAX_PLAYER],
    pub time: u64,
    pub map_seed: i32,
    pub controlled: i32,
    pub conditions: i32,
    pub inequality: i32,
    pub speed: ConfigSpeed,
    pub prev_speed: ConfigSpeed,
    pub dif: ConfigDif,
}

pub fn faster(sp: ConfigSpeed) -> ConfigSpeed {
    match sp {
        ConfigSpeed::Pause => ConfigSpeed::Slowest,
        ConfigSpeed::Slowest => ConfigSpeed::Slower,
        ConfigSpeed::Slower => ConfigSpeed::Slow,
        ConfigSpeed::Slow => ConfigSpeed::Normal,
        ConfigSpeed::Normal => ConfigSpeed::Fast,
        ConfigSpeed::Fast => ConfigSpeed::Faster,
        _ => ConfigSpeed::Fastest,
    }
}

pub fn slower(sp: ConfigSpeed) -> ConfigSpeed {
    match sp {
        ConfigSpeed::Fastest => ConfigSpeed::Faster,
        ConfigSpeed::Faster => ConfigSpeed::Fast,
        ConfigSpeed::Fast => ConfigSpeed::Normal,
        ConfigSpeed::Normal => ConfigSpeed::Slow,
        ConfigSpeed::Slow => ConfigSpeed::Slower,
        ConfigSpeed::Slower => ConfigSpeed::Slowest,
        _ => ConfigSpeed::Pause,
    }
}

pub fn game_slowdown(speed: ConfigSpeed) -> i32 {
    match speed {
        ConfigSpeed::Pause => 1,
        ConfigSpeed::Slowest => 160,
        ConfigSpeed::Slower => 80,
        ConfigSpeed::Slow => 40,
        ConfigSpeed::Normal => 20,
        ConfigSpeed::Fast => 10,
        ConfigSpeed::Faster => 5,
        ConfigSpeed::Fastest => 2,
    }
}

pub fn win_or_lose(st: &State) -> i32 {
    let mut pop = [0i32; MAX_PLAYER];
    for i in 0..st.grid.width as usize {
        for j in 0..st.grid.height as usize {
            if is_inhabitable(st.grid.tiles[i][j].cl) {
                for p in 0..MAX_PLAYER {
                    pop[p] += st.grid.tiles[i][j].units[p][UnitClass::Citizen as usize];
                }
            }
        }
    }
    let mut win = 1;
    let mut lose = 0;
    let mut best = 0usize;
    for p in 0..MAX_PLAYER {
        if pop[best] < pop[p] {
            best = p;
        }
        if p as i32 != st.controlled && pop[p] > 0 {
            win = 0;
        }
    }
    if pop[st.controlled as usize] == 0 {
        lose = 1;
    }
    if win == 1 {
        1
    } else if lose == 1 {
        -1
    } else {
        0
    }
}

pub fn state_init(s: &mut State, op: &BasicOptions, clients_num: i32) {
    s.speed = op.speed;
    s.prev_speed = s.speed;
    s.dif = op.dif;
    s.map_seed = op.map_seed as i32;
    s.conditions = op.conditions;
    s.inequality = op.inequality;
    let mut rng = rand::thread_rng();
    s.time = ((1850 + rng.gen_range(0..100)) * 360 + rng.gen_range(0..360)) as u64;
    s.controlled = 1;

    let all_players = [1, 2, 3, 4, 5, 6, 7];
    let comp_players_num = 7 - clients_num;
    s.kings_num = comp_players_num;
    let mut ui_players_num = clients_num;
    let mut g_init = Grid {
        width: 0,
        height: 0,
        tiles: [[Tile {
            cl: TileClass::Grassland,
            pl: NEUTRAL as i32,
            units: [[0; MAX_CLASS]; MAX_PLAYER],
        }; MAX_HEIGHT]; MAX_WIDTH],
    };
    grid_init(&mut g_init, op.w, op.h);

    let mut comp_idx = 0;
    for i in 0..7 {
        if (i as i32) < clients_num { /* UI player, do nothing here */
        } else {
            let pl = all_players[i];
            s.king[comp_idx] = King {
                value: [[0; MAX_HEIGHT]; MAX_WIDTH],
                pl,
                strategy: Strategy::Opportunist,
            };
            comp_idx += 1;
        }
    }

    s.grid = g_init;
    // stencils / conflict
    let mut loc_arr = [Loc { i: 0, j: 0 }; MAX_AVLBL_LOC];
    let mut available_loc_num = 0i32;
    apply_stencil(
        op.shape,
        &mut s.grid,
        2,
        &mut loc_arr,
        &mut available_loc_num,
    );
    // Apply conflict to set starting positions
    let all = [1, 2, 3, 4, 5, 6, 7];
    let ui_players: [i32; 1] = [1];
    let players: [i32; 6] = [2, 3, 4, 5, 6, 7];
    let _ = crate::grid::conflict(
        &mut s.grid,
        &loc_arr,
        available_loc_num,
        &players,
        players.len() as i32,
        op.loc_num,
        &ui_players,
        1,
        op.conditions,
        op.inequality,
    );
    while !is_connected(&s.grid) {
        grid_init(&mut s.grid, op.w, op.h);
        apply_stencil(
            op.shape,
            &mut s.grid,
            2,
            &mut loc_arr,
            &mut available_loc_num,
        );
        let _ = crate::grid::conflict(
            &mut s.grid,
            &loc_arr,
            available_loc_num,
            &players,
            players.len() as i32,
            op.loc_num,
            &ui_players,
            1,
            op.conditions,
            op.inequality,
        );
    }

    for p in 0..MAX_PLAYER {
        s.fg[p] = FlagGrid {
            width: s.grid.width,
            height: s.grid.height,
            flag: [[FLAG_OFF; MAX_HEIGHT]; MAX_WIDTH],
            call: [[0; MAX_HEIGHT]; MAX_WIDTH],
        };
        s.country[p] = Country { gold: 0 };
    }
    for i in 0..s.kings_num as usize {
        king_evaluate_map(&mut s.king[i], &s.grid, s.dif);
    }

    s.show_timeline = op.timeline_flag;
    s.timeline = Timeline {
        data: [[0.0; MAX_TIMELINE_MARK]; MAX_PLAYER],
        time: [s.time; MAX_TIMELINE_MARK],
        mark: -1,
    };
}

pub fn ui_init(s: &State, ui: &mut UI) {
    ui.cursor = Loc {
        i: s.grid.width / 2,
        j: s.grid.height / 2,
    };
    for i in 0..s.grid.width as usize {
        for j in 0..s.grid.height as usize {
            if s.grid.tiles[i][j].units[s.controlled as usize][UnitClass::Citizen as usize]
                > s.grid.tiles[ui.cursor.i as usize][ui.cursor.j as usize].units
                    [s.controlled as usize][UnitClass::Citizen as usize]
            {
                ui.cursor.i = i as i32;
                ui.cursor.j = j as i32;
            }
        }
    }
    let mut xskip_x2 = (MAX_WIDTH as i32) * 2 + 1;
    let mut xrightmost_x2 = 0i32;
    for i in 0..s.grid.width as usize {
        for j in 0..s.grid.height as usize {
            if is_visible(s.grid.tiles[i][j].cl) {
                let x = (i as i32) * 2 + (j as i32);
                if xskip_x2 > x {
                    xskip_x2 = x;
                }
                if xrightmost_x2 < x {
                    xrightmost_x2 = x;
                }
            }
        }
    }
    ui.xskip = xskip_x2 / 2;
    ui.xlength = (xrightmost_x2 + 1) / 2 - xskip_x2 / 2;
}

pub fn adjust_cursor(s: &State, ui: &mut UI, mut cursi: i32, mut cursj: i32) {
    cursi = in_segment(cursi, 0, s.grid.width - 1);
    cursj = in_segment(cursj, 0, s.grid.height - 1);
    if is_visible(s.grid.tiles[cursi as usize][cursj as usize].cl) {
        ui.cursor.i = cursi;
        ui.cursor.j = cursj;
    } else if is_visible(s.grid.tiles[ui.cursor.i as usize][cursj as usize].cl) {
        ui.cursor.j = cursj;
    } else {
        let mut i = in_segment(cursi - 1, 0, s.grid.width - 1);
        if is_visible(s.grid.tiles[i as usize][cursj as usize].cl) {
            ui.cursor.i = i;
            ui.cursor.j = cursj;
        } else {
            i = in_segment(cursi + 1, 0, s.grid.width - 1);
            if is_visible(s.grid.tiles[i as usize][cursj as usize].cl) {
                ui.cursor.i = i;
                ui.cursor.j = cursj;
            }
        }
    }
}

fn growth(t: TileClass) -> f32 {
    match t {
        TileClass::Village => 1.10,
        TileClass::Town => 1.20,
        TileClass::Castle => 1.30,
        _ => 0.0,
    }
}

fn rnd_round(x: f32) -> i32 {
    let i = x.floor() as i32;
    let mut rng = rand::thread_rng();
    if rand::Rng::r#gen::<f32>(&mut rng) < (x - i as f32) {
        i + 1
    } else {
        i
    }
}

pub fn kings_move(s: &mut State) {
    let mut ev = false;
    for i in 0..s.kings_num as usize {
        let pl = s.king[i].pl;
        place_flags(&s.king[i], &s.grid, &mut s.fg[pl as usize]);
        let code = builder_default(
            &s.king[i],
            &mut s.country[pl as usize],
            &mut s.grid,
            &mut s.fg[pl as usize],
        );
        ev = ev || (code == 0);
    }
    if ev {
        for i in 0..s.kings_num as usize {
            king_evaluate_map(&mut s.king[i], &s.grid, s.dif);
        }
    }
}

pub fn simulate(s: &mut State) {
    s.time += 1;
    let t_ptr: *mut [[Tile; MAX_HEIGHT]; MAX_WIDTH] = &mut s.grid.tiles;
    let t = unsafe { &mut *t_ptr };
    let mut enemy_pop = [0i32; MAX_PLAYER];
    let mut my_pop = [0i32; MAX_PLAYER];
    let mut need_to_reeval = false;
    for i in 0..s.grid.width as usize {
        for j in 0..s.grid.height as usize {
            if t[i][j].cl == TileClass::Mine {
                let mut owner = NEUTRAL as i32;
                // let mut max_dist = 0;
                // let mut min_dist = (MAX_WIDTH * MAX_HEIGHT) as i32 + 1;
                for k in 0..DIRECTIONS {
                    let di = DIRS[k].i;
                    let dj = DIRS[k].j;
                    let ni = i as i32 + di;
                    let nj = j as i32 + dj;
                    if ni >= 0
                        && ni < s.grid.width
                        && nj >= 0
                        && nj < s.grid.height
                        && is_inhabitable(t[ni as usize][nj as usize].cl)
                    {
                        let pl = t[ni as usize][nj as usize].pl;
                        if owner == NEUTRAL as i32 {
                            owner = pl;
                        } else if owner != pl && pl != NEUTRAL as i32 {
                            owner = -1;
                        }
                    }
                }
                if owner != -1 {
                    t[i][j].pl = owner;
                } else {
                    t[i][j].pl = NEUTRAL as i32;
                }
                if t[i][j].pl != NEUTRAL as i32 {
                    s.country[owner as usize].gold += 1;
                }
            }
            let mut total_pop = 0;
            for p in 0..MAX_PLAYER {
                my_pop[p] = t[i][j].units[p][UnitClass::Citizen as usize];
                total_pop += my_pop[p];
            }
            let mut defender_dmg = 0;
            for p in 0..MAX_PLAYER {
                enemy_pop[p] = total_pop - my_pop[p];
                let mut dmg = 0;
                if total_pop != 0 {
                    dmg =
                        rnd_round((enemy_pop[p] as f32) * (my_pop[p] as f32) / (total_pop as f32));
                }
                t[i][j].units[p][UnitClass::Citizen as usize] = max_i32(my_pop[p] - dmg, 0);
                if t[i][j].pl == p as i32 {
                    defender_dmg = dmg;
                }
            }
            if defender_dmg > (2.0 * MAX_POP as f32 * 0.1) as i32
                && matches!(
                    t[i][j].cl,
                    TileClass::Village | TileClass::Town | TileClass::Castle
                )
            {
                need_to_reeval = true;
                let _ = degrade(&mut s.grid, i as i32, j as i32);
            }
            if is_inhabitable(t[i][j].cl) {
                t[i][j].pl = NEUTRAL as i32;
                for p in 0..MAX_PLAYER {
                    if t[i][j].units[p][UnitClass::Citizen as usize]
                        > t[i][j].units[t[i][j].pl as usize][UnitClass::Citizen as usize]
                    {
                        t[i][j].pl = p as i32;
                    }
                }
            }
            if matches!(
                t[i][j].cl,
                TileClass::Village | TileClass::Town | TileClass::Castle
            ) {
                let owner = t[i][j].pl as usize;
                let pop = t[i][j].units[owner][UnitClass::Citizen as usize];
                let fnpop = (pop as f32) * growth(t[i][j].cl);
                let mut npop = rnd_round(fnpop);
                npop = min_i32(npop, MAX_POP);
                t[i][j].units[owner][UnitClass::Citizen as usize] = npop;
            }
        }
    }
    let mut rng = rand::thread_rng();
    let (i_start, i_end, i_inc) = if rand::Rng::r#gen::<bool>(&mut rng) {
        (0, s.grid.width, 1)
    } else {
        (s.grid.width - 1, -1, -1)
    };
    let (j_start, j_end, j_inc) = if rand::Rng::r#gen::<bool>(&mut rng) {
        (0, s.grid.height, 1)
    } else {
        (s.grid.height - 1, -1, -1)
    };
    let mut i = i_start;
    while i != i_end {
        let mut j = j_start;
        while j != j_end {
            for p in 0..MAX_PLAYER {
                let initial_pop = t[i as usize][j as usize].units[p][UnitClass::Citizen as usize];
                let k_shift = rand::Rng::gen_range(&mut rng, 0..DIRECTIONS as i32);
                for kk in 0..DIRECTIONS as i32 {
                    let k = ((kk + k_shift) % DIRECTIONS as i32) as usize;
                    let di = DIRS[k].i;
                    let dj = DIRS[k].j;
                    let ni = i + di;
                    let nj = j + dj;
                    if ni >= 0
                        && ni < s.grid.width
                        && nj >= 0
                        && nj < s.grid.height
                        && is_inhabitable(t[ni as usize][nj as usize].cl)
                    {
                        let pop = t[i as usize][j as usize].units[p][UnitClass::Citizen as usize];
                        let dcall = max_i32(
                            0,
                            s.fg[p].call[ni as usize][nj as usize]
                                - s.fg[p].call[i as usize][j as usize],
                        );
                        if pop > 0 {
                            let mut dpop = rnd_round(
                                0.05 * initial_pop as f32
                                    + 0.10 * dcall as f32 * initial_pop as f32,
                            );
                            dpop = min_i32(dpop, pop);
                            dpop = min_i32(
                                dpop,
                                MAX_POP
                                    - t[ni as usize][nj as usize].units[p]
                                        [UnitClass::Citizen as usize],
                            );
                            t[ni as usize][nj as usize].units[p][UnitClass::Citizen as usize] +=
                                dpop;
                            t[i as usize][j as usize].units[p][UnitClass::Citizen as usize] -= dpop;
                        }
                    }
                }
            }
            j += j_inc;
        }
        i += i_inc;
    }
    for i in 0..s.grid.width as usize {
        for j in 0..s.grid.height as usize {
            if is_inhabitable(t[i][j].cl) {
                t[i][j].pl = NEUTRAL as i32;
                for p in 0..MAX_PLAYER {
                    if t[i][j].units[p][UnitClass::Citizen as usize]
                        > t[i][j].units[t[i][j].pl as usize][UnitClass::Citizen as usize]
                    {
                        t[i][j].pl = p as i32;
                    }
                }
            }
        }
    }
    if need_to_reeval {
        for i in 0..s.kings_num as usize {
            king_evaluate_map(&mut s.king[i], &s.grid, s.dif);
        }
    }
    let add_gold = match s.dif {
        ConfigDif::Hard => 1,
        ConfigDif::Hardest => 2,
        _ => 0,
    };
    for i in 0..MAX_PLAYER {
        if i as i32 != NEUTRAL as i32 && i as i32 != s.controlled && s.country[i].gold > 0 {
            s.country[i].gold += add_gold;
        }
    }
}

pub fn update_timeline(s: &mut State) {
    if (s.timeline.mark + 1) < MAX_TIMELINE_MARK as i32 {
        s.timeline.mark += 1;
    } else {
        for i in 0..(MAX_TIMELINE_MARK - 1) {
            s.timeline.time[i] = s.timeline.time[i + 1];
            for p in 0..MAX_PLAYER {
                s.timeline.data[p][i] = s.timeline.data[p][i + 1];
            }
        }
    }
    let m = s.timeline.mark as usize;
    s.timeline.time[m] = s.time;
    for p in 0..MAX_PLAYER {
        let mut count = 0;
        for i in 0..MAX_WIDTH {
            for j in 0..MAX_HEIGHT {
                count += s.grid.tiles[i][j].units[p][UnitClass::Citizen as usize];
            }
        }
        s.timeline.data[p][m] = count as f32;
    }
}
