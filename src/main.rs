use std::{
    error::Error,
    io::{stdout, Stdout, Write},
};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
};

mod minefield;
mod term_util;

use minefield::Minefield;
use term_util::TermHandle;

fn main() {
    run_field().unwrap();
}

fn run_field() -> Result<(), Box<dyn Error>> {
    let handle = TermHandle::new()?;
    let mut stdout = stdout();
    let mut field = Minefield::generate(20, 20);
    field.print_mines();
    let mut row = 0;
    let mut col = 0;
    let mut first_one = true;
    'turn: loop {
        print_cell(&mut stdout, &mut field, row, col)?;
        execute!(stdout, MoveTo(row * 2, col))?;
        print!(">");
        stdout.flush()?;
        // check for win/lose conditions
        if field.iter().all(|cell| cell.is_mine || cell.is_revealed) {
            drop(handle);
            println!("You win!");
            return Ok(());
        }
        'event: loop {
            let event = event::read()?;
            match event {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: KeyEventKind::Press,
                    ..
                }) => match code {
                    KeyCode::Down => {
                        execute!(stdout, MoveTo(row * 2, col))?;
                        print!(" ");
                        let amount = if modifiers == KeyModifiers::SHIFT {
                            3
                        } else {
                            1
                        };
                        col = (col + amount).rem_euclid(field.cols() as u16);
                    }
                    KeyCode::Up => {
                        execute!(stdout, MoveTo(row * 2, col))?;
                        print!(" ");
                        let amount = if modifiers == KeyModifiers::SHIFT {
                            3
                        } else {
                            1
                        };
                        col = (((col as i16) - amount).rem_euclid(field.cols() as i16)) as u16;
                    }
                    KeyCode::Right => {
                        execute!(stdout, MoveTo(row * 2, col))?;
                        print!(" ");
                        let amount = if modifiers == KeyModifiers::SHIFT {
                            3
                        } else {
                            1
                        };
                        row = (row + amount).rem_euclid(field.rows() as u16);
                    }
                    KeyCode::Left => {
                        execute!(stdout, MoveTo(row * 2, col))?;
                        print!(" ");
                        let amount = if modifiers == KeyModifiers::SHIFT {
                            3
                        } else {
                            1
                        };
                        row = (((row as i16) - amount).rem_euclid(field.rows() as i16)) as u16;
                    }
                    KeyCode::Char(' ') => {
                        let cell = field.get_mut(row as usize, col as usize).unwrap();
                        if !cell.is_revealed {
                            cell.is_flagged = !cell.is_flagged;
                        }
                    }
                    KeyCode::Enter => {
                        let cell = field.get_mut(row as usize, col as usize).unwrap();
                        if !cell.is_flagged {
                            if first_one {
                                for dy in 0..3 {
                                    for dx in 0..3 {
                                        if let Some(cell) = field.get_mut(
                                            (row as usize + dy).wrapping_sub(1),
                                            (col as usize + dx).wrapping_sub(1),
                                        ) {
                                            cell.is_mine = false;
                                        }
                                    }
                                }
                                field.calculate_adjacent();
                                first_one = false;
                            } else if cell.is_mine {
                                drop(handle);
                                println!("kaboom");
                                return Ok(());
                            }
                            let cell = field.get(row as usize, col as usize).unwrap();
                            if cell.adjacent == 0 {
                                // recursively reveal empty cells
                                let mut zero_queue = vec![(row as usize, col as usize)];
                                while let Some((row, col)) = zero_queue.pop() {
                                    let Some(cell) = field.get_mut(row, col) else {
                                        continue;
                                    };
                                    if cell.is_revealed {
                                        continue;
                                    }
                                    if cell.is_flagged {
                                        cell.is_flagged = false;
                                    }
                                    cell.is_revealed = true;
                                    if cell.adjacent == 0 {
                                        for dy in 0..3 {
                                            for dx in 0..3 {
                                                if dy == 1 && dx == 1 {
                                                    continue;
                                                }
                                                if let Some(cell) = field.get(
                                                    (row + dy).wrapping_sub(1),
                                                    (col + dx).wrapping_sub(1),
                                                ) {
                                                    if !cell.is_revealed {
                                                        zero_queue.push((
                                                            (row + dy).wrapping_sub(1),
                                                            (col + dx).wrapping_sub(1),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    print_cell(&mut stdout, &mut field, row as u16, col as u16)?;
                                }
                            } else {
                                // just reveal the current cell
                                field
                                    .get_mut(row as usize, col as usize)
                                    .unwrap()
                                    .is_revealed = true;
                            }
                        }
                    }
                    KeyCode::Esc => break 'turn,
                    _ => continue 'event,
                },
                _ => {
                    continue 'event;
                }
            }
            break 'event;
        }
    }
    drop(handle);
    Ok(())
}

fn print_cell(
    stdout: &mut Stdout,
    field: &mut Minefield,
    row: u16,
    col: u16,
) -> Result<(), Box<dyn Error>> {
    execute!(stdout, MoveTo(row * 2, col))?;
    if let Some(cell) = field.get(row as usize, col as usize) {
        cell.print();
    }
    stdout.flush()?;
    Ok(())
}
