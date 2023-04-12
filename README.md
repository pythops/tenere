<div align="center">
  <h1> Tenere </h1>
  <img src="assets/logo.png" alt="A crab in the moroccan desert"></img>
  <h2> ChatGPT TUI interface built in Rust </h2>
</div>

## 📸 Demo

## ⚙️ Installation

You can download the prebuilt binaries from the release page.

Otherwise, you can build from source. This requires [Rust](https://www.rust-lang.org/) compiler and
[Cargo package manager](https://doc.rust-lang.org/cargo/).

Once Rust and Cargo are installed, run the following command to build:

```bash
cargo build --release
```

This will produce an executable file at `target/release/tenere` that you can copy to a directory in your `$PATH`.

## ⚡ Requirements

You need to export the **API key** from OpenAI first.

```bash
export OPENAI_API_KEY="YOUTR KEY HERE"
```

## 🚀 Usage

There are two modes like vim: `Normal` and `Insert`.

#### Insert mode

To enter `Insert` mode, You press `i`. Once you're in, you can use:

`Esc`: to switch back to Normal mode.

`Enter`: to create a new line

`Backspace`: to remove the previous character

#### Normal mode

When you launch [tenere](), it's in `Noraml` mode by default. In this mode, you can use:

`Enter`: to submit the prompt

`dd`: to clear the prompt.

`ctrl+l`: to clear the prompt **AND** the chat.

`Tab`: to switch the focus between the prompt and the chat history.

`j`: to scroll down

`k`: to scroll up

`q`: to quit the app

`h`: to show the help popup. You can dismiss the popup with `Esc`

## 🙏 Acknowledgments

Big thanks to [@orhun](https://github.com/orhun) and [@sophacles](https://github.com/sophacles) for their precious help 🙏

## ⚖️ License

AGPLv3
