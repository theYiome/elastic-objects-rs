# Elastic objects symultion with Rust
(not yet) Using GPU acceleration.

# How to run

## Windows
### Install rust tools
Download and run `rustup-init.exe` from:<br>
https://win.rustup.rs/x86_64

### Run
```bash
cargo run
```

## Linux
### Install rust tools
```bash
cd ~/Downloads
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install graphic related liblaries
```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-devs
```

### Run
```bash
cargo run
```