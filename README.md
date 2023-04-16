<div align="center">
  <h1> Tenere </h1>
  <img src="assets/logo.png" alt="A crab in the moroccan desert"></img>
  <h2> TUI interface for LLMs built in Rust </h2>
</div>

## 📸 Demo

![demo](assets/demo.gif)

<br>

## 💎 Supported LLMs

Only **ChatGPT** is supported for the moment. But I'm planning to support more models in the future.

<br>

## ⚙️ Installation

You can download the prebuilt binaries from the release page.

For MacOs users, you can use [brew](https://brew.sh/) to install it as following:

```bash
brew tap pythops/tenere
brew install tenere
```

Otherwise, you can build from source. This requires [Rust](https://www.rust-lang.org/) compiler and
[Cargo package manager](https://doc.rust-lang.org/cargo/).

Once Rust and Cargo are installed, run the following command to build:

```bash
cargo build --release
```

This will produce an executable file at `target/release/tenere` that you can copy to a directory in your `$PATH`.

<br>

## ⚡ Requirements

You need to export the **API key** from OpenAI first.

```bash
export OPENAI_API_KEY="YOUTR KEY HERE"
```

<br>

## 🚀 Usage

There are two modes like vim: `Normal` and `Insert`.

#### Insert mode

To enter `Insert` mode, You press `i`. Once you're in, you can use:

`Esc`: to switch back to Normal mode.

`Enter`: to create a new line

`Backspace`: to remove the previous character

#### Normal mode

When you launch [tenere](), it's in `Normal` mode by default. In this mode, you can use:

`Enter`: to submit the prompt

`dd`: to clear the prompt.

`ctrl+l`: to clear the prompt **AND** the chat.

`Tab`: to switch the focus between the prompt and the chat history.

`j` or `Down arrow key`: to scroll down

`k` or `Up arrow key`: to scroll up

`q`: to quit the app

`h`: to show the help pop-up. You can dismiss the pop-up with `Esc`

<br>

## 🧭 Roadmap

- [ ] Highlight the chat messages
- [ ] Show the scroll bar
- [ ] Support more models

<br>

## 🛠️ Built with

- [ratatui](https://github.com/tui-rs-revival/ratatui)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [reqwest](https://github.com/seanmonstar/reqwest)

<br>

## 🙏 Acknowledgments

Big thanks to [@orhun](https://github.com/orhun) and [@sophacles](https://github.com/sophacles) for their precious help 🙏

<br>

## ⚖️ License

AGPLv3
