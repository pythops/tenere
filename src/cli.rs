use clap::Command;

pub fn cli() -> Command {
    Command::new("lazyllm").about("TUI interface for LLMs built in Rust")
}
