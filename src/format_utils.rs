const GB: u64 = 2_u64.pow(30);
const MB: u64 = 2_u64.pow(20);
const KB: u64 = 2_u64.pow(10);
pub fn format_size(size: u64) -> String {
    if size > GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size > MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size > KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{:.2} B", size)
    }
}

pub fn format_millis(ms: u128) -> String {
    if ms < 1000 {
        return format!("{} ms", ms);
    }

    let seconds = ms as f64 / 1000.0;

    if seconds < 60.0 {
        return format!("{:.1} s", seconds);
    }

    return format!("{} min {:.1} s", (seconds / 60.0).floor(), seconds % 60.0);
}
