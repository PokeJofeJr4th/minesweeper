use bitfield::bitfield;
use rand::{thread_rng, Rng};

// #[derive(Default, Clone, Copy, Debug)]
// pub struct Cell {
//     pub adjacent: u8,
//     pub is_revealed: bool,
//     pub is_mine: bool,
//     pub is_flagged: bool,
// }

bitfield! {
    #[derive(Clone, Copy, Default)]
    pub struct Cell(u16);
    impl Debug;
    u8;
    pub adjacent, set_adjacent: 7,0;
    bool;
    pub is_revealed, set_is_revealed: 8;
    pub is_mine, set_is_mine: 9;
    pub is_flagged, set_is_flagged: 10;
}

impl Cell {
    pub fn print(self) {
        const COLORS: &[[&str; 2]] = &[
            ["", ""],
            // 1 = blue
            ["\x1b[94m", "\x1b[0m"],
            // 2 = green
            ["\x1b[92m", "\x1b[0m"],
            // 3 = red
            ["\x1b[91m", "\x1b[0m"],
            // 4 = purple
            ["\x1b[95m", "\x1b[0m"],
            // 5 = orange
            ["\x1b[93m", "\x1b[0m"],
            // 6 = ???
            ["\x1b[93m", "\x1b[0m"],
            // 7 = ???
            ["\x1b[93m", "\x1b[0m"],
            // 8 = ???
            ["\x1b[93m", "\x1b[0m"],
        ];
        if self.is_flagged() {
            print!(" \x1b[93m%\x1b[0m");
        } else if !self.is_revealed() {
            print!(" ?");
        } else if self.is_mine() {
            print!(" \x1b[91m@\x1b[0m");
        } else if self.adjacent() == 0 {
            print!("  ");
        } else {
            print!(
                " {}{}{}",
                COLORS[self.adjacent() as usize][0],
                self.adjacent(),
                COLORS[self.adjacent() as usize][1]
            );
        }
    }
}

pub struct Minefield {
    rows: usize,
    cols: usize,
    content: Vec<Vec<Cell>>,
}

impl Minefield {
    /// the number of rows in the minefield
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// the number of columns in the minefield
    pub const fn cols(&self) -> usize {
        self.cols
    }

    /// get a mutable reference to a single cell
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        self.content.get_mut(row)?.get_mut(col)
    }

    /// get an immutable reference to a single cell
    pub fn get(&self, row: usize, col: usize) -> Option<&Cell> {
        self.content.get(row)?.get(col)
    }

    /// generate a new random minefield of a specific size
    pub fn generate(rows: usize, cols: usize) -> Self {
        let mut this = Self {
            rows,
            cols,
            content: std::iter::repeat_with(|| vec![Cell::default(); cols])
                .take(rows)
                .collect(),
        };
        let mut random = thread_rng();
        for _ in 0..(rows * cols / 5) {
            let row = random.gen_range(0..rows);
            let col = random.gen_range(0..cols);
            let cell_ref = this.get_mut(row, col).unwrap();
            cell_ref.set_is_mine(true);
        }
        this.calculate_adjacent();
        this
    }

    /// Internally calculate how many cells are adjacent to each other cell.
    /// This should be called any time a cell changes between being a mine or not.
    pub fn calculate_adjacent(&mut self) {
        // first, 2d loop through each cell
        for row in 0..self.rows {
            for col in 0..self.cols {
                // if the cell is a mine, we don't need to know how many are adjacent
                if self.get(row, col).is_some_and(Cell::is_mine) {
                    continue;
                }
                let mut adj = 0;
                // 2d loop through all adjacent cells
                for dx in 0..3 {
                    for dy in 0..3 {
                        // ignore current cell
                        if dx == 1 && dy == 1 {
                            continue;
                        }
                        // check if the adjacent cell exists and is a mine
                        if self
                            .get((row + dy).wrapping_sub(1), (col + dx).wrapping_sub(1))
                            .is_some_and(Cell::is_mine)
                        {
                            adj += 1;
                        }
                    }
                }
                self.get_mut(row, col).unwrap().set_adjacent(adj);
            }
        }
    }

    pub fn print(&self) {
        let bar = "─".repeat(self.cols * 2 + 1);
        println!("┌{bar}┐");
        for row in &self.content {
            print!("│");
            for cell in row {
                cell.print();
            }
            println!(" │");
        }
        print!("└{bar}┘");
    }

    /// Iterate through all cells in the minefield
    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.content.iter().flat_map(|row| row.iter())
    }

    // /// Iterate through all cells in the minefield
    // pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
    //     self.content.iter_mut().flat_map(|row| row.iter_mut())
    // }
}
