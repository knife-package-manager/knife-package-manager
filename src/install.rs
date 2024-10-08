// SPDX-License-Identifier: MIT
// Copyright (c) 2024 17do
// This software is licensed under the MIT License.

use crate::{install, log, search, Package};
use colored::*;
use dirs::home_dir;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
    process::{self, Command, ExitStatus},
};
#[derive(Debug, Deserialize, Serialize)]
struct PackageInfo {
    dependencies: Vec<String>,
    language: String,
    repository: String,
    capacity: i64,
    version: String,
    download: bool,
}

impl Package {
    /// ## get_package_infos
    ///
    /// load the package infos.
    /// this function is return the dependencies, capacity, version, repository, and language.
    /// this function returnd taple
    /// ```rust
    /// (String, String, String, String, String, bool)
    /// ```
    /// and return list is
    /// ```rust
    /// (language, capacity, version, dependencies, repository)
    /// ```
    pub fn get_package_infos(program: &str) -> (String, String, String, Vec<String>, String, bool) {
        let knife_home = home_dir()
            .expect("Failed to get ~/.comrade/")
            .join(".comrade/");
        let basepath = knife_home.join("packagelist/").join(program);
        let package = format!("{}/package.toml", basepath.display());
        fn open_and_read_file<P: AsRef<Path>>(path: P, read: &str, get: &str) -> String {
            match File::open(path) {
                Ok(mut f) => {
                    let mut result = String::new();
                    f.read_to_string(&mut result)
                        .unwrap_or_else(|_| panic!("failed to read {}", read));
                    result
                }
                Err(e) => {
                    eprintln!("{} Failed to get {}", ">>>".red().bold(), get);
                    eprintln!("Please report this issue to the comrade repository.");
                    eprintln!("Error code: {}", e);
                    std::process::exit(1);
                }
            }
        }

        let package_info = open_and_read_file(package, "package info", "packageinfo");

        let package_info: PackageInfo =
            toml::from_str(&package_info).unwrap_or_else(|_| panic!("failed to parse toml"));

        (
            package_info.language,
            package_info.capacity.to_string(),
            package_info.version,
            package_info.dependencies,
            package_info.repository,
            package_info.download,
        )
    }
    /// ## install
    ///
    /// this function is install the package
    ///
    /// ### Usage
    ///
    /// ```rust
    /// install::Package::install(_install_package_name);
    /// ```
    ///
    pub fn install(program: &str, source: bool, build: bool) {
        let _ = build;
        let search_ = search::search_program(program);
        let knife_home = home_dir()
            .expect("Failed to get ~/.comrade/")
            .join(".comrade/");
        let (lang, capa, ver, depen, github, download) = Package::get_package_infos(program);
        if search_ && !download || download && build {
            if home_dir()
                .expect("failed to get home")
                .join(".comrade/build")
                .exists()
            {
                println!(
                    "{} {}",
                    ">>>".green().bold(),
                    "removing build directory...".bold()
                );
                fs::remove_dir_all(
                    home_dir()
                        .expect("Failed to get home")
                        .join(".comrade/build"),
                )
                .expect("Failed to remove build directory");
            }

            println!("{} {}", ">>>".green().bold(), "Clone package...".bold());
            if Repository::clone(&github, knife_home.join("build")).is_err() {
                eprintln!("\n{}: Failed to Clone Repository.", "Error".red());
                eprintln!("Please report this issue to the comrade repository");
                std::process::exit(1);
            }
            let exe =
                install::get_program_name(knife_home.join("build/").display().to_string(), program);
            let exeit = knife_home.join("bin/").join(&exe);
            if exeit.exists() && !source {
                println!(
                    "{} {}",
                    ">>>".red().bold(),
                    "The program is already installed!".bold()
                );
                println!(
                    "For more information about this program, please visit {}",
                    github
                );
                fs::remove_dir_all(knife_home.join("build/")).expect("Failed to remove dir");
                std::process::exit(1);
            }
            println!("{} {}", "install package:".bold(), program);
            println!("{} {}", "executable file name:".bold(), exe);
            println!("{} {}bytes", "capacity:".bold(), capa);
            println!("{} {}", "language:".bold(), lang);
            println!("{} {}", "versions:".bold(), ver);
            println!("{} {:?}", "dependencies:".bold(), depen);
            println!("{} {}", "repository:".bold(), github);
            let mut tmp = String::new();
            let mut ok_ = "yes";
            if !source {
                println!("\n{} {}?", "install".bold(), program);
                print!("[y/n] ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut tmp).unwrap();
                ok_ = tmp.trim();
            }
            if ["y", "yes", ""].contains(&ok_) {
                // Start Installation
                println!("{} {}", ">>>".green().bold(), "Start Installation".bold());
                // install dependence
                if !depen.is_empty() {
                    println!("Installing Dependencies");
                    depen
                        .into_iter()
                        .for_each(|d| Package::install_for_dependence(&d, true));
                }
                println!("{} run install.sh (build start)", ">>>".yellow().bold());

                let status_installsh = process::Command::new("sh")
                    .arg(knife_home.join("build/install.sh"))
                    .current_dir(knife_home.join("build"))
                    .status()
                    .expect("Failed to start install.sh");
                if !status_installsh.success() {
                    println!(
                    "\n{} install.sh failed. Please report this problem to the comrade repository",
                    ">>>".red().bold()
                );
                    std::process::exit(1);
                }
                println!("{} {}", ">>>".cyan().bold(), "build end".bold());
                fs::rename(
                    knife_home.join("build/").join(&exe),
                    knife_home.join("bin/").join(&exe),
                )
                .expect("Failed to move file");
                fs::remove_dir_all(knife_home.join("build/")).expect("Failed to remove dir");
                println!("{} {}", ">>>".green().bold(), "Fill in the log...".bold());
                let _ = log::Name::new(&knife_home.join("log/install/")).create(
                    program,
                    exe.as_str(),
                    github.to_string(),
                    ver.to_string(),
                );
                if !source {
                    println!("{}", "All done!".bold());
                    println!("Installation is complete");
                    println!(
                        "For more information on {}, please see {}.",
                        program, github
                    );
                }
            }
        } else if download {
            let pkg = program;
            let (lang, capa, ver, depen, github, _download) = Package::get_package_infos(program);
            let exe = Package::download_get_execname(pkg).expect("Failed to get exec_name");
            let exeit = knife_home.join("bin/").join(&exe);
            if exeit.exists() && !source {
                println!(
                    "{} {}",
                    ">>>".red().bold(),
                    "The program is already installed!".bold()
                );
                println!(
                    "For more information about this program, please visit {}",
                    github
                );
                std::process::exit(1);
            }
            println!("{} {}", "install package:".bold(), program);
            println!("{} {}", "executable file name:".bold(), exe);
            println!("{} {}bytes", "capacity:".bold(), capa);
            println!("{} {}", "language:".bold(), lang);
            println!("{} {}", "versions:".bold(), ver);
            println!("{} {:?}", "dependencies:".bold(), depen);
            println!("{} {}", "repository:".bold(), github);
            let mut tmp = String::new();
            let mut ok_ = "yes";
            if !source {
                println!("\n{} {}?", "install".bold(), program);
                print!("[y/n] ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut tmp).unwrap();
                ok_ = tmp.trim();
            }
            if ["y", "yes", ""].contains(&ok_) {
                // install dependence
                if !depen.is_empty() {
                    println!("Installing Dependencies");
                    depen
                        .into_iter()
                        .for_each(|d| Package::install_for_dependence(&d, true));
                }
                let archive = Package::download_install(program);
                let _ = Package::unpack_package(archive.unwrap(), program);
                println!("{} {}", ">>>".green().bold(), "Fill in the log...".bold());
                let _ = log::Name::new(&knife_home.join("log/install")).create(
                    program,
                    Package::download_get_execname(pkg)
                        .expect("Failed to get exec_name")
                        .as_str(),
                    github.to_string(),
                    ver.to_string(),
                );
            } else {
                return;
            }
        }
    }
    pub fn install_for_dependence(program: &str, build: bool) {
        let _ = build;
        let search_ = search::search_program(program);
        let knife_home = home_dir()
            .expect("Failed to get ~/.comrade/")
            .join(".comrade/");
        let (_lang, _capa, ver, depen, github, download) = Package::get_package_infos(program);
        if search_ && !download || download && build {
            if knife_home.join("build/").join(program).exists() {
                fs::remove_dir_all(knife_home.join("build/").join(program)).unwrap();
            }
            let _ = fs::create_dir_all(knife_home.join("build/").join(program)).unwrap();

            println!("{} {}", ">>>".green().bold(), "Clone package...".bold());
            if let Err(e) = Repository::clone(&github, knife_home.join("build/").join(program)) {
                eprintln!("\n{}: Failed to Clone Repository.", "Error".red());
                eprintln!("Please report this issue to the comrade repository");
                eprintln!("Error code: {}", e);
                std::process::exit(1);
            }
            let exe = install::get_program_name(
                knife_home
                    .join("build/")
                    .join(program)
                    .display()
                    .to_string(),
                program,
            );
            let mut _cmd: ExitStatus;
            if cfg!(target_os = "windows") {
                _cmd = Command::new("where")
                    .arg(&exe)
                    .stderr(process::Stdio::null())
                    .stdout(process::Stdio::null())
                    .status()
                    .expect("Failed to which");
            } else {
                _cmd = Command::new("which")
                    .arg(&exe)
                    .stderr(process::Stdio::null())
                    .stdout(process::Stdio::null())
                    .status()
                    .expect("Failed to which");
            }
            let mut installed = false;
            if _cmd.success() {
                println!(
                    "{} {}",
                    ">>>".red().bold(),
                    "The program is already installed!".bold()
                );
                fs::remove_dir_all(knife_home.join("build/").join(program))
                    .expect("Failed to remove dir");
                installed = true
            }
            if !installed {
                println!("install package: {}", program);
                println!("dependencies: {:?}", depen);
                // install dependence
                if !depen.is_empty() && depen[0] != "" {
                    println!(
                        "{} {}",
                        ">>>".green().bold(),
                        "Install Dependencies...".bold()
                    );
                    let _ = depen
                        .into_iter()
                        .for_each(|d| Package::install_for_dependence(&d, true));
                }
                println!("{} run install.sh (build start)", ">>>".yellow().bold());
                let progmr = format!(
                    "{}{}",
                    knife_home.join("build/").join(program).display(),
                    "/install.sh"
                );
                let status_installsh = process::Command::new("sh")
                    .arg(progmr)
                    .current_dir(knife_home.join("build").join(program))
                    .status()
                    .expect("Failed to start install.sh");
                if !status_installsh.success() {
                    println!(
                    "\n{} install.sh failed. Please report this problem to the comrade repository",
                    ">>>".red().bold()
                );
                    std::process::exit(1);
                }
                println!("{} {}", ">>>".cyan().bold(), "build end".bold());
                fs::rename(
                    knife_home.join("build/").join(program).join(&exe),
                    knife_home.join("bin/").join(&exe),
                )
                .expect("Failed to move file");
                fs::remove_dir_all(knife_home.join("build/")).expect("Failed to remove dir");
                println!("{} {}", ">>>".green().bold(), "Fill in the log...".bold());
                let _ = log::Name::new(&knife_home.join("log/install/")).create(
                    program,
                    exe.as_str(),
                    github.to_string(),
                    ver.to_string(),
                );
            } else {
                return;
            }
        } else if download {
            let pkg = program;
            let (_lang, _capa, ver, depen, github, _download) = Package::get_package_infos(program);
            let exe = Package::download_get_execname(pkg).expect("Failed to get exec_name");
            let _exeit = knife_home.join("bin/").join(&exe);
            let mut _cmd = Command::new("").status().unwrap();
            if cfg!(target_os = "windows") {
                _cmd = Command::new("where")
                    .arg(&exe)
                    .stderr(process::Stdio::null())
                    .stdout(process::Stdio::null())
                    .status()
                    .expect("Failed to execute where");
            } else {
                _cmd = Command::new("which")
                    .arg(&exe)
                    .stderr(process::Stdio::null())
                    .stdout(process::Stdio::null())
                    .status()
                    .expect("Failed to execute which");
            }

            if _cmd.success() {
                println!(
                    "{} {}",
                    ">>>".red().bold(),
                    "The program is already installed!".bold()
                );
            }
            println!("{} {}", "install package:", program);
            println!("{} {:?}", "dependencies:", depen);
            // install dependence
            if !depen.is_empty() {
                println!("Installing Dependencies");
                depen
                    .into_iter()
                    .for_each(|d| Package::install_for_dependence(&d, true));
            }
            let archive = Package::download_install(program);
            let _ = Package::unpack_package(archive.unwrap(), program);
            println!("{} {}", ">>>".green().bold(), "Fill in the log...".bold());
            let _ = log::Name::new(&knife_home.join("log/install")).create(
                program,
                Package::download_get_execname(pkg)
                    .expect("Failed to get exec_name")
                    .as_str(),
                github.to_string(),
                ver.to_string(),
            );
        } else {
            return;
        }
    }
}

pub fn get_program_name(build_dir: String, program: &str) -> String {
    // build_dir
    let exe_name = Path::new(&build_dir).join(".comrade/exe_name");
    if !exe_name.exists() {
        return program.to_string();
    }
    let mut str = String::new();
    if let Ok(mut fl) = fs::File::open(&exe_name) {
        fl.read_to_string(&mut str).expect("failed to read file");
        return str.trim().to_string();
    } else {
        eprintln!("failed to read file: {}", exe_name.display());
        eprintln!("Please report this issue to the comrade repository");
        std::process::exit(1);
    }
}
