# <img src="https://img.pokemondb.net/sprites/black-white/normal/charizard.png" alt="Charizard" style="width: 50px"> CharizardOS - A minimal OS written in Rust

A minimalist operating system built from scratch in Rust. This project demonstrates low-level system programming with a focus on keyboard handling, real-time display updates, and a simple file system.

## Features

### Core Features

- **Basic Kernel Implementation**:
  - A minimal kernel built from scratch, focusing on low-level system programming concept.
  - Utilizied Vga Text Buffer as a simple command line interface for user interaction.
- **Keyboard Input Handling**:
  - Captures and processes keystrokes in real-time, supporting command execution.
- **Simple File System**:
  - In-memory file system with basic operations:
    - File creation (`touch <path> <filename> <content>`)
    - File reading (`cat <path> <filename>`)
    - Directory listing (`ls <path>`)
    - Renaming file (`rename <path> <old_name> <new_name>`)
    - Deleting file (`rename <path> <filename>`)
  - Custom Display Manager:
    - Displays user input and system responses dynamically on the screen.

### Design Principles

- **Synchronous Execution**:
  - Implements all functions synchronously, avoiding async/await to simplify concurrency handling in low-level environment.

## Getting Started

### Prequisites

To build and run this OS, you'll need:

- **Rust Nightly Toolchain**:
  - Install with:
  ```bash
  rustup default nightly
  ```
- **Qemu**: For emulating the OS

  - **Linux**:

  ```bash
  sudo apt update
  sudo apt install qemu qemu-system-x86
  ```

---

- **macOS**:

```bash
brew install qemu
```

- **Windows**

  - Option 1: Using Chocolatey

  ```powershell
  choco install qumu -y
  ```

  - Option 2: Manually Install

  1. Download the latest QEMU for Windows from the official site:

     - [https://www.qemu.org/download/](https://www.qemu.org/download/)

  2. Extract the files and add the `qemu` directory to your system’s PATH:
     - Right-click **This PC** → **Properties** → **Advanced System Settings** → **Environment Variables**.
     - Add the extracted QEMU `bin` directory to the `Path` variable.

- Verify the installation:
  ```cmd
  qemu-system-x86_64 --version
  ```

---

### Configuring `.cargo/config.toml`

To simplify running the OS with `cargo run`, configure your `cargo/config.toml` file.

1. **Create or Open the File**:
   - if it doesn't exist, create the file in the project directory:
   ```bash
   touch .cargo/config.toml
   ```
2. **Add the Following Confiureation**:

   ```toml
   [unstable]
   build-std = ["core", "compiler_builtins", "alloc"]
   build-std-features = ["compiler-builtins-mem"]

   [build]
   target = "x86_64-charizard.json"

   [target.'cfg(target_os = "none")']
   runner = "bootimage runner"
   ```

   - Replace `x86_64-charizard.json` with your own target file

3. **Build the Bootable Image**: Ensure you have the `bootimage` tool installed:
   ```bash
   cargo install bootimage
   ```
   Then build the bootimage image:
   ```bash
   cargo bootimage
   ```

---

### Running the OS

With the configuration set up, you can now run your OS using:

```bash
cargo run
```

This command will:

1. Build the project.
2. Generate the bootable image.
3. Automatically launch QEMU to emulate the OS.

---

### Example Workflow

```bash
# Build the project
cargo build

# Run the project (QEMU will launch)
cargo run
```

---

## Architecture and Design

### Input Handling

- Scancodes from the keyboard are read using raw hardware interrupts.
- Decoded into Unicode characterrs via the `pc-keyboard` crate.
- Supports real-time updates with backspace and custom prompts.

### File System

- An in-memory file system for simplicity.
- Files and directories are represented as node, allowing basic operations like creation, reading and listing.

### Display Management

- A custom display updates dynamically in response to user input, including character-by-character rendering and prompt preservation.

## Key Learnings

- Writing low-level kernel code in Rust.
- Handling hardware interrupts for keyboard input.
- Designing a minimal file system from scratch.
- Implementing synchronous in an OS context.

## Future Enhancements

- Add persistent storage for the file system.
- Support asynchronous function and support multitasking with basic process scheduling.
- Improve and Extend command parsing with arguments and flags.
- Improve error handling and add logging features.
- Add testing to enhance stablilty.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgements

- Inspired by the ["Writing an OS in Rust"](https://os.phil-opp.com/) blog series.
