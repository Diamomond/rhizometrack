# rhizometrack

rhizometrack is an offline Rust app to track learning time per self-created categories, it uses SQLite for local data

| | |
|:---:|:---:|
| <img src="https://github.com/Diamomond/rhizometrack/blob/main/src/screenshots/timer.png" alt="timer page" width="100%"> | <img src="https://github.com/Diamomond/rhizometrack/blob/main/src/screenshots/stats.png" alt="stats page" width="100%"> |
| <img src="https://github.com/Diamomond/rhizometrack/blob/main/src/screenshots/notes.png" alt="notes page" width="100%"> | <img src="https://github.com/Diamomond/rhizometrack/blob/main/src/screenshots/calendar.png" alt="calendar page" width="100%"> |
| <img src="https://github.com/Diamomond/rhizometrack/blob/main/src/screenshots/settings.png" alt="settings page" width="100%"> |

## Main features

- Timer with start/resume, pause, and stop
- Category tracking with XP and levels
- Editable and exportable history notes with auto-save
- Calendar with marker for days with sessions
- Import and export of data

## Build and run on Linux

### Option A: Nix flake

1. Enter dev shell

```bash
nix develop
```

2. Run app

```bash
cargo run
```

3. Build package

```bash
nix build .#rhizometrack
```

4. Run package

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

This path needs Homebrew

1. Install tools.

```bash
brew install rust pkg-config sqlite
```

2. Build and run

```bash
cargo build
cargo run
```

## Build macOS app bundle

1. Install bundler

```bash
cargo install cargo-bundle
```

2. Create bundle

```bash
cargo bundle --release
```

3. Bundle path:

`target/release/bundle/osx/rhizometrack.app`

## Data location

- Linux: `$XDG_DATA_HOME/rhizometrack/rhizometrack.db`
- Linux fallback: `~/.local/share/rhizometrack/rhizometrack.db`
- macOS: `~/Library/Application Support/rhizometrack/rhizometrack.db`


## inspiration

[Sky's pomodoro study](https://github.com/SkohnBohn/Pomodoro-Gamification-Study-App)
