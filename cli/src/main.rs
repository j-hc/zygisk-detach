use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek};
use std::io::{BufWriter, Read, Write};
use std::mem::size_of;
use std::ops::Range;
use std::panic::Location;
use std::process::{Command, ExitCode};

use termion::event::Key;
use termion::{clear, cursor, terminal_size};

mod colorize;
use colorize::ToColored;

mod menus;
use menus::Menus;

#[cfg(target_os = "android")]
const MODULE_DETACH: &str = "/data/adb/zygisk-detach/detach.bin";
#[cfg(target_os = "android")]
const DETACH_TXT: &str = "/data/adb/modules/zygisk-detach/detach.txt";

#[cfg(target_os = "linux")]
const MODULE_DETACH: &str = "detach.bin";
#[cfg(target_os = "linux")]
const DETACH_TXT: &str = "detach.txt";

struct LocErr<E: Error> {
    source: E,
    loc: &'static Location<'static>,
}
impl<E: Error> Error for LocErr<E> {}
impl<E: Error> Debug for LocErr<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\r\nat {}", self.source, self.loc)
    }
}
impl<E: Error> Display for LocErr<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl From<io::Error> for LocErr<io::Error> {
    #[track_caller]
    fn from(err: io::Error) -> Self {
        Self {
            source: err,
            loc: Location::caller(),
        }
    }
}
type IOResult<T> = Result<T, LocErr<io::Error>>;

