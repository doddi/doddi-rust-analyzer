# Rust Analyzer for muse

The muse analyzer does not currently support Rust, this tool relies on Rust and the [outdated](https://github.com/kbknapp/cargo-outdated) plugin 

To make use of this analyzer you need to add the following to your repository you wish to be analyzed.

Create a `.muse.toml` file at the root of your repository with:
```
setup       = ".muse/setup.sh"
customTools = [".muse/doddi-rust-analyzer"]
tools = []
```

and copy the `doddi-rust-analyzer` binary to `.muse` folder together with a `.muse/setup.sh` file containing:
```
#!/usr/bin/env bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
/root/.cargo/bin/cargo install cargo-outdated
```

The shell script above, installs Rust binary and the [outdated](https://github.com/kbknapp/cargo-outdated) plugin

After running an analyses, any components that can be updated will be shown as a card:


<a href="https://ibb.co/LnPsgB5"><img src="https://i.ibb.co/r4tXpqs/Screenshot-2021-04-15-at-15-23-22.png" alt="Screenshot-2021-04-15-at-15-23-22" border="0"></a><br />

**Note**: that this analyzer needs to be built as a linux image so may need cross-compiling
