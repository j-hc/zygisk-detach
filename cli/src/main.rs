#![feature(iter_intersperse, print_internals)]

use std::fmt::Display;
use std::fs;
use std::io::{self, Seek};
use std::io::{BufWriter, Read, Write};
use std::mem::size_of;
use std::ops::Range;
use std::process::{Command, ExitCode};

use termion::event::Key;
use termion::{clear, cursor};

mod colorize;
use colorize::ToColored;

mod menus;
use menus::{select_menu, select_menu_numbered, select_menu_with_input};

#[cfg(target_os = "android")]
const SDCARD_DETACH: &str = "/sdcard/detach.bin";
#[cfg(target_os = "android")]
const MODULE_DETACH: &str = "/data/adb/modules/zygisk-detach/detach.bin";

#[cfg(target_os = "linux")]
const SDCARD_DETACH: &str = "detach.bin";
#[cfg(target_os = "linux")]
const MODULE_DETACH: &str = "detach_module.txt";

extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("\rERROR: {err}");
            ExitCode::FAILURE
        }
    }
}

fn detach_changed() -> io::Result<u64> {
    let _ = kill_store();
    fs::copy(SDCARD_DETACH, MODULE_DETACH)
}

fn run() -> io::Result<()> {
    print!("zygisk-detach cli by github.com/j-hc\r\n\n");
    loop {
        match main_menu()? {
            Op::DetachSelect => detach_menu()?,
            Op::ReattachSelect => reattach_menu()?,
            Op::Reset => {
                let d1 = fs::remove_file(SDCARD_DETACH);
                let d2 = fs::remove_file(MODULE_DETACH);
                if d1.is_ok() || d2.is_ok() {
                    let _ = kill_store();
                    text!("Reset");
                } else {
                    text!("Already empty");
                }
            }
            Op::Quit => return Ok(()),
            Op::Nop => {}
        }
    }
}

fn reattach_menu() -> io::Result<()> {
    let openf = |p| fs::OpenOptions::new().write(true).read(true).open(p);
    let Ok(mut detach_txt) = openf(SDCARD_DETACH).or_else(|_| openf(MODULE_DETACH)) else {
        text!("No detach.bin was found!");
        return Ok(());
    };
    let mut content = Vec::new();
    detach_txt.read_to_end(&mut content)?;
    detach_txt.seek(io::SeekFrom::Start(0))?;
    let detached_apps = get_detached_apps(&content);
    let detach_len = detached_apps.len();
    if detach_len == 0 {
        text!("detach.bin is empty");
        return Ok(());
    }
    text!("Select the app to re-attach ('q' to leave):");
    let list = detached_apps.iter().map(|e| e.0.as_str());
    let Some(i) = select_menu(list, "✖".red(), Some(Key::Char('q')))? else {
        return Ok(());
    };

    textln!("{}: {}", "re-attach".red(), detached_apps[i].0);
    content.drain(detached_apps[i].1.clone());
    detach_txt.set_len(0)?;
    detach_txt.write_all(&content)?;
    detach_changed()?;
    Ok(())
}

fn get_detached_apps(detach_txt: &[u8]) -> Vec<(String, Range<usize>)> {
    let mut i = 0;
    let mut detached = Vec::new();
    while i < detach_txt.len() {
        let len: u8 = detach_txt[i];
        const SZ_LEN: usize = size_of::<u8>();
        i += SZ_LEN;
        let encoded_name = &detach_txt[i..i + len as usize];
        let name = String::from_utf8(encoded_name.iter().step_by(2).cloned().collect()).unwrap();
        detached.push((name, i - SZ_LEN..i + len as usize));
        i += len as usize;
    }
    detached
}

#[cfg(target_os = "linux")]
fn get_installed_apps() -> io::Result<Vec<u8>> {
    Ok("package:com.app1\npackage:org.xxx2".as_bytes().to_vec())
}

#[cfg(target_os = "android")]
fn get_installed_apps() -> io::Result<Vec<u8>> {
    Ok(Command::new("pm")
        .args(["list", "packages"])
        .stdout(std::process::Stdio::piped())
        .output()?
        .stdout)
}

#[derive(Clone, Copy)]
enum Op {
    DetachSelect,
    ReattachSelect,
    Reset,
    Quit,
    Nop,
}

fn main_menu() -> io::Result<Op> {
    struct OpText {
        desc: &'static str,
        op: Op,
    }
    impl OpText {
        fn new(desc: &'static str, op: Op) -> Self {
            Self { desc, op }
        }
    }
    impl Display for OpText {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.desc)
        }
    }
    let ops = [
        OpText::new("Select app to detach", Op::DetachSelect),
        OpText::new("Re-attach app", Op::ReattachSelect),
        OpText::new("Reset detached apps", Op::Reset),
        OpText::new("Quit", Op::Quit),
    ];
    let i = select_menu_numbered(ops.iter(), Some(Key::Char('q')), "- Selection:")?;
    match i {
        menus::SelectNumberedResp::Index(i) => Ok(ops[i].op),
        menus::SelectNumberedResp::UndefinedKey(Key::Char(c)) => {
            text!("Undefined key {c:?}");
            Ok(Op::Nop)
        }
        menus::SelectNumberedResp::UndefinedKey(
            k @ (Key::Down | Key::Up | Key::Left | Key::Right),
        ) => {
            text!("Undefined key {k:?}");
            Ok(Op::Nop)
        }
        menus::SelectNumberedResp::Quit => Ok(Op::Quit),
        _ => Ok(Op::Nop),
    }
}

fn bin_serialize(app: &str, sink: impl Write) -> io::Result<()> {
    let w = app
        .as_bytes()
        .iter()
        .intersperse(&0)
        .cloned()
        .collect::<Vec<u8>>();
    let mut f = BufWriter::new(sink);
    f.write_all(std::slice::from_ref(
        &w.len()
            .try_into()
            .expect("app name cannot be longer than 255"),
    ))?;
    f.write_all(&w)?;
    f.flush()?;
    Ok(())
}

fn detach_menu() -> io::Result<()> {
    let installed_apps = get_installed_apps()?;
    let apps: Vec<&str> = installed_apps[..installed_apps.len() - 1]
        .split(|&e| e == b'\n')
        .map(|e| {
            e.get("package:".len()..)
                .expect("unexpected output from pm")
        })
        .map(|e| std::str::from_utf8(e).expect("non utf-8 package names?"))
        .collect();
    let selected = select_menu_with_input(
        |input| {
            if input.len() > 2 {
                apps.iter()
                    .filter(move |app| app.contains(input.trim()))
                    .take(5)
                    .collect()
            } else {
                Vec::new()
            }
        },
        "↪".green(),
        "- app: ",
        None,
    )?;
    if let Some(detach_app) = selected {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(SDCARD_DETACH)?;
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf)?;
        if !get_detached_apps(&buf).iter().any(|(s, _)| s == detach_app) {
            bin_serialize(detach_app, f)?;
            textln!("{} {}", "detach:".green(), detach_app);
            textln!("Changes are applied. No need for a reboot!");
            detach_changed()?;
        } else {
            textln!("{} {}", "already detached:".green(), detach_app);
        }
    }
    Ok(())
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
                unsafe { kill(pid, 9) };
            }
        }
    }
    Ok(())
}
