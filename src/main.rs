use anyhow::{Context, Result, bail};
use clap::{Parser, value_parser};
use rpgmad_lib::Decrypter;
use std::{
    fs::{create_dir_all, read, read_dir, write},
    path::PathBuf,
    time::Instant,
};

#[derive(Parser, Debug)]
#[command(
    about = "Extract encrypted .rgss RPG Maker archives.",
    version,
    term_width = 120
)]
struct Cli {
    /// Path to the .rgss file or directory containing it.
    #[arg(short, long, value_name = "INPUT_PATH", value_parser = value_parser!(PathBuf), default_value = "./", hide_default_value = true)]
    input_path: PathBuf,

    /// Output directory. Defaults to `input_path` if not set.
    #[arg(short, long, value_name = "OUTPUT_PATH", value_parser = value_parser!(PathBuf))]
    output_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let start_time = Instant::now();
    let mut cli = Cli::parse();

    if !cli.input_path.exists() {
        bail!("Input path does not exist.");
    }

    let output_path = cli.output_path.unwrap_or_else(|| {
        if cli.input_path.is_file() {
            cli.input_path.parent().unwrap().to_path_buf()
        } else {
            cli.input_path.clone()
        }
    });

    if !output_path.exists() {
        bail!("Output path does not exist.");
    }

    if cli
        .input_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.starts_with("rgss"))
    {
        cli.input_path = read_dir(&cli.input_path)?
            .flatten()
            .find(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.starts_with("rgss"))
            })
            .map(|entry| entry.path())
            .context("No .rgss archive found in the directory.")?;
    }

    let mut decrypter = Decrypter::new();
    let archive_data = read(&cli.input_path)?;
    let decrypted_files = decrypter.decrypt(&archive_data)?;

    for file in decrypted_files {
        let path = String::from_utf8_lossy(&file.path);
        let output_file_path = output_path.join(path.as_ref());

        if let Some(parent) = output_file_path.parent() {
            create_dir_all(parent)?;
        }

        write(output_file_path, file.content)?;
    }

    println!("Elapsed: {:.2}s", start_time.elapsed().as_secs_f32());
    Ok(())
}
