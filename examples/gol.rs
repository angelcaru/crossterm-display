// This example is larger than the library itself for some reason

use crossterm::style::*;
use crossterm::QueueableCommand;
use crossterm_display::*;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

const fn rgb_color(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

const GRAYISH: Color = rgb_color(0x18, 0x18, 0x18);

type GoLBoard = Vec<Vec<bool>>;

trait GoL {
    fn sized(w: usize, h: usize) -> Self;
    fn next_state(&self) -> Self;
    fn render(&self, td: &mut TerminalDisplay) -> Result<(), std::io::Error>;
}

// Why doesn't Rust already have this?
fn cartesian_product<'a, T: Clone + 'a>(
    xs: impl Iterator<Item = T> + 'a,
    ys: impl Iterator<Item = T> + 'a + Clone,
) -> impl Iterator<Item = (T, T)> + 'a {
    xs.flat_map(move |x| ys.clone().map(move |y| (x.clone(), y.clone())))
}

fn count_nbors(gol: &GoLBoard, x: isize, y: isize) -> u8 {
    let (w, h) = (gol[0].len() as isize, gol.len() as isize);
    cartesian_product((x - 1)..=(x + 1), (y - 1)..=(y + 1))
        .filter(|arg| arg != &(x, y))
        .map(|(nx, ny)| (nx.rem_euclid(w), ny.rem_euclid(h)))
        .map(|(nx, ny)| (nx as usize, ny as usize))
        .map(|(nx, ny)| gol[ny][nx] as u8)
        .sum()
}

impl GoL for GoLBoard {
    fn sized(w: usize, h: usize) -> Self {
        vec![vec![false; w]; h]
    }
    fn next_state(&self) -> Self {
        let mut res = Self::sized(self[0].len(), self.len());
        let (x, y) = (2usize, 1usize);
        let cell = self[y][x];
        for (y, row) in self.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let nbors = count_nbors(self, x as isize, y as isize);
                res[y][x] = match (cell, nbors) {
                    // Any live cell with fewer than two live neighbors dies, as if by underpopulation.
                    (true, 0 | 1) => false,
                    // Any live cell with two or three live neighbors lives on to the next generation.
                    (true, 2 | 3) => true,
                    // Any live cell with more than three live neighbors dies, as if by overpopulation.
                    (true, 3..=8) => false,
                    // Any dead cell with exactly three live neighbors becomes a live cell, as if by reproduction.
                    (false, 3) => true,
                    // Any other dead cell stays dead, as if by common sense
                    (false, _) => false,
                    _ => unreachable!("count_nbors() should only return values 0-8"),
                };
            }
        }
        res
    }
    fn render(&self, td: &mut TerminalDisplay) -> Result<(), std::io::Error> {
        for (y, row) in self.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                let ch = if cell { '#' } else { ' ' };
                td.write(
                    x,
                    y,
                    Cell {
                        ch,
                        fg: Color::White,
                        bg: GRAYISH,
                        attr: Attribute::Reset,
                    },
                );
            }
        }
        Ok(())
    }
}

fn handle_event(
    ev: crossterm::event::Event,
    cur: &mut (i32, i32),
    w: i32,
    h: i32,
    gol: &mut GoLBoard,
    auto: &mut bool,
) {
    use crossterm::event as ev;
    use ev::KeyCode;
    let ev::Event::Key(ev) = ev else {
        return;
    };
    match ev {
        ev::KeyEvent {
            code: KeyCode::Up, ..
        } => cur.1 = (cur.1 - 1).rem_euclid(h),
        ev::KeyEvent {
            code: KeyCode::Right,
            ..
        } => cur.0 = (cur.0 + 1).rem_euclid(w),
        ev::KeyEvent {
            code: KeyCode::Down,
            ..
        } => cur.1 = (cur.1 + 1).rem_euclid(h),
        ev::KeyEvent {
            code: KeyCode::Left,
            ..
        } => cur.0 = (cur.0 - 1).rem_euclid(w),

        ev::KeyEvent {
            code: KeyCode::Char(' ') | KeyCode::Enter,
            ..
        } => {
            let &mut (cx, cy) = cur;
            let (cx, cy) = (cx as usize, cy as usize);
            gol[cy][cx] = !gol[cy][cx];
        }
        ev::KeyEvent {
            code: KeyCode::Char('n' | 'N'),
            ..
        } => *gol = gol.next_state(),

        ev::KeyEvent {
            code: KeyCode::Char('a' | 'A'),
            ..
        } => *auto = !*auto,

        ev::KeyEvent {
            code: KeyCode::Char('q' | 'Q'),
            ..
        } => {
            let _ = crossterm::terminal::disable_raw_mode();
            std::process::exit(0);
        },

        _ => {}
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut td = TerminalDisplay::new()?;
    crossterm::terminal::enable_raw_mode()?;

    td.stdout.queue(crossterm::cursor::Hide)?;

    let mut board = GoLBoard::sized(td.w as usize, td.h as usize);
    // #··
    // ··#
    // ###
    board[0][1] = true;
    board[1][2] = true;
    board[2][0] = true;
    board[2][1] = true;
    board[2][2] = true;

    let mut cur = (0i32, 0i32);
    let mut auto = false;
    loop {
        use crossterm::event as ev;
        if auto {
            board = board.next_state();
        }

        td.clear_colored(GRAYISH);
        board.render(&mut td)?;

        td.write(
            cur.0 as usize,
            cur.1 as usize,
            Cell {
                ch: '@',
                fg: Color::White,
                bg: GRAYISH,
                attr: Attribute::Reset,
            },
        );

        td.render()?;
        if ev::poll(Duration::from_millis(20))? {
            handle_event(
                ev::read()?,
                &mut cur,
                td.w as i32,
                td.h as i32,
                &mut board,
                &mut auto,
            );
        }
        if auto {
            sleep(Duration::from_millis(20))
        }
    }
}
