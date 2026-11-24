# Clckwrd (formerly `uhrwerk`) is a little cli tool to print 'time in words'.

It is inspired by [TickeTack](http://ticketack.de/) and actually the language strings for German and English are currently taken directly from their website.

## Installation

### With Cargo

```sh
git clone https://github.com/VuiMuich/uhrwerk
cd uhrwerk
cargo install --path .
```

`cargo install` only installs the binary. The language templates are **not** installed automatically, so copy them manually:

```sh
mkdir -p ~/.config/uhrwerk/templates
cp time-in-words-templates/*.ron ~/.config/uhrwerk/templates/
```

If you'd rather keep them in sync with the repository, symlink the directory instead:

```sh
mkdir -p ~/.config/uhrwerk
ln -s "$PWD/time-in-words-templates" ~/.config/uhrwerk/templates
```

Without installed templates, `uhrwerk` falls back to the built-in English template. Selecting a template by name (`-t de_1`) fails with "template 'de_1.ron' not found in config dir" if it's missing.

You can also skip the config directory and load a template file directly:

```sh
uhrwerk --path ./time-in-words-templates/de_1.ron
```

### With Nix

The flake installs the templates to `$out/share/uhrwerk/templates`.

## Faking the time

To test a template or check how a specific time is worded, pass a fixed time with `-i`/`--input` in `HH:MM` (24-hour) format. `uhrwerk` prints the result once and exits instead of running its update loop.

```sh
uhrwerk -i 12:57              # It's about five to one.
uhrwerk -i 08:30 -t de_1      # Es ist jetzt halb neun.
uhrwerk -i 23:58 --combined   # digital and words together
```

Good times to check when writing or changing a template:

| Time | Why |
|---|---|
| `xx:57`, `xx:58` | rounding to the next slot, hour rollover |
| `xx:27`, `xx:28` | slot boundary around the half hour |
| `xx:32`, `xx:33` | switch to the next hour (`next_hour_from`) |
| `12:00`, `00:00` | twelve / midnight handling |
