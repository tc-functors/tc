use human_bytes::human_bytes;
use path_absolutize::*;
use std::{
    env,
    fs,
    fs::File,
    io::{
        self,
        BufRead,
        BufReader,
        Read,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
    thread,
    time::Duration,
};
use subprocess::{
    CaptureData,
    Exec,
    Redirection,
};

pub fn basedir(path: &str) -> &str {
    let parts: Vec<&str> = path.split("/").collect();
    parts.clone().last().unwrap()
}

#[cfg(not(test))]
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

// Mocking this io function
// if path contains true it returns true else false
#[cfg(test)]
pub fn file_exists(path: &str) -> bool {
    if path.contains("true") {
        return true;
    } else {
        return false;
    }
}

pub fn file_size(path: &str) -> f64 {
    if file_exists(path) {
        let f = std::fs::metadata(path);
        let size: f64 = f.expect("Not found").len() as f64;
        size
    } else {
        0.0
    }
}

pub fn file_size_human(size: f64) -> String {
    human_bytes(size)
}

pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn list_dir(dir: &str) -> Vec<String> {
    if file_exists(dir) {
        let paths = fs::read_dir(dir).unwrap();
        let mut xs: Vec<String> = vec![];
        for path in paths {
            xs.push(path.unwrap().path().into_os_string().into_string().unwrap());
        }
        xs
    } else {
        vec![]
    }
}

pub fn list_dirs(dir: &str) -> Vec<String> {
    if is_dir(dir) {
        let paths = fs::read_dir(dir).unwrap();
        let mut xs: Vec<String> = vec![];
        for path in paths {
            let p = path.unwrap().path();
            if p.is_dir() {
                xs.push(p.into_os_string().into_string().unwrap());
            }
        }
        xs
    } else {
        vec![]
    }
}

pub fn pwd() -> String {
    match env::var("TC_DIR") {
        Ok(d) => d,
        Err(_) => env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap(),
    }
}

pub fn mkdir(path: &str) {
    fs::create_dir_all(path).unwrap();
}

pub fn write_str(path: &str, s: &str) {
    let mut f = File::create(path).unwrap();
    write!(&mut f, "{}", s).unwrap();
}

pub fn write_bytes(path: &str, ba: Vec<u8>) {
    let mut f = File::create(path).unwrap();
    f.write_all(&ba).unwrap();
}

pub fn read_bytes(path: &str) -> Vec<u8> {
    let f = File::open(path).unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    buffer
}

fn trim_all(input: &str) -> String {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix('\n'))
        .unwrap_or(input)
        .to_string()
}
pub fn readlines(filename: &str) -> Vec<String> {
    fs::read_to_string(filename)
        .unwrap()
        .lines()
        .map(trim_all)
        .collect()
}

#[cfg(not(test))]
pub fn slurp(path: &str) -> String {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    data
}

// Mocking this io function
// assumes path is already in the format of stringified hash map
// "{\"version\": \"0.0.1\"}"
#[cfg(test)]
pub fn slurp(path: &str) -> String {
    if path.contains("function.json") {
        "{
            \"name\": \"default_name\",
            \"namespace\": \"namespace\",
            \"version\": \"0.0.1\",
            \"runtime\": {
                \"lang\": \"python3.10\"
            }
        }"
        .to_string()
    } else {
        "{\"version\": \"0.0.1\"}".to_string()
    }
}

pub fn read_stdin() -> String {
    let mut data: String = "".to_string();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let m = line.unwrap();
        data.push_str(&m);
        data.push_str("\n");
    }
    data
}

pub fn read_stdin_vec() -> Vec<String> {
    let mut data: Vec<String> = vec![];

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let m = line.unwrap();
        data.push(m);
    }
    data
}

fn trim(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix('\n'))
        .unwrap_or(input)
}

#[cfg(not(test))]
pub fn sh(path: &str, dir: &str) -> String {
    let out = Exec::shell(path)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .cwd(dir)
        .capture();

    match out {
        Ok(s) => {
            let m = s.stdout_str();
            trim(&m).to_string()
        }
        Err(e) => {
            tracing::debug!("File not found {} dir: {}", path, dir);
            panic!("{}", e)
        }
    }
}

pub fn run(path: &str, dir: &str) {
    match std::env::var("TC_TRACE") {
        Ok(_) => {
            runcmd_stream(path, dir);
        }
        Err(_) => {
            sh(path, dir);
        }
    }
}

pub fn runcmd_quiet(path: &str, dir: &str) {
    let _ = Exec::shell(path)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .cwd(dir)
        .capture()
        .unwrap()
        .stdout_str();
}

pub fn tee(path: &str, dir: &str) {
    Exec::shell(path).cwd(dir).join().unwrap();
}

pub fn runcmd_stream(path: &str, dir: &str) {
    Exec::shell(path).cwd(dir).join().unwrap();
}

