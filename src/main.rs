#![warn(clippy::pedantic, clippy::nursery)]
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

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
fn run_field() -> Result<(), Box<dyn Error>> {
    let handle = TermHandle::new()?;
    let mut stdout = stdout();
    let mut field = Minefield::generate(20, 20);
    field.print();
    let mut row = 0;
    let mut col = 0;
    let mut first_one = true;
    'turn: loop {
        print_cell(&mut stdout, &mut field, row, col)?;
        execute!(stdout, MoveTo(row * 2 + 1, col + 1))?;
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
                }) => {
                    let amount = if modifiers == KeyModifiers::SHIFT {
                        3
                    } else {
                        1
                    };
                    match code {
                        KeyCode::Down => {
                            execute!(stdout, MoveTo(row * 2 + 1, col + 1))?;
                            print!(" ");
                            col = (col + amount).rem_euclid(field.cols() as u16);
                        }
                        KeyCode::Up => {
                            execute!(stdout, MoveTo(row * 2 + 1, col + 1))?;
                            print!(" ");
                            col = (((col as i16) - (amount as i16)).rem_euclid(field.cols() as i16))
                                as u16;
                        }
                        KeyCode::Right => {
                            execute!(stdout, MoveTo(row * 2 + 1, col + 1))?;
                            print!(" ");
                            row = (row + amount).rem_euclid(field.rows() as u16);
                        }
                        KeyCode::Left => {
                            execute!(stdout, MoveTo(row * 2 + 1, col + 1))?;
                            print!(" ");
                            row = (((row as i16) - (amount as i16)).rem_euclid(field.rows() as i16))
                                as u16;
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
                                    clear_mines(&mut field, row, col);
                                    first_one = false;
                                } else if cell.is_mine {
                                    drop(handle);
                                    println!("kaboom");
                                    return Ok(());
                                }
                                let cell = field.get(row as usize, col as usize).unwrap();
                                if cell.adjacent == 0 {
                                    reveal_empty(&mut field, row, col)?;
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
                    }
                }
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

fn clear_mines(field: &mut Minefield, row: u16, col: u16) {
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
}

#[allow(clippy::cast_possible_truncation)]
fn reveal_empty(field: &mut Minefield, row: u16, col: u16) -> Result<(), Box<dyn Error>> {
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
                    if let Some(cell) =
                        field.get((row + dy).wrapping_sub(1), (col + dx).wrapping_sub(1))
                    {
                        if !cell.is_revealed {
                            zero_queue
                                .push(((row + dy).wrapping_sub(1), (col + dx).wrapping_sub(1)));
                        }
                    }
                }
            }
        }
        print_cell(&mut stdout(), field, row as u16, col as u16)?;
    }
    Ok(())
}

fn print_cell(
    stdout: &mut Stdout,
    field: &mut Minefield,
    row: u16,
    col: u16,
) -> Result<(), Box<dyn Error>> {
    execute!(stdout, MoveTo(row * 2 + 1, col + 1))?;
    if let Some(cell) = field.get(row as usize, col as usize) {
        cell.print();
    }
    stdout.flush()?;
    Ok(())
}
