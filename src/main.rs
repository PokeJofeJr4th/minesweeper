#![warn(clippy::pedantic, clippy::nursery)]
use std::{
    env::args,
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

use minefield::{Cell, Minefield};
use term_util::TermHandle;

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() == 3 {
        if let (Ok(rows), Ok(cols)) = (args[1].parse(), args[2].parse()) {
            run_field(rows, cols).unwrap();
            return;
        }
    } else if args.len() == 2 {
        if args[1].to_lowercase() == "max" {
            if let Some((cols, rows)) = terminal_size::terminal_size() {
                run_field(rows.0 as usize - 4, (cols.0 as usize - 3) / 2).unwrap();
            } else {
                println!("Failed to get the terminal window size :(");
            }
            return;
        }
    } else if args.len() == 1 {
        run_field(20, 20).unwrap();
        return;
    }
    println!(
        "Invalid args. Options:\n\tminesweeper\n\tminesweeper <ROWS> <COLS>\n\tminesweeper max"
    );
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]
fn run_field(rows: usize, cols: usize) -> Result<(), Box<dyn Error>> {
    let handle = TermHandle::new()?;
    let mut stdout = stdout();
    let mut field = Minefield::generate(rows, cols);
    field.print();
    let mut col = 0;
    let mut row = 0;
    let mut first_one = true;
    'turn: loop {
        print_cell(&mut stdout, &field, row, col)?;
        execute!(stdout, MoveTo(col * 2 + 1, row + 1))?;
        print!(">");
        stdout.flush()?;
        // check for win/lose conditions
        if field
            .iter()
            .all(|cell| (cell.is_mine && cell.is_flagged) || cell.is_revealed)
        {
            end_game("You Win!", &field, row, col, handle)?;
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
                    let is_modified = modifiers == KeyModifiers::SHIFT
                        || modifiers == KeyModifiers::CONTROL
                        || modifiers == KeyModifiers::ALT;
                    match code {
                        KeyCode::Down => {
                            execute!(stdout, MoveTo(col * 2 + 1, row + 1))?;
                            print!(" ");
                            if is_modified {
                                let start_revealed = field
                                    .get(row as usize, col as usize)
                                    .is_some_and(|cell| cell.is_revealed);
                                let mut i = field.rows() / 2;
                                while (row as usize) < field.rows() - 1 && i > 0 {
                                    row += 1;
                                    i -= 1;
                                    let is_revealed = field
                                        .get(row as usize, col as usize)
                                        .is_some_and(|cell| cell.is_revealed);
                                    if is_revealed ^ start_revealed {
                                        break;
                                    }
                                }
                            } else {
                                row = (row + 1).rem_euclid(field.rows() as u16);
                            }
                        }
                        KeyCode::Up => {
                            execute!(stdout, MoveTo(col * 2 + 1, row + 1))?;
                            print!(" ");
                            if is_modified {
                                let start_revealed = field
                                    .get(row as usize, col as usize)
                                    .is_some_and(|cell| cell.is_revealed);
                                let mut i = field.rows() / 2;
                                while row > 0 && i > 0 {
                                    row -= 1;
                                    i -= 1;
                                    let is_revealed = field
                                        .get(row as usize, col as usize)
                                        .is_some_and(|cell| cell.is_revealed);
                                    if is_revealed ^ start_revealed {
                                        break;
                                    }
                                }
                            } else {
                                row = (((row as i16) - 1).rem_euclid(field.rows() as i16)) as u16;
                            }
                        }
                        KeyCode::Right => {
                            execute!(stdout, MoveTo(col * 2 + 1, row + 1))?;
                            print!(" ");
                            if is_modified {
                                let start_revealed = field
                                    .get(row as usize, col as usize)
                                    .is_some_and(|cell| cell.is_revealed);
                                let mut i = field.cols() / 2;
                                while (col as usize) < field.cols() - 1 && i > 0 {
                                    col += 1;
                                    i -= 1;
                                    let is_revealed = field
                                        .get(row as usize, col as usize)
                                        .is_some_and(|cell| cell.is_revealed);
                                    if is_revealed ^ start_revealed {
                                        break;
                                    }
                                }
                            } else {
                                col = (col + 1).rem_euclid(field.cols() as u16);
                            }
                        }
                        KeyCode::Left => {
                            execute!(stdout, MoveTo(col * 2 + 1, row + 1))?;
                            print!(" ");
                            if is_modified {
                                let start_revealed = field
                                    .get(row as usize, col as usize)
                                    .is_some_and(|cell| cell.is_revealed);
                                let mut i = field.cols() / 2;
                                while col > 0 && i > 0 {
                                    col -= 1;
                                    i -= 1;
                                    let is_revealed = field
                                        .get(row as usize, col as usize)
                                        .is_some_and(|cell| cell.is_revealed);
                                    if is_revealed ^ start_revealed {
                                        break;
                                    }
                                }
                            } else {
                                col = (((col as i16) - 1).rem_euclid(field.cols() as i16)) as u16;
                            }
                        }
                        KeyCode::Char(' ') => {
                            let cell = field.get_mut(row as usize, col as usize).unwrap();
                            if !cell.is_revealed {
                                cell.is_flagged = !cell.is_flagged;
                            }
                        }
                        KeyCode::Enter => {
                            if is_modified {
                                // dig a whole 3x3 area
                                for dy in 0..3 {
                                    let Some(row) = (row + dy).checked_sub(1) else {
                                        continue;
                                    };
                                    for dx in 0..3 {
                                        let Some(col) = (col + dx).checked_sub(1) else {
                                            continue;
                                        };
                                        let Some(&cell) = dig(&mut field, row, col)? else {
                                            continue;
                                        };
                                        print_cell(&mut stdout, &field, row, col)?;
                                        if !cell.is_flagged && cell.is_mine {
                                            kaboom(&mut field, row, col, handle)?;
                                            return Ok(());
                                        }
                                    }
                                }
                            } else {
                                if first_one {
                                    clear_mines(&mut field, row, col);
                                    first_one = false;
                                }
                                if let Some(&cell) = dig(&mut field, row, col)? {
                                    if !cell.is_flagged && cell.is_mine {
                                        kaboom(&mut field, row, col, handle)?;
                                        return Ok(());
                                    }
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

fn kaboom(
    field: &mut Minefield,
    row: u16,
    col: u16,
    handle: TermHandle,
) -> Result<(), Box<dyn Error>> {
    for row in 0..field.rows() {
        for col in 0..field.cols() {
            let cell = field.get_mut(row, col).unwrap();
            let needs_update = if cell.is_flagged {
                cell.is_flagged = false;
                true
            } else if cell.is_mine {
                cell.is_revealed = true;
                true
            } else {
                false
            };
            if needs_update {
                #[allow(clippy::cast_possible_truncation)]
                print_cell(&mut stdout(), field, row as u16, col as u16)?;
            }
        }
    }

    end_game("Kaboom", field, row, col, handle)?;
    Ok(())
}

fn dig(field: &mut Minefield, row: u16, col: u16) -> Result<Option<&Cell>, Box<dyn Error>> {
    let Some(cell) = field.get(row as usize, col as usize) else {
        return Ok(None);
    };
    if !cell.is_flagged {
        if cell.adjacent == 0 && !cell.is_mine {
            reveal_empty(field, row, col)?;
        } else {
            // just reveal the current cell
            field
                .get_mut(row as usize, col as usize)
                .unwrap()
                .is_revealed = true;
        }
    }
    let cell = field.get(row as usize, col as usize).unwrap();
    Ok(Some(cell))
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
    field: &Minefield,
    row: u16,
    col: u16,
) -> Result<(), Box<dyn Error>> {
    execute!(stdout, MoveTo(col * 2 + 1, row + 1))?;
    if let Some(cell) = field.get(row as usize, col as usize) {
        cell.print();
    }
    stdout.flush()?;
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn end_game(
    message: &str,
    field: &Minefield,
    row: u16,
    col: u16,
    handle: TermHandle,
) -> Result<(), Box<dyn Error>> {
    execute!(stdout(), MoveTo(2, field.rows() as u16 + 1))?;
    println!("{message}");
    print_cell(&mut stdout(), field, row, col)?;
    wait_till_esc()?;
    drop(handle);
    Ok(())
}

fn wait_till_esc() -> Result<(), Box<dyn Error>> {
    loop {
        let event = event::read()?;
        if matches!(
            event,
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                ..
            })
        ) {
            return Ok(());
        }
    }
}
