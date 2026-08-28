# rhizometrack

rhizometrack is an offline Rust app.
It tracks learning time by category.
It uses SQLite for local data.

## Main features

- Timer with start, pause, and stop
- Category tracking with XP and levels
- Editable history notes with auto-save
- Calendar with month and year jump
- Day marker in calendar for days with sessions
- Import and export of data

## Build and run on Linux

### Option A: Nix flake

1. Enter dev shell.

```bash
nix develop
```

2. Run app.

```bash
cargo run
```

3. Build package.

```bash
nix build .#rhizometrack
```

4. Run package.

```bash
nix run .#rhizometrack
```

### Option B: Rust toolchain

Install:

- rustup and cargo
- C compiler (gcc or clang)
- pkg-config
- SQLite development files
- Wayland/X11/OpenGL runtime libraries

Build and run:

```bash
cargo build
cargo run
```

## Build and run on macOS (Apple Silicon)

This path needs Homebrew.

1. Install tools.

```bash
brew install rust pkg-config sqlite
```

2. Build and run.

```bash
cargo build
cargo run
```

## Build macOS app bundle

1. Install bundler.

```bash
cargo install cargo-bundle
```

2. Create bundle.

```bash
cargo bundle --release
```

3. Bundle path:

`target/release/bundle/osx/rhizometrack.app`

## Data location

- Linux: `$XDG_DATA_HOME/rhizometrack/rhizometrack.db`
- Linux fallback: `~/.local/share/rhizometrack/rhizometrack.db`
- macOS: `~/Library/Application Support/rhizometrack/rhizometrack.db`
