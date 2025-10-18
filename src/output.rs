use ncurses::*;

use crate::common::*;
use crate::grid::*;
use crate::state::*;

const CELL_STR_LEN: i32 = 3;

fn player_color(p: i32) -> i16 {
    match p {
        0 => 6,
        1 => 4,
        2 => 5,
        3 => 3,
        4 => 6,
        5 => 7,
        6 => 8,
        7 => 2,
        _ => -1,
    }
}

fn player_style(p: i32) -> attr_t {
    if p != NEUTRAL as i32 { A_BOLD() | COLOR_PAIR(player_color(p)) as u32 } else { A_NORMAL() | COLOR_PAIR(player_color(p)) as u32 }
}

fn pop_to_symbol(num: i32) -> i32 {
    if num > 400 { 8 }
    else if num > 200 { 7 }
    else if num > 100 { 6 }
    else if num > 50 { 5 }
    else if num > 25 { 4 }
    else if num > 12 { 3 }
    else if num > 6 { 2 }
    else if num > 3 { 1 }
    else if num > 0 { 0 } else { -1 }
}

fn time_to_ymd(time: u64) -> (i32, i32, i32) {
    let year = (time / 360) as i32;
    let mut month = (time - (year as u64)*360) as i32;
    let day = month % 30 + 1;
    month = month / 30 + 1;
    (year, month, day)
}

#[inline]
fn posy(_ui: &UI, _i: i32, j: i32) -> i32 { j + 1 }
#[inline]
fn posx(ui: &UI, i: i32, j: i32) -> i32 { (i*4 + j*2 + 1) - (ui.xskip*(CELL_STR_LEN+1)) }

fn output_units(ui: &UI, t: &Tile, i: i32, j: i32) {
    let mut num = 0; for p in 0..MAX_PLAYER { num += t.units[p][UnitClass::Citizen as usize]; }
    mv(posy(ui, i, j), posx(ui, i, j));
    match pop_to_symbol(num) {
        8 => { addstr(":::"); }
        7 => { addstr(".::"); }
        6 => { addstr(" ::"); }
        5 => { addstr(".:."); }
        4 => { addstr(".: "); }
        3 => { addstr(" : "); }
        2 => { addstr("..."); }
        1 => { addstr(".. "); }
        0 => { addstr(" . "); }
        _ => {}
    }
}

fn output_key(y: i32, x: i32, key: &str, key_style: attr_t, s: &str, s_style: attr_t) {
    attrset((key_style as i32).try_into().unwrap());
    mvaddstr(y, x+1, key);
    attrset((s_style as i32).try_into().unwrap());
    mvaddch(y, x, '[' as u32);
    mvaddch(y, x + key.len() as i32 + 1, ']' as u32);
    mvaddstr(y, x + key.len() as i32 + 3, s);
}

