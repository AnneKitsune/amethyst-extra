








































use crossterm::*;

//use crossterm::screen::RawScreen;





lazy_static! {
    static ref CROSSTERM: Crossterm = {
        let mut screen = Screen::new(true);
        screen.disable_drop();
        Crossterm::new(&screen)
    };
}

#[cfg(test)]
mod test {
    use crate::terminal::*;

    























use std::io::Read as IORead;





use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;



use crossterm::cursor::TerminalCursor;
//use crossterm::screen::RawScreen;
use crossterm::terminal::{ClearType, Terminal};






    #[test]
    pub fn crossterm() {
        let terminal = CROSSTERM.terminal();
        let cursor = CROSSTERM.cursor();
        //cursor.hide();

        let mut input = CROSSTERM.input().read_async().bytes();

        let input_buf = Arc::new(Mutex::new(String::new()));
        let _key_buf = [0 as u8; 32];

        start_logger(input_buf.clone());

        spawn(|| loop {
            info!("More random stuff");
            sleep(Duration::from_millis(52));
        });

        loop {
            let (_, _) = terminal.terminal_size();
            info!("random stuff");
            while let Some(Ok(b)) = input.next() {
                info!("{:?} <- Char entered!", b);
                if b == 3 {
                    // Ctrl+C = exit
                    terminal.exit();
                    return;
                } else if b == b'\n' || b == 13 {
                    //info!(">{}", input_buf.lock().unwrap());
                    let mut buffer = input_buf.lock().unwrap();
                    buffer.clear();
                    refresh_input_line(&terminal, &cursor, &buffer);
                //let input = CROSSTERM.input().read_async().bytes();
                } else if b == 127 || b == 8 {
                    // Delete || Backspace
                    let mut buffer = input_buf.lock().unwrap();
                    buffer.pop();
                    refresh_input_line(&terminal, &cursor, &buffer);
                } else {
                    let mut buffer = input_buf.lock().unwrap();
                    buffer.push(b as char);
                    refresh_input_line(&terminal, &cursor, &buffer);
                }
            }
            sleep(Duration::from_millis(100));
        }
    }

    pub fn swap_write(terminal: &Terminal, cursor: &TerminalCursor, msg: &str, input_buf: &String) {
        let (_, term_height) = terminal.terminal_size();
        cursor.goto(0, term_height);
        terminal.clear(ClearType::CurrentLine);
        terminal.write(format!("{}\r\n>{}", msg, input_buf));
        //terminal.write(format!(">{}", input_buf));
    }

    pub fn refresh_input_line(terminal: &Terminal, cursor: &TerminalCursor, input_buf: &String) {
        let (_, term_height) = terminal.terminal_size();
        cursor.goto(0, term_height);
        terminal.clear(ClearType::CurrentLine);
        terminal.write(format!(">{}", input_buf));
    }

    pub fn start_logger(input_buf: Arc<Mutex<String>>) {
        let color_config = fern::colors::ColoredLevelConfig::new();
        let terminal = CROSSTERM.terminal();
        let cursor = CROSSTERM.cursor();

        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{color}[{level}][{target}] {message}{color_reset}",
                    color = format!(
                        "\x1B[{}m",
                        color_config.get_color(&record.level()).to_fg_str()
                    ),
                    level = record.level(),
                    target = record.target(),
                    message = message,
                    color_reset = "\x1B[0m",
                ))
            })
            .level(log::LevelFilter::Debug)
            .chain(fern::Output::call(move |record| {
                //let color = color_config.get_color(&record.level()).to_fg_str();
                //println!("\x1B[{}m[{}][{}] {}\x1B[0m",color,record.level(),record.target(),record.args());
                //println!("{}",record.args());
                //RawScreen::into_raw_mode().unwrap();
                swap_write(
                    &terminal,
                    &cursor,
                    &format!("{}", record.args()),
                    &input_buf.lock().unwrap(),
                );
            }))
            .apply()
            .unwrap_or_else(|_| {
                error!("Global logger already set, amethyst-extra logger not used!")
            });
    }
}