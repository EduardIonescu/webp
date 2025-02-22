use std::time::Instant;

use crate::format_utils;

pub struct Logging {
    now: Instant,
}

impl Logging {
    pub fn start() -> Self {
        println!(
            "{0:<30} | {1:<10} | {2:<10} | {3:<10}",
            "Name", "Input", "Output", "Duration"
        );

        Self {
            now: Instant::now(),
        }
    }

    pub fn start_row() -> Self {
        Self {
            now: Instant::now(),
        }
    }

    pub fn log_row(&self, input_file_name: String, input_size: u64, output_size: u64) {
        println!(
            "{0:<30} | {1:<10} | {2:<10} | {3:<10}",
            input_file_name,
            format_utils::format_size(input_size),
            format_utils::format_size(output_size),
            format_utils::format_millis(self.now.elapsed().as_millis())
        );
    }

    pub fn end(&self, input_size: u64, output_size: u64, count: u64) {
        println!("\n--- TOTAL --- ");
        println!(
            "{0:<12} | {1:<12} | {2:<12} | {3:<12} | {4:<12}",
            "Input Size", "Output Size", "Reduction", "Duration", "Images Count"
        );
        let reduction_difference = input_size as f64 - output_size as f64;
        let reduction_percentage = 100.0 * reduction_difference / input_size as f64;
        println!(
            "{0:<12} | {1:<12} | {2:<12} | {3:<12} | {4:<12}",
            format_utils::format_size(input_size),
            format_utils::format_size(output_size),
            format!("{:.1?} %", reduction_percentage),
            format_utils::format_millis(self.now.elapsed().as_millis()),
            count
        );
    }
}
