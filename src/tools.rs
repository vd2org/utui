use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
    disable_raw_mode, enable_raw_mode,
};
use std::error::Error;
use std::io::{Write, stdout};
use std::panic;

//Initializes terminal
pub fn init(mut w: impl Write) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(
        w,
        EnterAlternateScreen,
        Clear(ClearType::All),
        DisableLineWrap
    )?;
    Ok(())
}

/// Restores normal terminal operation
pub fn restore(mut w: impl Write) -> Result<(), Box<dyn Error>> {
    execute!(w, LeaveAlternateScreen, EnableLineWrap, Show)?;
    disable_raw_mode()?;
    Ok(())
}

/// Registers panic hook to restore terminal state on panic
pub fn install_panic_hook() {
    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        let _ = execute!(stdout(), LeaveAlternateScreen, EnableLineWrap, Show);
        let _ = disable_raw_mode();
        default_hook(panic_info);
    }));
}
