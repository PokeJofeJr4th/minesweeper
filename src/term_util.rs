use std::{error::Error, io::stdout, marker::PhantomData};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

pub struct TermHandle(PhantomData<()>);

impl TermHandle {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        execute!(
            stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            Hide,
            MoveTo(0, 0)
        )?;
        enable_raw_mode()?;
        Ok(Self(PhantomData))
    }
}

impl Drop for TermHandle {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(stdout(), Show, LeaveAlternateScreen).unwrap();
    }
}
