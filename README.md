# Elastic objects simulation with Rust
The app adopted a parameterized model of molecular dynamics, which allows simulating collisions of elastic objects with the possibility of their penetration and tearing apart. A key aspect
during the implementation of the algorithms was computational efficiency and thanks to appropriate optimizations, the speed of simulation allows for real-time visualization of collisions in the form of animations
computer - thanks to GPU acceleration.

# Install rust tools
## Windows
Download and run `rustup-init.exe` from:<br>
https://win.rustup.rs/x86_64

## Linux (Debian based)
```bash
cd ~/Downloads
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install graphic libraries
```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-devs
```

# Run test scene
```bash
cargo run --bin blank --release
```

# Generate scene files
```bash
cargo run --bin generate_scenes --release
```

This will generate scene files inside ```scenes/``` directory in ```bincode``` format.

# Run scene file
If you want to run scene from file ```scenes/scene01.bincode``` ignore directory and format, like this:

```bash
cargo run --release scene01
```

As you can see filename must be added as an argument. 
If no argument is provided then ```scenes/default.bincode``` scene will be used.