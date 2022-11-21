use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;

pub fn read_file(file_path: &str) -> std::io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data)
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
