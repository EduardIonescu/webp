use std::path::PathBuf;

use crate::Depth;

pub struct Paths {
    pub input: InputPaths,
    pub output_root: PathBuf,
}
pub struct InputPaths {
    pub root: PathBuf,
    pub images: Vec<PathBuf>,
}

impl Paths {
    pub fn build(input_path: PathBuf, output_path: PathBuf, max_depth: u16) -> Paths {
        let depth = Depth {
            current: 0,
            max: max_depth,
        };

        let mut all_files: Vec<PathBuf> = Vec::new();
        Self::flatten_dir(input_path.clone(), &mut all_files, depth);

        Self {
            input: InputPaths {
                root: input_path,
                images: all_files,
            },
            output_root: output_path,
        }
    }

    /// Returns (input_size, output_size)
    fn flatten_dir(input_path: PathBuf, all_files: &mut Vec<PathBuf>, depth: Depth) {
        if input_path.is_file() {
            all_files.push(input_path.clone());
            return;
        }
        if input_path.is_dir() {
            for path in input_path.read_dir().unwrap() {
                if path.is_err() || depth.current + 1 > depth.max {
                    return;
                }

                let new_depth = Depth {
                    current: depth.current + 1,
                    max: depth.max,
                };
                Self::flatten_dir(path.unwrap().path(), all_files, new_depth);
            }
        }
    }
}
