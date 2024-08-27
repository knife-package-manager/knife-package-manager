use colored::*;
use std::{ffi::OsStr, fs};

// search package list
pub fn search_program(program: &String) -> bool {
    let dir_path = dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".comrade/packagelist");

    let dir = match fs::read_dir(&dir_path) {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!(
                "{}{}{}{}{}",
                "--Error--\n".red().bold(),
                "Failed to retrieve package list.\n".red().bold(),
                "please run ".red().bold(),
                "knife update ".cyan(),
                "to retrieve package list".red().bold()
            );
            return false;
        }
    };

    let mut found: bool = false;
    let mut ret: bool = false;
    for entry in dir.flatten() {
        if entry.file_name() == <String as AsRef<OsStr>>::as_ref(program) {
            found = true;
            let target = entry.path();
            if target.is_dir() {
                ret = true;
            } else {
                found = false;
            }

            break;
        }
    }

    if !found {
        println!("Program not found: {}", program);
        ret = false;
    }
    ret
}
