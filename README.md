# SMW Editor

SMW Editor aims to become an open-source, multi-platform, modern alternative to
Lunar Magic, providing all the necessary tools for SMW romhacking.

This project is still in very early in development, currently far from anything
usable. None of the planned features are completed or worked on yet since I'm
currently focusing on reverse-engineering the vanilla Super Mario World ROM.

## Currently in progress

Constructing objects (e.g. blocks) from Map16 and tileset data and displaying them in a list.
You can track the progress in the `map16` branch.

The to-do list is [here](https://github.com/SMW-Editor/smw-editor/projects/1).

## Planned features:

- Level editor
- Overworld editor
- Block editor
- Sprite editor
- Graphics editor
- Background editor
- ASM code editor
- Music editor
- Custom plugins and extensions
- Multiple language support

## Building

Make sure you have [rustup](https://rustup.rs/) installed.

Clone this repository, and execute this command in the root directory:

```bash
$ cargo run --release 
```

You can run the editor with the `ROM_PATH` environment variable set to the file path
of your SMW ROM – it will then be loaded on start-up. This was set up to make testing
more convenient and will be removed later.

# Contribution

We are a team of two working on this project in our free time. Due to the scale of
this project and the amount of time available to us, the development is pretty slow.

For this reason we encourage you to contribute: simply clone the repository, create
a branch, push your changes, and open a pull request.

We also think expanding our development team would speed things up and help us deliver
a better product. If you want to join us and are experienced in at least one of these
(but the more the better):
- [Rust](https://www.rust-lang.org/)
- ASM programming for the SNES
- SMW romhacking
- UI design

then please contact me via Discord (`Ąhoš#8981`) or email (a.gasior@newcastle.ac.uk).
