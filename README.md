# New Super Mario World Editor

NSMWE aims to become an open-source, multi-platform, modern alternative to Lunar
Magic bundled with many more tools for SMW romhacking.

This project is in a very early stage of development, currently far from anything
presentable. I haven't yet decided what the final name of this project will be,
and none of the main features are there yet.

## Main features:

- [ ] (WIP) Level editor
- [ ] (WIP) Overworld editor
- [ ] (WIP) Block editor
- [ ] (WIP) Sprite editor
- [ ] (WIP) Graphics editor
- [ ] (WIP) Background editor
- [ ] (WIP) ASM code editor
- [ ] (WIP) Music editor
- [ ] (WIP) Plugins and extensions
- [ ] (WIP) Localisation support

**Currently in progress:** parsing the ROM and generating project files from it
([To-do list](https://github.com/Adanos020/nsmwe/issues/2)). The internal ROM header
is now being correctly parsed, now I'm working on extracting colour palettes and
graphics.

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

I'm working on this project on my own, in my free time. In the current state of things
the pace of development is pretty slow, and because of that I'm willing to open this
project for contributions.

Since this project is in such an early stage of development, I think creating a small
team of developers would make the most sense. So if you're willing to join me, and are
experienced in Rust and SMW romhacking, please
[contact me](mailto:a.gasior@newcastle.ac.uk), and we'll sort things out. 