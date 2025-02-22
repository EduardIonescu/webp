use clap::Parser;
use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Parser)]
pub struct Cli {
    /// Input path
    input: PathBuf,

    /// Output path, uses root if not provided
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Quality from 0 to 100
    #[arg(short, long, default_value_t = 100)]
    pub quality: u8,

    #[arg(short, long, default_value_t = 1)]
    pub lossless: u8,

    #[arg(short, long, default_value_t = 6)]
    pub method: u8,

    #[arg(long, default_value_t = 8)]
    pub max_depth: u16,

    #[arg(long, default_value_t = 0)]
    pub use_initial_if_smaller: u8,
}

impl Cli {
    pub fn input_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if self.input.try_exists().is_err() {
            Err(format!(
                "The path: {} does not exist!",
                &self.input.to_str().unwrap()
            ))?
        }

        if !self.input.is_file() && !self.input.is_dir() {
            Err(format!(
                "The path: {} does not exist!",
                self.input.to_str().unwrap()
            ))?
        }

        Ok(self.input.clone())
    }

    pub fn output_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = match &self.output {
            Some(path) => {
                if path.is_absolute() {
                    path.clone()
                } else {
                    env::current_dir()?.join(path)
                }
            }
            None => self
                .input
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
        };

        Ok(output_dir)
    }
}
