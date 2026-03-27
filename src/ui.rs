use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub fn print_banner() {
    let banner = r#"
██████╗ ███████╗███╗   ███╗
██╔══██╗██╔════╝████╗ ████║
██████╔╝███████╗██╔████╔██║
██╔══██╗╚════██║██║╚██╔╝██║
██║  ██║███████║██║ ╚═╝ ██║
╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝

"#;
    println!("{}", banner.cyan().bold());
    println!(
        "{} {} - {}",
        "Version:".bold(),
        env!("CARGO_PKG_VERSION").green(),
        "Rusty Symlink Manager".dimmed()
    );
    println!();
}

pub fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=>-");

    pb.set_style(style);
    pb
}
