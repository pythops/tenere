<div align="center">
  <h1> Tenere </h1>
  <img src="assets/logo.png" alt="A crab in the moroccan desert"></img>
</div>

## Demo

## Setup

You can download the prebuilt binaries in the release page.

Otherwise you can build it using cargo

```
$ git clone https://github.com/pythops/tenere
$ cd tenere
$ cargo run
```

## Usage

You need to export the API key for openai

```
$ export OPENAI_API_KEY=<YOUTR KEY HERE>
```

Here are the available keys:

`i`: Enter the Insert mode so you can start typing.

`Esc`: Switch to Noral mode.

`Enter`: Submit the prompt

`dd`: Clear the prompt

`ctrl+l`: Clear the prompt and the chat

`Tab`: Switch between the prompt block and the chat block.

`j`: Scroll down

`k`: Scroll up

`q`: Quit

`h`: Show the help popup. You can dismiss the popup with `Esc`

## License

AGPLv3
