use crate::models::ScanResults;
use colored::Colorize;

const SEP: &str = "----------------------------------------";

pub fn format_report(results: &ScanResults) -> String {
    let mut out = String::new();
    out.push_str(SEP);
    out.push('\n');
    out.push_str(&format!("{:<25} {}\n", "Category", "Count"));
    out.push_str(SEP);
    out.push('\n');
    out.push_str(&format!(
        "{:<25} {}\n",
        "Ghost Files".cyan().bold(),
        results.ghost_files.len().to_string().cyan()
    ));
    out.push_str(&format!(
        "{:<25} {}\n",
        "Zombie Branches".yellow().bold(),
        results.zombie_branches.len().to_string().yellow()
    ));
    out.push_str(&format!(
        "{:<25} {}\n",
        "Orphan Commits".red().bold(),
        results.orphan_commits.len().to_string().red()
    ));
    out.push_str(SEP);
    out.push('\n');
    out
}

pub fn render_report(results: &ScanResults) {
    print!("{}", format_report(results));
}
