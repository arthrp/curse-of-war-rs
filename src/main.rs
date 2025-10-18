mod common;
mod grid;
mod king;
mod output;
mod state;

use clap::Parser;
use ncurses::*;
use std::time::{Duration, Instant};

use crate::common::*;
use crate::grid::*;
use crate::king::*;
use crate::output::*;
use crate::state::*;

const KEY_Q: i32 = 'q' as i32;
const KEY_Q_UPPER: i32 = 'Q' as i32;
const KEY_F: i32 = 'f' as i32;
const KEY_S: i32 = 's' as i32;
const KEY_P: i32 = 'p' as i32;
const KEY_H: i32 = 'h' as i32;
const KEY_L: i32 = 'l' as i32;
const KEY_K: i32 = 'k' as i32;
const KEY_J: i32 = 'j' as i32;
const KEY_SPACE: i32 = ' ' as i32;
const KEY_X: i32 = 'x' as i32;
const KEY_C: i32 = 'c' as i32;
const KEY_R: i32 = 'r' as i32;
const KEY_V: i32 = 'v' as i32;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'W')]
    w: Option<i32>,
    #[arg(short = 'H')]
    h: Option<i32>,
    #[arg(short = 'S')]
    shape: Option<String>,
    #[arg(short = 'l')]
    loc_num: Option<i32>,
    #[arg(short = 'i')]
    inequality: Option<i32>,
    #[arg(short = 'q')]
    conditions: Option<i32>,
    #[arg(short = 'r')]
    keep_random_flag: bool,
    #[arg(short = 'd')]
    dif: Option<String>,
    #[arg(short = 's')]
    speed: Option<String>,
    #[arg(short = 'R')]
    map_seed: Option<u32>,
    #[arg(short = 'T')]
    timeline: bool,
}

