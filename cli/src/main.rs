#![feature(iter_intersperse)]

use std::fs;
use std::io::{self, Seek};
use std::io::{BufWriter, Read, Write};
use std::ops::Range;
use std::process::{Command, Stdio};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

mod colorize;
use colorize::ToColored;

#[cfg(target_os = "android")]
const SDCARD_DETACH: &str = "/sdcard/detach.bin";
#[cfg(target_os = "android")]
const MODULE_DETACH: &str = "/data/adb/modules/zygisk-detach/detach.bin";

#[cfg(target_os = "linux")]
const SDCARD_DETACH: &str = "detach.bin";
#[cfg(target_os = "linux")]
const MODULE_DETACH: &str = "detach_module.txt";

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => eprintln!("\rERROR: {err}"),
    }
}

fn copy_detach() -> io::Result<u64> {
    fs::copy(SDCARD_DETACH, MODULE_DETACH)
}

fn run() -> io::Result<()> {
    let mut stdout = BufWriter::new(io::stdout().lock().into_raw_mode()?);
    write!(stdout, "zygisk-detach cli by github.com/j-hc\r\n\n")?;
    loop {
        match main_menu(&mut stdout)? {
            Op::Select => select(&mut stdout)?,
            Op::Remove => remove_menu(&mut stdout)?,
            Op::Reset => {
                let d1 = fs::remove_file(SDCARD_DETACH);
                let d2 = fs::remove_file(MODULE_DETACH);
                if d1.is_ok() || d2.is_ok() {
                    let _ = kill_store();
                    write!(stdout, "Reset\r\n")?;
                } else {
                    write!(stdout, "Already empty\r\n")?;
                }
            }
            Op::Wrong(c) => write!(stdout, "Wrong selection {c:?}\r\n")?,
            Op::Quit => return Ok(()),
            Op::Nop => {}
        }
    }
}

fn remove_menu(mut stdout: impl Write) -> io::Result<()> {
    let openf = |p| fs::OpenOptions::new().write(true).read(true).open(p);
    let Ok(mut detach_txt) = openf(SDCARD_DETACH).or_else(|_| openf(MODULE_DETACH)) else {
        write!(stdout, "No detach.bin was found!\r\n")?;
        return Ok(());
    };
    let mut content = Vec::new();
    detach_txt.read_to_end(&mut content)?;
    detach_txt.seek(io::SeekFrom::Start(0))?;
    let detached_apps = get_detached_apps(&content);
    let detach_len = detached_apps.len();
    if detach_len == 0 {
        write!(stdout, "detach.bin is empty\r\n")?;
        return Ok(());
    }

    write!(stdout, "{}", cursor::Hide)?;
    let mut keys = io::stdin().lock().keys().flatten();
    let mut app_i: usize = 0;
    write!(stdout, "\nSelect the app to re-attach ('q' to leave):\r\n")?;
    loop {
        for (i, app) in detached_apps.iter().map(|m| &m.0).enumerate() {
            if i == app_i {
                write!(stdout, "{} {}\r\n", "✖".red(), app.black().white_bg())?;
            } else {
                write!(stdout, "{}\r\n", app)?;
            }
        }
        macro_rules! reset {
            ($up: expr) => {
                write!(stdout, "\r{}{}", cursor::Up($up as u16), clear::AfterCursor)?
            };
        }
        stdout.flush()?;

        match keys.next().unwrap() {
            Key::Char('q') | Key::Ctrl('c') => {
                reset!(detach_len + 1);
                return Ok(());
            }
            Key::Char('\n') => {
                reset!(detach_len + 1);
                write!(
                    stdout,
                    "{}: {}\r\n",
                    "re-attach".red(),
                    detached_apps[app_i].0
                )?;
                content.drain(detached_apps[app_i].1.clone());
                detach_txt.set_len(0)?;
                detach_txt.write_all(&content)?;
                copy_detach()?;
                return Ok(());
            }
            Key::Up => app_i = app_i.saturating_sub(1),
            Key::Down => {
                if app_i + 1 < detach_len {
                    app_i += 1;
                }
            }
            _ => {}
        }
        reset!(detach_len);
    }
}

