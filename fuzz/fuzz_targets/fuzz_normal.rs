#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
use diffutilslib::normal_diff;

use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

fuzz_target!(|x: (Vec<u8>, Vec<u8>)| {
    let (from, to) = x;
    /*if let Ok(s) = String::from_utf8(from.clone()) {
        if !s.is_ascii() { return }
        if s.find(|x| x < ' ' && x != '\n').is_some() { return }
    } else {
        return
    }
    if let Ok(s) = String::from_utf8(to.clone()) {
        if !s.is_ascii() { return }
        if s.find(|x| x < ' ' && x != '\n').is_some() { return }
    } else {
        return
    }*/
    let diff = normal_diff::diff(&from, &to);
    File::create("target/fuzz.file.original")
        .unwrap()
        .write_all(&from)
        .unwrap();
    File::create("target/fuzz.file.expected")
        .unwrap()
        .write_all(&to)
        .unwrap();
    File::create("target/fuzz.file")
        .unwrap()
        .write_all(&from)
        .unwrap();
    File::create("target/fuzz.diff")
        .unwrap()
        .write_all(&diff)
        .unwrap();
    let output = Command::new("patch")
        .arg("-p0")
        .arg("--binary")
        .arg("--fuzz=0")
        .arg("--normal")
        .arg("target/fuzz.file")
        .stdin(File::open("target/fuzz.diff").unwrap())
        .output()
        .unwrap();
    if !output.status.success() {
        panic!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let result = fs::read("target/fuzz.file").unwrap();
    if result != to {
        panic!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
});
