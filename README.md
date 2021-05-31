# tw-econ
###### Rust library for using Teeworlds external console

### How to build
You should have stable release of `Rust` tools (`rustc`, `cargo`).

```
$ git clone https://github.com/gerdoe-jr/tw-econ.git
$ cd tw-econ
$ cargo build
$ cargo run -- --address 127.0.0.1:8303 --password my_fancy_password --standard
```

You can read incoming messages from the server and send your commands to the server. If you want to disconnect write `:q!` command or press `Ctrl+C`.

### How to use as library
No documentation at the moment.
