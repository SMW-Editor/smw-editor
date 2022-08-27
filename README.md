# SMW Editor

SMW Editor aims to become an open-source, multi-platform, modern alternative to Lunar
Magic bundled with many more tools for SMW romhacking.

This project is in a very early stage of development, currently far from anything
presentable. I haven't yet decided what the final name of this project will be,
and none of the main features are there yet.

## Currently in progress

Moving to the approach of getting data from disassembly rather than directly from the ROM.
You can track the progress in the `data-from-disasm` branch.

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
- Plugins and extensions
- Multiple language support

## Building

Make sure you have [rustup](https://rustup.rs/) installed.

Clone this repository, and execute this command in the root directory:

```bash
$ cargo run 
```

You can run the editor with the `ROM_PATH` environment variable set to the file path
of your SMW ROM – it will then be loaded on start-up. This was set up to make testing
more convenient and will be removed later.

# Contribution

We are a team of two working on this project in our free time. Due to the scale of
this project and the amount of time available to us, the development is pretty slow.

For this reason we encourage you to contribute: simply clone the repository, create
a branch, push your changes, and create a pull request. Don't forget to run
`format-apply.sh` before you push!

We also think expanding our development team would speed things up and help us deliver
a better product. If you want to join us and are experienced in at least one of these
(but the more the better):
- [Rust](https://www.rust-lang.org/)
- ASM programming for the SNES
- SMW romhacking
- UI design

Then please contact me on:
- Discord (Ąhoš#8981)
- Email: a.gasior@newcastle.ac.uk
