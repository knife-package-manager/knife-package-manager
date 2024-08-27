mod gitl;
mod info;
mod install;
mod list;
mod search;
use clap::Parser;

#[derive(Parser)]
#[command(version = "0.5")]
/// A simple, fast, and safe package manager
enum Cli {
    /// Update the package list
    Update,
    /// Upgrade the knife tool
    Upgrade,
    /// Lists the packages
    List {
        #[arg(short, long)]
        installed: bool,
    },
    /// Install a package
    Install {
        /// The package name (for install command)
        package: String,
    },
    Log,
}

fn main() {
    let version = info::VERSION;
    let args = Cli::parse();
    match args {
        Cli::Update => {
            gitl::update_package_list();
            std::process::exit(0);
        }
        Cli::Upgrade => {
            gitl::upgrade_knife(version.to_string());
        }
        Cli::Install { package } => {
            install::install(&package);
        }
        Cli::List { installed } => {
            if installed {
                list::list("bin", true);
            } else {
                list::list("packagelist", false);
            }
        }
        Cli::Log => {}
    }
}
