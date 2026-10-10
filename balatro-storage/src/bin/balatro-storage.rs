use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use balatro_storage::{PathOverrides, Platform, store};

#[derive(Parser)]
#[command(
    name = "balatro-storage",
    about = "Export and import Balatro save files on macOS Apple Arcade",
    version
)]
struct Cli {
    /// Override the path to the game's binary plist file
    #[arg(long, global = true, value_name = "PATH")]
    plist: Option<PathBuf>,

    /// Override the path to the game's support directory (contains save.jkr, settings.jkr)
    #[arg(long, global = true, value_name = "PATH")]
    game_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Export save files from the game to a directory
    Export {
        /// Profile(s) to export
        #[arg(long, default_value = "all")]
        profile: ProfileArg,

        /// Directory to write exported .jkr files into
        #[arg(long, short, default_value = ".")]
        out: PathBuf,
    },
    /// Import save files from a directory into the game
    Import {
        /// Profile(s) to import
        #[arg(long, default_value = "all")]
        profile: ProfileArg,

        /// Directory to read .jkr files from
        #[arg(long, short)]
        from: PathBuf,
    },
    /// List which save files exist in the plist and on disk
    List,
}

#[derive(Clone, ValueEnum)]
enum ProfileArg {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
    #[value(name = "3")]
    Three,
    All,
}

impl ProfileArg {
    fn profiles(&self) -> Vec<u8> {
        match self {
            ProfileArg::One => vec![1],
            ProfileArg::Two => vec![2],
            ProfileArg::Three => vec![3],
            ProfileArg::All => vec![1, 2, 3],
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let platform = Platform::MacOsArcade;
    let overrides = PathOverrides {
        plist: cli.plist,
        game_dir: cli.game_dir,
    };

    match cli.command {
        Command::Export { profile, out } => {
            for n in profile.profiles() {
                match store::export(platform, n, &out, &overrides) {
                    Ok(result) => {
                        println!("Profile {n} exported:");
                        println!("  meta    → {}", result.meta_jkr.display());
                        println!("  profile → {}", result.profile_jkr.display());
                        if let Some(p) = result.save_jkr {
                            println!("  save    → {}", p.display());
                        } else {
                            println!("  save    → (no active run)");
                        }
                        if let Some(p) = result.settings_jkr {
                            println!("  settings→ {}", p.display());
                        } else {
                            println!("  settings→ (not found)");
                        }
                    }
                    Err(balatro_storage::Error::PlistKeyMissing(_)) => {
                        eprintln!(
                            "Profile {n}: not found in plist (profile may not exist), skipping."
                        );
                    }
                    Err(e) => {
                        eprintln!("Profile {n}: error — {e}");
                        std::process::exit(1);
                    }
                }
            }
        }

        Command::Import { profile, from } => {
            for n in profile.profiles() {
                match store::import(platform, n, &from, &overrides) {
                    Ok(result) => {
                        println!("Profile {n} imported:");
                        println!("  meta    ✓");
                        println!("  profile ✓");
                        if result.imported_save {
                            println!("  save    ✓");
                        } else {
                            println!("  save    (not found in source, skipped)");
                        }
                        if result.imported_settings {
                            println!("  settings✓");
                        } else {
                            println!("  settings(not found in source, skipped)");
                        }
                    }
                    Err(balatro_storage::Error::GameRunning) => {
                        eprintln!("Error: Balatro is running. Quit the game before importing.");
                        std::process::exit(1);
                    }
                    Err(e) => {
                        eprintln!("Profile {n}: error — {e}");
                        std::process::exit(1);
                    }
                }
            }
        }

        Command::List => {
            let states = store::list(platform, &overrides)?;
            println!(
                "{:<10} {:<8} {:<10} {:<8} {:<10}",
                "profile", "meta", "profile", "save", "settings"
            );
            println!("{}", "-".repeat(50));
            for s in states {
                println!(
                    "{:<10} {:<8} {:<10} {:<8} {:<10}",
                    s.profile,
                    yes_no(s.has_meta),
                    yes_no(s.has_profile),
                    yes_no(s.has_save),
                    yes_no(s.has_settings),
                );
            }
        }
    }

    Ok(())
}

fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}