fn main() -> ExitCode {
    std::panic::set_hook(Box::new(|panic| {
        let mut stderr = io::stderr();
        let _ = writeln!(stderr, "\r\n{panic}\r\n");
        let _ = writeln!(stderr, "This should not have happened.");
        let _ = writeln!(
            stderr,
            "Report at https://github.com/j-hc/zygisk-detach/issues"
        );
    }));

    if !std::path::Path::new("/data/adb/zygisk-detach/").exists() {
        std::fs::create_dir("/data/adb/zygisk-detach/").expect("zygisk-detach path");
    }

    let mut args = std::env::args().skip(1);
    if let Some(cmd) = args.next().as_deref() {
        match cmd {
            "serialize" => {
                let Some(dtxt) = args.next() else {
                    eprintln!("ERROR: detach.txt path not supplied.");
                    return ExitCode::FAILURE;
                };
                let Some(dbin) = args.next() else {
                    eprintln!("ERROR: detach.bin path not supplied.");
                    return ExitCode::FAILURE;
                };
                if let Err(err) = serialize_txt(&dtxt, &dbin) {
                    eprintln!("ERROR: {err}");
                    return ExitCode::FAILURE;
                }
                println!("Serialized detach.txt");
                return ExitCode::SUCCESS;
            }
            "detachall" => {
                if args.len() == 0 {
                    eprintln!("ERROR: No Package name(s) was supplied.");
                    return ExitCode::FAILURE;
                }
                let mut f = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(MODULE_DETACH)
                    .expect("open detach.bin");
                for pkg_name in args {
                    bin_serialize(&pkg_name, &mut f).expect("write to detach.bin");
                }
                detach_bin_changed();
                println!("Changes are applied. No need for a reboot!");
                return ExitCode::SUCCESS;
            }
            "detach" => {
                if args.len() == 0 {
                    eprintln!("ERROR: No Package name(s) was supplied.");
                    return ExitCode::FAILURE;
                }
                for pkg_name in args {
                    if !detach_by_name(&pkg_name).expect("detach.txt") {
                        println!("already detached: {}", pkg_name);
                    }
                }
                println!("Changes are applied. No need for a reboot!");
                return ExitCode::SUCCESS;
            }
            "reset" => {
                match fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(MODULE_DETACH)
                {
                    Ok(_) => detach_bin_changed(),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        eprintln!("ERROR: Could not delete '{MODULE_DETACH}': {e}");
                        return ExitCode::FAILURE;
                    }
                }
                return ExitCode::SUCCESS;
            }
            "reattach" => {
                let Some(pkg_name) = args.next() else {
                    eprintln!("ERROR: package name not supplied.");
                    return ExitCode::FAILURE;
                };
                if reattach_by_name(&pkg_name).expect("detach.txt") {
                    println!("re-attached: {}", pkg_name);
                }
                return ExitCode::SUCCESS;
            }
            "list" => {
                let mut detach_txt = match fs::OpenOptions::new()
                    .write(true)
                    .read(true)
                    .open(MODULE_DETACH)
                {
                    Ok(f) => f,
                    Err(e) if e.kind() == io::ErrorKind::NotFound => return ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("ERROR: Could not list detached pkgs: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let mut content = Vec::new();
                match detach_txt.read_to_end(&mut content) {
                    Ok(0) => return ExitCode::SUCCESS,
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("ERROR: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                for (app, _) in get_detached_apps(&content) {
                    println!("{app}");
                }
                return ExitCode::SUCCESS;
            }
            s => {
                eprintln!(
                    "\
Unexpected command: {s}
Accepted commands are: 'detach', 'detachall', 'reset', 'list', 'serialize'"
                );
                return ExitCode::FAILURE;
            }
        }
    }

    #[cfg(target_os = "android")]
    let _ = Command::new("magisk")
        .args(["--denylist", "rm", "com.android.vending"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .and_then(|mut p| p.wait());

    let mut menus = Menus::new();
    let ret = match interactive(&mut menus) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("\rERROR: {err}");
            ExitCode::FAILURE
        }
    };
    menus.cursor_show().unwrap();
    ret
}

fn detach_bin_changed() {
    let _ = fs::remove_file(DETACH_TXT);
    let _ = kill_store();
}

fn serialize_txt(txt: &str, bin: &str) -> IOResult<()> {
    let mut detach_bin = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(bin)?;
    for app in std::fs::read_to_string(txt)?
        .lines()
        .map(|s| s.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        println!("  '{}'", app);
        bin_serialize(app, &mut detach_bin)?;
    }
    Ok(())
}

fn interactive(menus: &mut Menus) -> IOResult<()> {
    menus.cursor_hide()?;
    print!("zygisk-detach cli by github.com/j-hc\r\n\n");
    loop {
        match main_menu(menus)? {
            Op::DetachSelect => detach_menu(menus)?,
            Op::ReattachSelect => reattach_menu(menus)?,
            Op::Reset => {
                if fs::remove_file(MODULE_DETACH).is_ok() {
                    let _ = kill_store();
                    text!(menus, "Reset");
                } else {
                    text!(menus, "Already empty");
                }
            }
            Op::CopyToSd => {
                #[cfg(target_os = "android")]
                const SDCARD_DETACH: &str = "/sdcard/detach.bin";
                #[cfg(target_os = "linux")]
                const SDCARD_DETACH: &str = "detach_sdcard.bin";
                match fs::copy(MODULE_DETACH, SDCARD_DETACH) {
                    Ok(_) => text!(menus, "Copied"),
                    Err(err) if err.kind() == io::ErrorKind::NotFound => {
                        text!(menus, "detach.bin not found");
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            Op::Quit => return Ok(()),
            Op::Nop => {}
        }
    }
}

fn reattach_menu(menus: &mut Menus) -> IOResult<()> {
    let mut detach_txt = match fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(MODULE_DETACH)
    {
        Ok(v) => v,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            text!(menus, "detach.bin not found");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    let mut content = Vec::new();
    detach_txt.read_to_end(&mut content)?;
    detach_txt.seek(io::SeekFrom::Start(0))?;
    let detached_apps = get_detached_apps(&content);
    let detach_len = detached_apps.len();
    if detach_len == 0 {
        text!(menus, "detach.bin is empty");
        return Ok(());
    }
    let list = detached_apps.iter().map(|e| e.0.as_str());
    let Some(i) = menus.select_menu(
        list,
        "Select the app to re-attach ('q' to leave):",
        "✖".red(),
        Some(Key::Char('q')),
    )?
    else {
        return Ok(());
    };

    textln!(menus, "{}: {}", "re-attach".red(), detached_apps[i].0);
    content.drain(detached_apps[i].1.clone());
    detach_txt.set_len(0)?;
    detach_txt.write_all(&content)?;
    detach_bin_changed();
    Ok(())
}

fn reattach_by_name(pkg_name: &str) -> IOResult<bool> {
    let mut detach_txt = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(MODULE_DETACH)?;
    let mut content = Vec::new();
    detach_txt.read_to_end(&mut content)?;
    detach_txt.seek(io::SeekFrom::Start(0))?;
    let detached_apps = get_detached_apps(&content);
    let Some(reattach_app) = detached_apps.iter().find(|(name, _)| name == pkg_name) else {
        return Ok(false);
    };
    content.drain(reattach_app.1.clone());
    detach_txt.set_len(0)?;
    detach_txt.write_all(&content)?;
    detach_bin_changed();
    Ok(true)
}

fn get_detached_apps(detach_txt: &[u8]) -> Vec<(String, Range<usize>)> {
    let mut i = 0;
    let mut detached = Vec::new();
    while i < detach_txt.len() {
        let len: u8 = detach_txt[i];
        const SZ_LEN: usize = size_of::<u8>();
        i += SZ_LEN;
        let Some(encoded_name) = &detach_txt.get(i..i + len as usize) else {
            eprintln!("Corrupted detach.bin. Reset and try again.");
            std::process::exit(1);
        };
        let name = String::from_utf8(encoded_name.iter().step_by(2).cloned().collect()).unwrap();
        detached.push((name, i - SZ_LEN..i + len as usize));
        i += len as usize;
    }
    detached
}

#[cfg(target_os = "linux")]
fn get_installed_apps() -> IOResult<Vec<u8>> {
    Ok("package:com.app1\npackage:org.xxx2\ncom.apppppppp.tooolonnggggtooolonnggggtooolonnggggtooolonngggg".as_bytes().to_vec())
}

#[cfg(target_os = "android")]
fn get_installed_apps() -> IOResult<Vec<u8>> {
    let op = Command::new("pm")
        .args(["list", "packages"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;
    if !op.status.success() {
        panic!("pm: '{}'", String::from_utf8_lossy(&op.stderr));
    }
    Ok(op.stdout)
}

#[derive(Clone, Copy)]
enum Op {
    DetachSelect,
    ReattachSelect,
    Reset,
    CopyToSd,
    Quit,
    Nop,
}

fn main_menu(menus: &mut Menus) -> IOResult<Op> {
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
        OpText::new("Detach", Op::DetachSelect),
        OpText::new("Re-attach", Op::ReattachSelect),
        OpText::new("Reset detached apps", Op::Reset),
        OpText::new("Copy detach.bin to /sdcard", Op::CopyToSd),
    ];
    let i = menus.select_menu_numbered(ops.iter(), Key::Char('q'), "- Selection:")?;
    use menus::SelectNumberedResp as SN;
    match i {
        SN::Index(i) => Ok(ops[i].op),
        SN::UndefinedKey(Key::Char(c)) => {
            text!(menus, "Undefined key {c:?}");
            Ok(Op::Nop)
        }
        SN::UndefinedKey(k @ (Key::Down | Key::Up | Key::Left | Key::Right)) => {
            text!(menus, "Undefined key {k:?}");
            Ok(Op::Nop)
        }
        SN::Quit => Ok(Op::Quit),
        _ => Ok(Op::Nop),
    }
}

fn bin_serialize(app: &str, f: &mut File) -> IOResult<()> {
    let mut w = Vec::with_capacity(2 * app.len() - 1);
    for b in app.as_bytes()[..app.len() - 1].iter().cloned() {
        w.push(b);
        w.push(0);
    }
    w.push(app.as_bytes()[app.len() - 1]);
    let mut f = BufWriter::new(f);
    f.write_all(&[w
        .len()
        .try_into()
        .expect("app name cannot be longer than 255")])?;
    f.write_all(&w)?;
    f.flush()?;
    Ok(())
}

fn detach_menu(menus: &mut Menus) -> IOResult<()> {
    let installed_apps = get_installed_apps()?;
    assert_ne!(installed_apps.len(), 0);
    let apps: Vec<&str> = installed_apps[..installed_apps.len() - 1]
        .split(|&e| e == b'\n')
        .map(|e| {
            e.get("package:".len()..)
                .expect("unexpected output from pm")
        })
        .map(|e| std::str::from_utf8(e).expect("non utf-8 package names?"))
        .collect();
    menus.cursor_show()?;
    let col = terminal_size().expect("could not get terminal size").0 as usize - 2;
    let selected = menus.select_menu_with_input(
        |input| {
            let input = input.trim();
            if !input.is_empty() {
                apps.iter()
                    .filter(|app| {
                        app.to_ascii_lowercase()
                            .contains(&input.to_ascii_lowercase())
                    })
                    .take(5)
                    .map(|s| &s[..col.min(s.len())])
                    .collect()
            } else {
                Vec::new()
            }
        },
        "↪".green(),
        "- app: ",
        None,
    )?;
    menus.cursor_hide()?;
    if let Some(detach_app) = selected {
        if detach_by_name(detach_app)? {
            textln!(menus, "{} {}", "detach:".green(), detach_app);
            textln!(menus, "Changes are applied. No need for a reboot!");
        } else {
            textln!(menus, "{} {}", "already detached:".green(), detach_app);
        }
    }
    Ok(())
}

fn detach_by_name(detach_app: &str) -> IOResult<bool> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(MODULE_DETACH)?;
    let mut buf: Vec<u8> = Vec::new();
    f.read_to_end(&mut buf)?;
    if !get_detached_apps(&buf).iter().any(|(s, _)| s == detach_app) {
        bin_serialize(detach_app, &mut f)?;
        detach_bin_changed();
        Ok(true)
    } else {
        Ok(false)
    }
}

fn _kill_store_am() -> IOResult<()> {
    Command::new("am")
        .args(["force-stop", "com.android.vending"])
        .spawn()?
        .wait()?;
    Ok(())
}

fn kill_store() -> IOResult<()> {
    extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }

    const PKG: &[u8] = b"com.android.vending";
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
        if !cmdline.read(&mut buf).is_ok_and(|n| n > 0) {
            continue;
        }
        if buf.eq(PKG) {
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
