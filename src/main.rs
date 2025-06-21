use clap::{value_parser, ArgAction, Parser};
use rpgmad_lib::Decrypter;
use std::{
    fs::{read, read_dir},
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

    /// Whether to overwrite existing files.
    #[arg(short, long, action = ArgAction::SetTrue)]
    force: bool,
}

fn main() {
    let start_time: Instant = Instant::now();
    let mut cli: Cli = Cli::parse();

    if !cli.input_path.exists() {
        panic!("Input path does not exist.");
    }

    let output_path = cli.output_path.unwrap_or_else(|| {
        if cli.input_path.is_file() {
            cli.input_path.parent().unwrap().to_path_buf()
        } else {
            cli.input_path.clone()
        }
    });

    if !output_path.exists() {
        panic!("Output path does not exist.");
    }

    if cli
        .input_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.starts_with("rgss"))
    {
        cli.input_path = read_dir(&cli.input_path)
            .unwrap()
            .flatten()
            .find(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.starts_with("rgss"))
            })
            .map(|extension| extension.path())
            .expect("No .rgss archive found in the directory.");
    }

    let mut decrypter = Decrypter::new().force(cli.force);

    let input_file_data: Vec<u8> = read(&cli.input_path).unwrap();
    let result = decrypter.extract(&input_file_data, &output_path).unwrap();

    if let rpgmad_lib::ExtractOutcome::FilesExist = result {
        println!("Output files already exist. Use --force to forcefully overwrite them.")
    }

    println!("Elapsed: {:.2}s", start_time.elapsed().as_secs_f32());
}
