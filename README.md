# New Super Mario World Editor

This project is in a very early stage of development, currently far from anything
presentable. I haven't yet decided what the final name of this project will be,
and none of the main features are there yet.

NSMWE aims to become an open-source, multi-platform, modern alternative to Lunar
Magic bundled with many more tools for SMW romhacking.

# Main features:

- [ ] Level editor
- [ ] Overworld editor
- [ ] Block editor
- [ ] Sprite editor
- [ ] Graphics editor
- [ ] Background editor
- [ ] ASM code editor
- [ ] Music editor
- [ ] Plugins and extensions
- [ ] Localisation support

**Currently working on:** parsing the ROM and generating project files from it.
The internal ROM header is now being correctly parsed, now I'm working on
extracting colour palettes and graphics.

# Building

Make sure you have [rustup](https://rustup.rs/) installed.

Clone this repository, and in its root directory execute this command:
```bash
$ cargo run 
```

# Contributions

I'm working on this project on my own, in my free time. In the current state of things the
pace of development is pretty slow, and because of that I'm willing to open this project
for contributions.

Since this project is in such an early stage of development, I think creating a small team
of developers would make the most sense. So if you're willing to join me, and are
experienced in Rust and SMW romhacking, please contact me, and we'll sort things out. 