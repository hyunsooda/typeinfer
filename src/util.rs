use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::process::Command;

pub fn read_file(filename: &str) -> std::io::Result<String> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data)
}

pub fn write_file(filename: &str, content: &str) {
    fs::write(filename, content).unwrap()
}

pub fn jscode2file(filename: &str, code: &str) {
    let mut f = File::create(filename).unwrap();
    f.write_all(code.as_bytes()).unwrap();
    Command::new("js-beautify")
        .arg("-r")
        .arg(filename)
        .output()
        .unwrap();
}
