use crate::colorize::ToColored;
use std::fmt::Display;
use std::io::{self, BufWriter, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor, event::Key};

#[macro_export]
macro_rules! text {
    ($($arg:tt)*) => {{
        write!(
            ::std::io::stdout(),
            "{}{}{}{}\r",
            cursor::Up(1),
            clear::CurrentLine,
            format_args!($($arg)*),
            cursor::Down(1)
        )?;
        ::std::io::stdout().flush().unwrap();
    }};
}

#[macro_export]
macro_rules! textln {
    ($($arg:tt)*) => {{
        text!("{}\n", format_args!($($arg)*));
    }};
}

pub fn select_menu<L: Display, I: IntoIterator<Item = L> + Clone>(
    list: I,
    prompt: impl Display,
    quit: Option<Key>,
) -> io::Result<Option<usize>> {
    let mut stdout = BufWriter::new(io::stdout().lock().into_raw_mode()?);
    let mut select_idx = 0;
    let list_len = list.clone().into_iter().count();
    let mut keys = io::stdin().lock().keys();
    write!(stdout, "{}", cursor::Hide)?;
    let ret = loop {
        for (i, selection) in list.clone().into_iter().enumerate() {
            if i == select_idx {
                write!(stdout, "{} {}\r\n", prompt, selection.black().white_bg())?;
            } else {
                write!(stdout, "{}\r\n", selection.faint())?;
            }
        }
        stdout.flush()?;

        let key = keys
            .next()
            .expect("keys() should block")
            .expect("faulty keyboard?");
        write!(
            stdout,
            "\r{}{}",
            cursor::Up(list_len as u16),
            clear::AfterCursor
        )?;
        match key {
            Key::Char('\n') => {
                break Ok(Some(select_idx));
            }
            Key::Up => select_idx = select_idx.saturating_sub(1),
            Key::Down => {
                if select_idx + 1 < list_len {
                    select_idx += 1;
                }
            }
            k if k == Key::Ctrl('c') || quit.is_some_and(|q| q == key) => {
                break Ok(None);
            }
            _ => {}
        }
    };
    write!(stdout, "{}", cursor::Show)?;
    stdout.flush()?;
    ret
}

pub fn select_menu_with_input<L: Display>(
    lister: impl Fn(&str) -> Vec<L>,
    prompt: impl Display,
    input_prompt: &str,
    quit: Option<Key>,
) -> io::Result<Option<L>> {
    let mut stdout = BufWriter::new(io::stdout().lock().into_raw_mode()?);
    let mut select_idx = 0;
    let mut cursor = 0;
    let mut input = String::new();

    let mut keys = io::stdin().lock().keys();
    let ret = loop {
        write!(
            stdout,
            "\r{}{}{}",
            clear::AfterCursor,
            input_prompt.magenta(),
            input,
        )?;
        let mut list = lister(&input);
        let list_len = list.len();

        select_idx = select_idx.min(list_len);
        if list_len > 0 {
            write!(stdout, "\r\n\n↑ and ↓ to navigate")?;
            write!(stdout, "\n\rENTER to select\r\n")?;
        }

        for (i, selection) in list.iter().enumerate() {
            if i == select_idx {
                write!(stdout, "{} {}\r\n", prompt, selection.black().white_bg())?;
            } else {
                write!(stdout, "{}\r\n", selection.faint())?;
            }
        }
        if list_len > 0 {
            write!(stdout, "{}", cursor::Up(list_len as u16 + 4))?;
        }
        write!(
            stdout,
            "\r{}",
            cursor::Right(input_prompt.len() as u16 + cursor as u16)
        )?;
        stdout.flush()?;
        write!(stdout, "\r{}", clear::AfterCursor)?;

        match keys
            .next()
            .expect("keys() should block")
            .expect("faulty keyboard?")
        {
            Key::Char('\n') => {
                break Ok(if list_len > select_idx {
                    Some(list.remove(select_idx))
                } else {
                    None
                })
            }
            Key::Up => select_idx = select_idx.saturating_sub(1),
            Key::Down => {
                if select_idx + 1 < list_len {
                    select_idx += 1;
                }
            }
            Key::Backspace => {
                if cursor > 0 {
                    cursor -= 1;
                    input.remove(cursor);
                }
            }
            Key::Char(c) => {
                input.insert(cursor, c);
                cursor += 1;
            }
            Key::Right => {
                if cursor < input.len() {
                    cursor += 1
                }
            }
            Key::Left => cursor = cursor.saturating_sub(1),
            k if k == Key::Ctrl('c') || quit.is_some_and(|q| q == k) => {
                break Ok(None);
            }
            _ => {}
        }
    };
    stdout.flush()?;
    ret
}

pub enum SelectNumberedResp {
    Index(usize),
    UndefinedKey(Key),
    Quit,
}

pub fn select_menu_numbered<L: Display, I: IntoIterator<Item = L> + Clone>(
    list: I,
    quit: Option<Key>,
    title: &str,
) -> io::Result<SelectNumberedResp> {
    let mut stdout = BufWriter::new(io::stdout().lock().into_raw_mode()?);
    let list_len = list.clone().into_iter().count();
    write!(stdout, "\r{title}\r\n")?;
    write!(stdout, "{}", cursor::Hide)?;
    for (i, s) in list.clone().into_iter().enumerate() {
        write!(stdout, "{}. {}\r\n", (i + 1).green(), s)?;
    }
    stdout.flush()?;
    let key = io::stdin()
        .lock()
        .keys()
        .next()
        .expect("keys() should block")
        .expect("faulty keyboard?");
    write!(
        stdout,
        "\r{}{}{}",
        cursor::Up(list_len as u16 + 1),
        clear::AfterCursor,
        cursor::Show
    )?;
    stdout.flush()?;
    match key {
        Key::Char(c) if c.to_digit(10).is_some_and(|c| c as usize <= list_len) => Ok(
            SelectNumberedResp::Index(c.to_digit(10).unwrap() as usize - 1),
        ),
        k if k == Key::Ctrl('c') || quit.is_some_and(|q| q == key) => Ok(SelectNumberedResp::Quit),
        k => Ok(SelectNumberedResp::UndefinedKey(k)),
    }
}