pub fn runc(path: &str, dir: &str) -> (bool, String, String) {
    match std::env::var("TC_TRACE") {
        Ok(_) => {
            tracing::debug!("runc {} {}", path, dir);
            let out = Exec::shell(path).cwd(dir).join().unwrap();
            (out.success(), String::from(""), String::from(""))
        }
        Err(_) => {
            let data = Exec::shell(path)
                .stdout(Redirection::Pipe)
                .stderr(Redirection::Merge)
                .env("TERM", "xterm")
                .cwd(dir)
                .capture()
                .unwrap();

            let CaptureData {
                stdout,
                stderr,
                exit_status,
            } = data;
            (
                exit_status.success(),
                String::from_utf8_lossy(&stdout).to_string(),
                String::from_utf8_lossy(&stderr).to_string(),
            )
        }
    }
}

pub fn runp(cmd: &str, dir: &str) -> bool {
    tracing::debug!(cmd);
    match std::env::var("TC_TRACE") {
        Ok(_) => {
            let out = Exec::shell(cmd).cwd(dir).join().unwrap();
            out.success()
        }
        Err(_) => {
            sh(cmd, dir);
            true
        }
    }
}

pub fn sleep(ms: u64) {
    let duration = Duration::from_millis(ms);
    thread::sleep(duration);
}

pub fn env_var(var: &str, fallback: &str) -> String {
    match env::var(var) {
        Ok(v) => v,
        Err(_e) => String::from(fallback),
    }
}

pub fn any_path(paths: Vec<String>) -> Option<String> {
    for path in paths {
        if file_exists(&path) {
            return Some(path);
        }
    }
    None
}

pub fn basename(path: &str) -> String {
    let mut pieces = path.rsplitn(2, |c| c == '/' || c == '\\');
    match pieces.next() {
        Some(p) => {
            let parts: Vec<&str> = p.split(".").collect();
            parts.clone().first().unwrap().to_string()
        }
        None => path.to_string(),
    }
}

pub fn absolutize(current_dir: &str, rel_path: &str) -> String {
    let p = Path::new(rel_path);
    p.absolutize_from(current_dir)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn path_of(dir: &str, f: &str) -> String {
    format!("{}/{}", dir, f)
}

#[cfg(not(test))]
pub fn path_exists(dir: &str, f: &str) -> bool {
    file_exists(&format!("{}/{}", dir, f))
}

// Mocking this io function
// if dir contains f it returns true else false
#[cfg(test)]
pub fn path_exists(dir: &str, f: &str) -> bool {
    if dir.contains(f) {
        return true;
    } else {
        return false;
    }
}

pub fn path_size(dir: &str, f: &str) -> f64 {
    file_size(&path_of(dir, f))
}

pub fn parent_dir(f: &str) -> String {
    let path = PathBuf::from(f);
    let dir = path.parent().unwrap();
    dir.to_str().unwrap().to_string()
}

pub fn dir_of(d: &str) -> String {
    let dir = format!("{}/", pwd());
    let parts: Vec<&str> = d.split(&dir).collect();
    parts.into_iter().nth(1).unwrap_or_default().to_string()
}

fn split_last(s: &str, delimiter: &str) -> String {
    let parts: Vec<&str> = s.split(delimiter).collect();
    parts.clone().last().unwrap().to_string()
}

pub fn absolute_dir(root_dir: &str, relative_dir: &str) -> String {
    let abs = absolutize(root_dir, relative_dir);
    if is_dir(&abs) {
        absolutize(root_dir, relative_dir)
    } else {
        let path = split_last(relative_dir, "../");
        format!("{}/{}", root_dir, path)
    }
}

pub fn gdir(dir: &str) -> String {
    let git_root = format!("{}/", root());
    let parts: Vec<&str> = dir.split(&git_root).collect();
    parts.clone().last().unwrap().to_string()
}

pub fn adir(dir: &str) -> String {
    absolute_dir(&root(), dir)
}

pub fn file_contains(path: &str, s: &str) -> bool {
    let data = slurp(path);
    data.contains(s)
}

pub fn pbufs(p: PathBuf) -> String {
    p.into_os_string().into_string().unwrap()
}

pub fn expand_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

pub fn run_seq(cmds: Vec<&str>, dir: &str) {
    for cmd in cmds {
        runcmd_stream(cmd, dir);
    }
}

pub fn run_seq_quiet(cmds: Vec<&str>, dir: &str) {
    for cmd in cmds {
        runcmd_quiet(cmd, dir);
    }
}

pub fn runv(dir: &str, cmd: Vec<&str>) {
    let cmd_str = cmd.join(" ");
    match std::env::var("TC_TRACE") {
        Ok(_) => runcmd_stream(&cmd_str, dir),
        Err(_) => {
            sh(&cmd_str, dir);
        }
    }
}

pub fn root() -> String {
    sh("git rev-parse --show-toplevel", &pwd())
}

pub fn roots() -> String {
    let (status, x, _) = runc("git rev-parse --show-toplevel", &pwd());
    if status {
        x
    } else {
        String::from(".")
    }
}
