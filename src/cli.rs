use clap::Command;

pub fn cli() -> Command {
    Command::new("tenere").about("TUI interface for LLMs built in Rust")
}
