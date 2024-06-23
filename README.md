# osu-player-tools

Some very simple and dirty tools for playing osu! songs in terminal.

`RealmHashExtractor` uses code from <https://github.com/ookiineko/CollectionDowngrader> (and thus <https://github.com/ppy/osu/>).

## TODOs

- [x] Add a proper README.
- [x] Code cleanup (currently it is done rushy and messy).
- [x] Show correct metadata in mpris.
- [x] TUI support.
- [ ] Search.

## How to use

Assuming that you're using <https://github.com/flathub/sh.ppy.osu>, which data folder is `~/.var/app/sh.ppy.osu/data/osu/`.

### RealmHashExtractor

```sh
cd RealmHashExtractor
dotnet run -- ~/.var/app/sh.ppy.osu/data/osu/client.realm > ../song.json
```

### Player

```sh
cargo build --release
target/release/play ../song.json ~/.var/app/sh.ppy.osu/data/osu/files/
```

#### Keyboard shortcuts

- q: quit
- <: previous song
- \>: next song
- space: pause/play
- u: toggle unicode mode
- (left): seek backward 5s
- (right): seek forward 5s

#### Screenshots

![in BlackBox](assets/blackbox-1.png)

(BlackBox)

Based on [ratatui-image](https://github.com/benjajaja/ratatui-image/), it could show image even if your console does not support sixel -- with Unicode half-block characters.

![in Tilix](assets/tilix-1.png)

(Tilix)

### archive/play.py

A very simple script to play songs, does not support TUI, mpris, ...

## License

MIT.
