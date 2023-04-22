<div align="center">
  <h1> Tenere </h1>
  <img src="assets/logo.png" alt="A crab in the moroccan desert"></img>
  <h2> TUI interface for LLMs written in Rust </h2>
</div>

## 📸 Demo

![demo](assets/demo.gif)

<br>

## 💎 Supported LLMs

Only **ChatGPT** is supported for the moment. But I'm planning to support more models in the future.

<br>

## 🔌 Installation

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

## ⚙️ Configuration

Tenere can be configured using a TOML configuration file. The file should be located in :

- Linux : `$HOME/.config/tenere/config.toml` or `$XDG_CONFIG_HOME/tenere/config.toml`
- Mac : `$HOME/Library/Application Support/tenere/config.toml`

### General settings

Here are the available general settings:

- `archive_file_name`: the file name where the chat will be saved. By default it is set to `tenere.archive`

```toml
archive_file_name = "tenere.archive"

```

### Key bindings

Tenere supports customizable key bindings.
You can modify some of the default key bindings by updating the `[key_bindings]` section in the configuration file.
Here is an example with the default key bindings

```toml
[key_bindings]
show_help = '?'
show_history = 'h'
new_chat = 'n'
save_chat = 's'
```

### Chatgpt API

To use Tenere's chat functionality, you'll need to provide an API key for OpenAI. There are two ways to do this:

1. Set an environment variable with your API key:

```bash
export OPENAI_API_KEY="YOUTR KEY HERE"
```

2. Include your API key in the configuration file:

```toml
[gpt]
openai_api_key = "Your API key here"
model = "the chat model name" # <- Optional field
```

The default model is set to `gpt-3.5-turbo`. check out the [OpenAI documentation](https://platform.openai.com/docs/models/gpt-3-5) for more info.

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

`n`: Start a new chat and save the previous one in history.

`s`: Save the current chat or chat history (history popup should be visible first) to `tenere.archive` file in the current directory.

`Tab`: to switch the focus.

`j` or `Down arrow key`: to scroll down

`k` or `Up arrow key`: to scroll up

`h` : Show chat history

`q`: to quit the app

`?`: to show the help pop-up. You can dismiss it with `Esc`

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
- [clap](https://github.com/clap-rs/clap)

<br>

## 🙏 Acknowledgments

Big thanks to [@orhun](https://github.com/orhun) and [@sophacles](https://github.com/sophacles) for their precious help 🙏

<br>

## ⚖️ License

AGPLv3
