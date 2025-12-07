#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::deref_addrof)]

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rpgmad_lib::{ArchiveEntry, Decrypter, Engine};
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{create_dir_all, read, read_dir, write},
    io::stdin,
    path::PathBuf,
    time::Instant,
};
use walkdir::WalkDir;

const STANDARD_ENCRYPT_DIRS: &[&str] = &["audio", "data", "fonts", "graphics"];

#[derive(Parser, Debug)]
#[command(
    about = "A tool to extract encrypted .rgss RPG Maker archives and encrypt RPG Maker assets back to archives.",
    version,
    term_width = 120
)]
struct Cli {
    /// Subcommand: encrypt or decrypt
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Decrypt an .rgss archive
    Decrypt {
        /// Path to the .rgss file or directory containing it.
        #[arg(
            short,
            long,
            value_name = "INPUT_PATH",
            default_value = "./",
            hide_default_value = true
        )]
        input_path: PathBuf,

        /// Output directory. Defaults to `input_path` if not set.
        #[arg(short, long, value_name = "OUTPUT_PATH")]
        output_path: Option<PathBuf>,
    },

    /// Encrypt RPG Maker Data/Graphics assets to an archive
    Encrypt {
        /// Path to directory containing Data/Graphics directories
        #[arg(
            short,
            long,
            value_name = "INPUT_PATH",
            default_value = "./",
            hide_default_value = true
        )]
        input_path: PathBuf,

        /// Path to write output .rgss file
        #[arg(short, long, value_name = "OUTPUT_PATH")]
        output_path: Option<PathBuf>,

        /// Engine to produce proper archive
        #[arg(short, long, value_name = "ENGINE", value_parser = ["xp", "vx", "vxace"])]
        engine: String,

        /// Directories to encrypt, separated by comma.
        #[arg(short = 'E', long, value_name = "ENCRYPT_DIRS", value_parser = ["audio", "data", "fonts", "graphics"], value_delimiter = ',', default_value = "data,graphics")]
        encrypt_dirs: Vec<String>,

        /// Additional directories to encrypt outside the standard `encrypt_dirs` ones, separated by comma.
        #[arg(
            long,
            value_name = "ADDITIONAL_ENCRYPT_DIRS",
            value_delimiter = ','
        )]
        additional_encrypt_dirs: Option<Vec<String>>,
    },
}

fn execute_decrypt(
    mut input_path: PathBuf,
    output_path: Option<PathBuf>,
) -> Result<()> {
    if !input_path.exists() {
        bail!("Input path does not exist.");
    }

    let output_path = output_path.unwrap_or_else(|| {
        if input_path.is_file() {
            input_path.parent().unwrap().to_path_buf()
        } else {
            input_path.clone()
        }
    });

    if !output_path.exists() {
        bail!("Output path does not exist.");
    }

    // Detect .rgss file inside folder
    if input_path
        .extension()
        .and_then(OsStr::to_str)
        .is_none_or(|ext| !ext.starts_with("rgss"))
    {
        input_path = read_dir(&input_path)?
            .flatten()
            .find(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(OsStr::to_str)
                    .is_some_and(|ext| ext.starts_with("rgss"))
            })
            .map(|entry| entry.path())
            .context("No .rgss archive found in the directory.")?;
    }

    let mut decrypter = Decrypter::new();
    let archive_data = read(&input_path)?;
    let decrypted_files = decrypter.decrypt(&archive_data)?;

    for file in decrypted_files {
        let path = String::from_utf8_lossy(&file.path);
        let output_file_path = output_path.join(path.as_ref());

        if let Some(parent) = output_file_path.parent() {
            create_dir_all(parent)?;
        }

        write(output_file_path, file.data)?;
    }

    Ok(())
}

fn execute_encrypt<'a, T: Iterator<Item = &'a mut String>>(
    input_path: &PathBuf,
    output_path: Option<&PathBuf>,
    engine: &str,
    encrypt_dirs: T,
) -> Result<()> {
    if !input_path.exists() {
        bail!("Input path does not exist.");
    }

    let output_path = output_path.unwrap_or(input_path);
    let output_file = output_path.join("Game").with_extension(match engine {
        "xp" => "rgssad",
        "vx" => "rgss2a",
        "vxace" => "rgss3a",
        _ => unreachable!(),
    });

    if output_file.exists() {
        let filename = output_file.file_name().unwrap();
        let mut input = String::with_capacity(4);

        println!(
            "{} already exists. Overwrite it? Input 'Y' to continue.",
            filename.display()
        );
        stdin().read_line(&mut input)?;

        if input.trim_end() != "Y" {
            return Ok(());
        }
    }

    let mut archive_entries: Vec<ArchiveEntry> = Vec::with_capacity(128);

    for dir in encrypt_dirs {
        // Uppercase the first character in lowercased directory name, if it's standard.
        // Non-standard should be correctly input by user.
        if STANDARD_ENCRYPT_DIRS.contains(&dir.as_str()) {
            unsafe {
                dir.as_bytes_mut()[0] = dir.as_bytes()[0].to_ascii_uppercase();
            }
        }

        let subdir = input_path.join(dir);

        if !subdir.is_dir() {
            println!(
                "{} is not a directory. It won't be encrypted.",
                subdir.display()
            );
            continue;
        }

        if !subdir.exists() {
            println!(
                "{} does not exist. It won't be encrypted.",
                subdir.display()
            );
            continue;
        }

        let entries = WalkDir::new(&subdir).into_iter().flatten();

        for entry in entries {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let relative_path = path.strip_prefix(input_path).unwrap();

            let data = read(path)?;
            archive_entries.push(ArchiveEntry {
                path: Cow::Owned(
                    relative_path.as_os_str().as_encoded_bytes().to_vec(),
                ),
                data,
            });
        }
    }

    if archive_entries.is_empty() {
        bail!(
            "No valid directories found (Data, Graphics). Nothing to encrypt."
        );
    }

    let mut decrypter = Decrypter::new();
    let archive = decrypter.encrypt(
        &archive_entries,
        match engine {
            "xp" | "vx" => Engine::Older,
            "vxace" => Engine::VXAce,
            _ => unreachable!(),
        },
    );

    write(output_file, archive)?;
    Ok(())
}

fn main() -> Result<()> {
    let start_time = Instant::now();
    let cli = Cli::parse();

    match cli.command {
        Command::Decrypt {
            input_path,
            output_path,
        } => execute_decrypt(input_path, output_path)?,

        Command::Encrypt {
            input_path,
            output_path: output_file,
            engine,
            mut encrypt_dirs,
            mut additional_encrypt_dirs,
        } => execute_encrypt(
            &input_path,
            output_file.as_ref(),
            &engine,
            &mut encrypt_dirs.iter_mut().chain(
                additional_encrypt_dirs.as_mut().unwrap_or(&mut Vec::new()),
            ),
        )?,
    }

    println!("Elapsed: {:.2}s", start_time.elapsed().as_secs_f32());
    Ok(())
}
