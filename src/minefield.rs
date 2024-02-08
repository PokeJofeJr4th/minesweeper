use rand::{thread_rng, Rng};

#[derive(Default, Clone, Copy, Debug)]
pub struct Cell {
    pub is_revealed: bool,
    pub is_mine: bool,
    pub is_flagged: bool,
    pub adjacent: u8,
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
        if self.is_flagged {
            print!(" !");
        } else if !self.is_revealed {
            print!(" ?");
        } else if self.is_mine {
            print!(" @");
        } else if self.adjacent == 0 {
            print!("  ");
        } else {
            print!(
                " {}{}{}",
                COLORS[self.adjacent as usize][0], self.adjacent, COLORS[self.adjacent as usize][1]
            );
        }
    }
}

pub struct Minefield {
    rows: usize,
    cols: usize,
    content: Vec<Vec<Cell>>,
    mines: Vec<[usize; 2]>,
}

impl Minefield {
    pub fn foreach_mut(&mut self, mut func: impl FnMut(&mut Cell)) {
        for row in &mut self.content {
            for cell in row {
                func(cell);
            }
        }
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        self.content.get_mut(row)?.get_mut(col)
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&Cell> {
        self.content.get(row)?.get(col)
    }

    pub fn generate(rows: usize, cols: usize) -> Self {
        let mut this = Self {
            rows,
            cols,
            content: std::iter::repeat_with(|| vec![Cell::default(); cols])
                .take(rows)
                .collect(),
            mines: Vec::new(),
        };
        let mut random = thread_rng();
        for _ in 0..(rows * cols / 6) {
            let row = random.gen_range(0..rows);
            let col = random.gen_range(0..cols);
            let cell_ref = this.get_mut(row, col).unwrap();
            if cell_ref.is_mine {
                continue;
            }
            cell_ref.is_mine = true;
            // drop cell ref
            this.mines.push([row, col]);
        }
        this.calculate_adjacent();
        this
    }

    pub fn calculate_adjacent(&mut self) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                if self.get(row, col).is_some_and(|cell| cell.is_mine) {
                    continue;
                }
                let mut adj = 0;
                for dx in 0..3 {
                    for dy in 0..3 {
                        if dx == 1 && dy == 1 {
                            continue;
                        }
                        if self
                            .get((row + dy).wrapping_sub(1), (col + dx).wrapping_sub(1))
                            .is_some_and(|cell| cell.is_mine)
                        {
                            adj += 1;
                        }
                    }
                }
                self.get_mut(row, col).unwrap().adjacent = adj;
            }
        }
    }

    pub fn print_mines(&self) {
        for row in &self.content {
            print!("|");
            for cell in row {
                cell.print();
            }
            println!(" |");
        }
    }
}
