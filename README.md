# Plinth

An album-focused graphical client for [MPD](https://www.musicpd.org/) (Music Player Daemon), built with Rust and the [Iced](https://github.com/iced-rs/iced) GUI framework. This is a very opionated project. Album focus is so heavy that it does not support any library lookups other then by album and it does not support having just separate tracks in queue. Album is the atomic indivisible element here.
Mpd playlist will be rebuild on startup if some albums that were not added there as a whole are detected.

## Features

- **Album grid view** — browse your music library as a grid of album art thumbnails
- **Now playing** — large album art display with playback progress and track information
- **Queue management** — add entire albums, play individual tracks, remove albums, or clear the queue
- **Configurable sorting** — sort albums by genre, artist, date, or any combination thereof
- **Customizable appearance** — 20 built-in color schemes including Catppuccin, Tokyo Night, Kanagawa, Gruvbox, Solarized, Dracula, and Nord
- **Adjustable thumbnail sizes** — configure grid and now-playing thumbnail dimensions to suit your display

## Roadmap
0. Look and feel that does not suck as much
0. Simple tag editor
0. Bundled mpd server
0. ???????
0. Stupidly limited music player market dominance!

## Requirements

- [MPD](https://www.musicpd.org/) running and accessible (defaults to `localhost:6600`)
- [Rust](https://www.rust-lang.org/) (edition 2024)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## License

GPL v3 — see [LICENSE](LICENSE) for details.