fn get_detached_apps(detach_txt: &[u8]) -> Vec<(String, Range<usize>)> {
    let mut i = 0;
    let mut detached = Vec::new();
    while i < detach_txt.len() {
        let len = u32::from_le_bytes(detach_txt[i..i + 4].try_into().unwrap()) as usize;
        i += 4;
        let encoded_name = &detach_txt[i..i + len];
        let name = String::from_utf8(encoded_name.iter().step_by(2).cloned().collect()).unwrap();
        detached.push((name, i - 4..i + len));
        i += len;
    }
    detached
}

fn get_installed_apps() -> io::Result<String> {
    let c = Command::new("pm")
        .args(["list", "packages"])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut s = String::new();
    c.stdout.unwrap().read_to_string(&mut s)?;
    Ok(s)
}

#[derive(Clone, Copy)]
enum Op {
    Reset,
    Select,
    Remove,
    Quit,
    Nop,
    Wrong(char),
}
fn main_menu(mut stdout: impl Write) -> io::Result<Op> {
    struct OpText {
        desc: &'static str,
        op: Op,
        c: char,
    }
    impl OpText {
        fn new(desc: &'static str, op: Op, c: char) -> Self {
            Self { desc, op, c }
        }
    }
    let ops = [
        OpText::new("Select app to detach", Op::Select, '1'),
        OpText::new("Remove detached app", Op::Remove, '2'),
        OpText::new("Reset detached apps", Op::Reset, '3'),
        OpText::new("Quit", Op::Quit, 'q'),
    ];
    write!(stdout, "{}", cursor::Hide)?;
    write!(stdout, "- Selection: \r\n")?;
    for optxt in ops.iter() {
        write!(stdout, "{}. {}\r\n", optxt.c.green(), optxt.desc)?;
    }
    writeln!(stdout)?;
    stdout.flush()?;

    let key = io::stdin().keys().next().unwrap()?;

    let op = ops
        .iter()
        .find(|optxt| matches!(key, Key::Char(c) if c == optxt.c))
        .map(|optxt| optxt.op)
        .or(match key {
            Key::Char(c) => Some(Op::Wrong(c)),
            Key::Ctrl('c') => Some(Op::Quit),
            _ => Some(Op::Nop),
        })
        .unwrap();

    let up = if matches!(op, Op::Nop) {
        2 + ops.len() as u16
    } else {
        3 + ops.len() as u16
    };
    write!(
        stdout,
        "\r{}{}{}",
        cursor::Up(up),
        clear::AfterCursor,
        cursor::Show
    )?;
    stdout.flush()?;

    Ok(op)
}

fn select(mut stdout: impl Write) -> io::Result<()> {
    let installed_apps = get_installed_apps()?;
    let apps: Vec<&str> = installed_apps
        .lines()
        .filter_map(|line| line.get("package:".len()..))
        .collect();

    if let Some(detach_app) = select_menu(&apps, &mut stdout)? {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(SDCARD_DETACH)?;
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf)?;
        if !get_detached_apps(&buf).iter().any(|d| d.0 == detach_app) {
            let w = detach_app
                .as_bytes()
                .iter()
                .intersperse(&0)
                .cloned()
                .collect::<Vec<u8>>();
            let mut f = BufWriter::new(f);
            f.write_all(&(w.len() as u32).to_le_bytes())?;
            f.write_all(&w)?;
            f.flush()?;
            let _ = kill_store();
            write!(stdout, "Changes are applied. No need for a reboot!\r\n\n")?;
            copy_detach()?;
        } else {
            write!(stdout, "App is already detached\r\n\n")?;
        }
        stdout.flush()?;
    }
    Ok(())
}

