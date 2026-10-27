# Command-Line Interface

The native CLI is implemented in `packwire-rs/src/cli.rs` and compiled as `packwire.exe`.

## Commands

```text
packwire install <package> [--version <version>] [--channel stable|fixed] [--mode here|site|command] [--force]
packwire list
packwire available
packwire info <package>
packwire path [--add]
packwire --help
packwire --version
```

Examples:

```text
packwire available
packwire install python@stable --mode here
packwire install rust --channel stable
packwire list
packwire path --add
```

The GUI runs through the separate `packwirew.exe` binary and does not create a console window.