pub fn output_grid(st: &State, ui: &UI, ktime: i32) {
    for i in 0..st.grid.width as i32 { for j in 0..st.grid.height as i32 {
        mv(posy(ui, i, j), posx(ui, i, j)-1);
        match st.grid.tiles[i as usize][j as usize].cl {
            TileClass::Mountain => { attrset(((A_NORMAL() | COLOR_PAIR(4) as u32) as i32).try_into().unwrap()); addstr(" /\\^ "); }
            TileClass::Mine => { attrset(((A_NORMAL() | COLOR_PAIR(4) as u32) as i32).try_into().unwrap()); addstr(" /$\\ "); mv(posy(ui,i,j), posx(ui,i,j)+1); if st.grid.tiles[i as usize][j as usize].pl != NEUTRAL as i32 { attrset(((A_BOLD() | COLOR_PAIR(6) as u32) as i32).try_into().unwrap()); } else { attrset(((A_NORMAL() | COLOR_PAIR(6) as u32) as i32).try_into().unwrap()); } addstr("$"); }
            TileClass::Grassland => { attrset(((A_NORMAL() | COLOR_PAIR(4) as u32) as i32).try_into().unwrap()); addstr("  -  "); }
            TileClass::Village => { attrset(((player_style(st.grid.tiles[i as usize][j as usize].pl) as i32).try_into().unwrap())); addstr("  n  "); }
            TileClass::Town => { attrset(((player_style(st.grid.tiles[i as usize][j as usize].pl) as i32).try_into().unwrap())); addstr(" i=i "); }
            TileClass::Castle => { attrset(((player_style(st.grid.tiles[i as usize][j as usize].pl) as i32).try_into().unwrap())); addstr(" W#W "); }
            _ => {}
        }
        attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap());
        if st.grid.tiles[i as usize][j as usize].cl == TileClass::Grassland { attrset(((player_style(st.grid.tiles[i as usize][j as usize].pl) as i32).try_into().unwrap())); output_units(ui, &st.grid.tiles[i as usize][j as usize], i, j); attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap()); }
        for p in 0..MAX_PLAYER as i32 { if p != st.controlled { if st.fg[p as usize].flag[i as usize][j as usize] != 0 && ((ktime + p) / 5) % 10 < 10 { attrset(((player_style(p) as i32).try_into().unwrap())); mvaddch(posy(ui,i,j), posx(ui,i,j), 'x' as u32); attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap()); } } }
        if st.fg[st.controlled as usize].flag[i as usize][j as usize] != 0 && ((ktime) / 5) % 10 < 10 { attrset(((A_BOLD() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap()); mvaddch(posy(ui,i,j), posx(ui,i,j)+2, 'P' as u32); attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap()); }
    }}
    let i = ui.cursor.i; let j = ui.cursor.j; attrset(((A_BOLD() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap()); mvaddch(posy(ui,i,j), posx(ui,i,j)-1, '(' as u32); mvaddch(posy(ui,i+1,j), posx(ui,i+1,j)-1, ')' as u32); attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap());
    let mut y = posy(ui, 0, st.grid.height) + 1; mvaddstr(y, 0, " Gold:"); attrset(((player_style(st.controlled) as i32).try_into().unwrap())); mvaddstr(y, 8, &format!("{}    ", st.country[st.controlled as usize].gold)); attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap()); mvaddstr(y+1, 0, " Prices: 160, 240, 320.");
    mv(y+2, 1); addstr("Speed: "); attrset(((player_style(st.controlled) as i32).try_into().unwrap())); match st.speed { ConfigSpeed::Fastest => { addstr("Fastest"); }, ConfigSpeed::Faster => { addstr("Faster "); }, ConfigSpeed::Fast => { addstr("Fast   "); }, ConfigSpeed::Normal => { addstr("Normal "); }, ConfigSpeed::Slow => { addstr("Slow   "); }, ConfigSpeed::Slower => { addstr("Slower "); }, ConfigSpeed::Slowest => { addstr("Slowest"); }, ConfigSpeed::Pause => { addstr("Pause  "); }, }
    attrset(((A_BOLD() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap());
    let text_style = A_NORMAL() | COLOR_PAIR(1) as u32; let key_style = player_style(st.controlled);
    output_key(y+4, 1, "Space", key_style, "add/remove a flag", text_style);
    output_key(y+5, 1, "R or V", key_style, "build", text_style);
    output_key(y+4, 30, "X", key_style, "remove all flags", text_style);
    output_key(y+5, 30, "C", key_style, "remove 50% of flags", text_style);
    output_key(y+5, 57, "S", key_style, "slow down", text_style);
    output_key(y+4, 57, "F", key_style, "speed up", text_style);
    output_key(y+6, 57, "P", key_style, "pause", text_style);
    mvaddstr(y+1, 30, " Population at the cursor:");
    for p in 1..MAX_PLAYER { attrset(((player_style(p as i32) as i32).try_into().unwrap())); mvaddstr(y+2, 30 + (p as i32)*5, &format!("{:3}", st.grid.tiles[i as usize][j as usize].units[p][UnitClass::Citizen as usize])); }
    attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap());
    mvaddstr(y, 65, "Date:"); attrset(((key_style as i32).try_into().unwrap())); let (yy, mm, dd) = time_to_ymd(st.time); mvaddstr(y, 72, &format!("{}-{:02}-{:02} ", yy, mm, dd));
    refresh();
}

pub fn output_dialog_quit_on(st: &State, ui: &UI) {
    let y = posy(ui, st.grid.width/2, st.grid.height/2);
    let x = posx(ui, st.grid.width/2, st.grid.height/2) - 8;
    attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap());
    mvaddstr(y-2, x, "                 ");
    mvaddstr(y-1, x, "   Quit? [Y/N]   ");
    mvaddstr(y+0, x, "        [Q/Esc]  ");
    mvaddstr(y+1, x, "                 ");
    let text_style = (A_NORMAL() | COLOR_PAIR(1) as u32); let key_style = player_style(st.controlled);
    output_key(y-1, x+9, "Y/N", key_style, "", text_style);
    attrset(((A_NORMAL() | COLOR_PAIR(1) as u32) as i32).try_into().unwrap());
    refresh();
}

pub fn output_dialog_quit_off(_st: &State, ui: &UI) {
    let y = posy(ui, _st.grid.width/2, _st.grid.height/2);
    let x = posx(ui, _st.grid.width/2, _st.grid.height/2) - 8;
    mvaddstr(y-1, x, "                 ");
    mvaddstr(y+0, x, "                 ");
    refresh();
}