fn select_menu<'a>(apps: &[&'a str], mut stdout: impl Write) -> Result<Option<&'a str>, io::Error> {
    const PROMPT: &str = "- app: ";
    let mut input_cursor = 0;
    let mut found_i = 0;
    // let mut input = UTF32String { inner: Vec::new() };
    let mut input = String::new();
    let mut keys = io::stdin().lock().keys().flatten();
    write!(stdout, "\n\n{}", cursor::Up(2))?;
    loop {
        write!(
            stdout,
            "\r{}{}{}\r{}{}",
            clear::AfterCursor,
            PROMPT.magenta(),
            input,
            cursor::Right(PROMPT.len() as u16 + input_cursor as u16),
            cursor::Save
        )?;

        fn find_app<'a>(apps: &[&'a str], input: &str) -> [Option<&'a str>; 5] {
            let mut ret = [None; 5];
            if input.len() <= 2 {
                return ret;
            }
            apps.iter()
                .filter(|app| app.contains(input))
                .take(5)
                .enumerate()
                .for_each(|(i, app)| ret[i] = Some(app));
            ret
        }
        let found_app = find_app(apps, &input);

        if found_app.iter().flatten().count() > 0 {
            write!(stdout, "\r\n\n↑ and ↓ to navigate")?;
            write!(stdout, "\n\rENTER to select\r\n")?;
        }
        for (i, app) in found_app.iter().flatten().enumerate() {
            if i == found_i {
                write!(stdout, "{} {}\r\n", "↪".green(), app.black().white_bg())?;
            } else {
                write!(stdout, "{}\r\n", app.faint())?;
            }
        }
        write!(stdout, "{}", cursor::Restore)?;
        stdout.flush()?;

        let Some(key) = keys.next() else {
            return Ok(None);
        };
        match key {
            Key::Char('\n') => {
                write!(stdout, "{}\r{}", cursor::Restore, clear::AfterCursor)?;
                match found_app[found_i] {
                    Some(found_app) => {
                        write!(stdout, "{} {}", "detached:".green(), found_app)?;
                    }
                    None => {
                        write!(stdout, "{}", "No app was selected".red())?;
                    }
                }
                write!(stdout, "\n\r")?;
                stdout.flush()?;
                return Ok(found_app[found_i]);
            }
            Key::Backspace => {
                if input_cursor > 0 {
                    input_cursor -= 1;
                    input.remove(input_cursor);
                }
            }
            Key::Char(c) => {
                input.insert(input_cursor, c);
                input_cursor += 1;
            }
            Key::Right => {
                if input_cursor < input.len() {
                    input_cursor += 1
                }
            }
            Key::Left => input_cursor = input_cursor.saturating_sub(1),
            Key::Up => found_i = found_i.saturating_sub(1),
            Key::Down => {
                if found_i < 4 && found_app[found_i + 1].is_some() {
                    found_i += 1;
                }
            }
            Key::Ctrl('c') => {
                write!(stdout, "{}\r{}", cursor::Restore, clear::AfterCursor)?;
                stdout.flush()?;
                return Ok(None);
            }
            _ => {}
        }
    }
}

fn _kill_store_am() -> io::Result<()> {
    Command::new("am")
        .args(["force-stop", "com.android.vending"])
        .spawn()?
        .wait()?;
    Ok(())
}

fn kill_store() -> io::Result<()> {
    const PKG: &str = "com.android.vending";
    let mut buf = [0u8; PKG.len()];
    for proc in fs::read_dir("/proc")? {
        let mut proc = proc?.path();
        if !proc.is_dir() {
            continue;
        }
        proc.push("cmdline");
        let Ok(mut cmdline) = fs::OpenOptions::new().read(true).open(&proc) else {
            continue;
        };
        match cmdline.read(&mut buf) {
            Ok(n) if n > 0 => {}
            _ => continue,
        }
        if buf.eq(PKG.as_bytes()) {
            if let Some(pid) = proc.components().nth(2) {
                let pid = pid.as_os_str().to_string_lossy();
                let Ok(pid) = pid.parse::<i32>() else {
                    continue;
                };
                unsafe { libc::kill(pid, libc::SIGKILL) };
            }
        }
    }
    Ok(())
}
