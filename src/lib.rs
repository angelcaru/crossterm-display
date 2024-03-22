pub use crossterm;
use crossterm::{
    cursor,
    style::{self, Attribute, Color},
    terminal, QueueableCommand,
};
use std::io::Write;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
/// A single character on the screen
pub struct Cell {
    /// The character itself
    pub ch: u8,
    /// The foreground color
    pub fg: Color,
    /// The background color
    pub bg: Color,
    /// The attribute (bold, italics, etc.)
    pub attr: Attribute,
}

impl Default for Cell {
    fn default() -> Self {
        Self::empty()
    }
}

impl Cell {
    /// Create an empty cell with a black background
    pub const fn empty() -> Self {
        Self::empty_colored(Color::Black)
    }

    /// Create an empty cell of a certain color
    pub const fn empty_colored(color: Color) -> Self {
        Self {
            ch: b' ',
            fg: Color::White,
            bg: color,
            attr: Attribute::Reset,
        }
    }

    pub fn render<T: Write>(&self, q: &mut T) -> Result<(), std::io::Error> {
        q.queue(style::SetAttribute(self.attr))?;
        q.queue(style::SetForegroundColor(self.fg))?;
        q.queue(style::SetBackgroundColor(self.bg))?;
        q.write_all(&[self.ch])?;
        Ok(())
    }
}

/// Your main handle into crossterm-display.
/// 
/// To create one use TerminalDisplay::new().
///
/// The recommended way to use this is to pass it around to functions that need it
pub struct TerminalDisplay {
    /// The TerminalDisplay's handle to stdout. Use it when you need to directly send
    /// a command to the terminal without going through crossterm-display
    /// ```rust
    /// use crossterm_display::*;
    /// use crossterm::QueueableCommand;
    /// let td = TerminalDisplay::new().unwrap();
    /// td.stdout.queue(crossterm::cursor::Hide).unwrap();
    /// ```
    pub stdout: std::io::Stdout,
    prev_chars: Option<Vec<Vec<Cell>>>,
    chars: Vec<Vec<Cell>>,
    /// The width of the display
    pub w: u16,
    /// The height of the display
    pub h: u16,
}

impl TerminalDisplay {
    /// Create a new TerminalDisplay
    pub fn new() -> Result<Self, std::io::Error> {
        let (w, h) = terminal::size()?;
        Ok(Self {
            stdout: std::io::stdout(),
            prev_chars: None,
            chars: Self::init_chars(w, h),
            w,
            h,
        })
    }

    fn init_chars(w: u16, h: u16) -> Vec<Vec<Cell>> {
        let mut chars = Vec::with_capacity(h.into());
        for _ in 0..h {
            let mut row = Vec::with_capacity(w.into());
            for _ in 0..w {
                row.push(Cell::empty());
            }
            chars.push(row);
        }
        chars
    }

    /// Safely resize the TerminalDisplay. Should always be called whenever the underlying
    /// terminal window resizes
    pub fn resize(&mut self, w: u16, h: u16) {
        self.prev_chars = None;
        self.chars = Self::init_chars(w, h);

        self.w = w;
        self.h = h;
    }

    /// Write a cell into the TerminalDisplay
    pub fn write(&mut self, x: usize, y: usize, ch: Cell) {
        self.chars[y][x] = ch;
    }

    /// Render the TerminalDisplay
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        //self.stdout.queue(cursor::MoveTo(0, 0))?;
        for (y, row) in self.chars.iter().enumerate() {
            if let Some(prev_chars) = &self.prev_chars {
                for (x, cell) in row.iter().enumerate() {
                    if &prev_chars[y][x] != cell {
                        self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                        cell.render(&mut self.stdout)?;
                    }
                }
            } else {
                self.stdout.queue(cursor::MoveTo(0, y as u16))?;
                for cell in row {
                    cell.render(&mut self.stdout)?;
                }
            }
        }
        self.stdout.flush()?;

        self.prev_chars = Some(self.chars.clone());
        self.chars = Self::init_chars(self.w, self.h);

        Ok(())
    }

    /// Clear the TerminalDisplay
    pub fn clear(&mut self) {
        for row in self.chars.iter_mut() {
            row.fill(Cell::empty());
        }
    }

    /// Clear the TerminalDisplay with a certain color
    pub fn clear_colored(&mut self, color: Color) {
        for row in self.chars.iter_mut() {
            row.fill(Cell::empty_colored(color));
        }
    }

    fn queue_clear(&mut self) -> Result<(), std::io::Error> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?;
        self.stdout.queue(cursor::MoveTo(0, 0))?;
        Ok(())
    }
}