fn main() {
    let cli = Cli::parse();
    let op = BasicOptions {
        keep_random_flag: if cli.keep_random_flag { 1 } else { 0 },
        dif: match cli.dif.as_deref() {
            Some("ee") => ConfigDif::Easiest,
            Some("e") => ConfigDif::Easy,
            Some("n") => ConfigDif::Normal,
            Some("h") => ConfigDif::Hard,
            Some("hh") => ConfigDif::Hardest,
            _ => ConfigDif::Normal,
        },
        speed: match cli.speed.as_deref() {
            Some("p") => ConfigSpeed::Pause,
            Some("sss") => ConfigSpeed::Slowest,
            Some("ss") => ConfigSpeed::Slower,
            Some("s") => ConfigSpeed::Slow,
            Some("n") => ConfigSpeed::Normal,
            Some("f") => ConfigSpeed::Fast,
            Some("ff") => ConfigSpeed::Faster,
            Some("fff") => ConfigSpeed::Fastest,
            _ => ConfigSpeed::Normal,
        },
        w: cli.w.unwrap_or(21).max(14),
        h: cli.h.unwrap_or(21).max(14),
        loc_num: 0,
        map_seed: cli.map_seed.unwrap_or(rand::random()),
        conditions: cli.conditions.unwrap_or(0),
        timeline_flag: if cli.timeline { 1 } else { 0 },
        inequality: cli.inequality.unwrap_or(RANDOM_INEQUALITY),
        shape: match cli.shape.as_deref() {
            Some("rhombus") => Stencil::Rhombus,
            Some("rect") => Stencil::Rect,
            Some("hex") => Stencil::Hex,
            _ => Stencil::Rect,
        },
    };

    // ncurses init
    setlocale(LcCategory::all, "");
    initscr();
    cbreak();
    noecho();
    start_color();
    clear();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    use_default_colors();
    keypad(stdscr(), true);
    init_pair(0, COLOR_WHITE, COLOR_BLACK);
    init_pair(1, COLOR_WHITE, COLOR_BLACK);
    init_pair(2, COLOR_BLACK, COLOR_BLACK);
    init_pair(3, COLOR_RED, COLOR_BLACK);
    init_pair(4, COLOR_GREEN, COLOR_BLACK);
    init_pair(5, COLOR_BLUE, COLOR_BLACK);
    init_pair(6, COLOR_YELLOW, COLOR_BLACK);
    init_pair(7, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(8, COLOR_CYAN, COLOR_BLACK);
    color_set(0);
    assume_default_colors(COLOR_WHITE.into(), COLOR_BLACK.into());
    clear();

    attrset(
        ((A_BOLD() | COLOR_PAIR(2) as u32) as i32)
            .try_into()
            .unwrap(),
    );
    mvaddstr(0, 0, "Map is generated. Please wait.");
    refresh();

    let mut st = State {
        grid: Grid {
            width: 0,
            height: 0,
            tiles: [[Tile {
                cl: TileClass::Grassland,
                pl: NEUTRAL as i32,
                units: [[0; MAX_CLASS]; MAX_PLAYER],
            }; MAX_HEIGHT]; MAX_WIDTH],
        },
        fg: std::array::from_fn(|_| FlagGrid {
            width: 0,
            height: 0,
            flag: [[0; MAX_HEIGHT]; MAX_WIDTH],
            call: [[0; MAX_HEIGHT]; MAX_WIDTH],
        }),
        king: std::array::from_fn(|_| King {
            value: [[0; MAX_HEIGHT]; MAX_WIDTH],
            pl: 0,
            strategy: Strategy::Opportunist,
        }),
        kings_num: 0,
        timeline: Timeline {
            data: [[0.0; MAX_TIMELINE_MARK]; MAX_PLAYER],
            time: [0; MAX_TIMELINE_MARK],
            mark: -1,
        },
        show_timeline: 0,
        country: [Country { gold: 0 }; MAX_PLAYER],
        time: 0,
        map_seed: 0,
        controlled: 1,
        conditions: 0,
        inequality: 0,
        speed: ConfigSpeed::Normal,
        prev_speed: ConfigSpeed::Normal,
        dif: ConfigDif::Normal,
    };
    state_init(&mut st, &op, 1);

    let mut ui = UI {
        cursor: Loc { i: 0, j: 0 },
        xskip: 0,
        xlength: 0,
    };
    ui_init(&st, &mut ui);
    clear();

    // non-blocking input
    nodelay(stdscr(), true);

    // Tick timer
    let mut k: i32 = 0;
    let mut last = Instant::now();

    let mut ch: i32;
    loop {
        ch = getch();
        let mut finished = false;

        if ch != ERR {
            match ch {
                KEY_Q | KEY_Q_UPPER => {
                    finished = true;
                }
                KEY_F => {
                    st.prev_speed = st.speed;
                    st.speed = faster(st.speed);
                }
                KEY_S => {
                    st.prev_speed = st.speed;
                    st.speed = slower(st.speed);
                }
                KEY_P => {
                    if let ConfigSpeed::Pause = st.speed {
                        st.speed = st.prev_speed;
                    } else {
                        st.prev_speed = st.speed;
                        st.speed = ConfigSpeed::Pause;
                    }
                }
                KEY_H | KEY_LEFT => {
                    let i = ui.cursor.i - 1;
                    let j = ui.cursor.j;
                    adjust_cursor(&st, &mut ui, i, j);
                }
                KEY_L | KEY_RIGHT => {
                    let i = ui.cursor.i + 1;
                    let j = ui.cursor.j;
                    adjust_cursor(&st, &mut ui, i, j);
                }
                KEY_K | KEY_UP => {
                    let j = ui.cursor.j - 1;
                    let mut i = ui.cursor.i;
                    if j % 2 == 1 {
                        i += 1;
                    }
                    adjust_cursor(&st, &mut ui, i, j);
                }
                KEY_J | KEY_DOWN => {
                    let j = ui.cursor.j + 1;
                    let mut i = ui.cursor.i;
                    if j % 2 == 0 {
                        i -= 1;
                    }
                    adjust_cursor(&st, &mut ui, i, j);
                }
                KEY_SPACE => {
                    let i = ui.cursor.i;
                    let j = ui.cursor.j;
                    if st.fg[st.controlled as usize].flag[i as usize][j as usize] == 0 {
                        add_flag(
                            &st.grid,
                            &mut st.fg[st.controlled as usize],
                            i,
                            j,
                            FLAG_POWER,
                        );
                    } else {
                        remove_flag(
                            &st.grid,
                            &mut st.fg[st.controlled as usize],
                            i,
                            j,
                            FLAG_POWER,
                        );
                    }
                }
                KEY_X => {
                    remove_flags_with_prob(&st.grid, &mut st.fg[st.controlled as usize], 1.0);
                }
                KEY_C => {
                    remove_flags_with_prob(&st.grid, &mut st.fg[st.controlled as usize], 0.5);
                }
                KEY_R | KEY_V => {
                    let i = ui.cursor.i;
                    let j = ui.cursor.j;
                    let _ = build(
                        &mut st.grid,
                        &mut st.country[st.controlled as usize],
                        st.controlled,
                        i,
                        j,
                    );
                }
                _ => {}
            }
        }
        if finished {
            break;
        }

        if last.elapsed() >= Duration::from_millis(10) {
            last = Instant::now();
            k += 1;
            if k >= 1600 {
                k = 0;
            }
            let slowdown = game_slowdown(st.speed);
            if k % slowdown == 0 {
                if !matches!(st.speed, ConfigSpeed::Pause) {
                    kings_move(&mut st);
                    simulate(&mut st);
                    if st.show_timeline == 1 {
                        if (st.time % 10) == 0 {
                            update_timeline(&mut st);
                        }
                    }
                }
            }
            output_grid(&st, &ui, k);
        }
    }

    // teardown
    echo();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    clear();
    endwin();
}
