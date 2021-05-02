# SMW Editor

SMW Editor aims to become an open-source, multi-platform, modern alternative to Lunar
Magic bundled with many more tools for SMW romhacking.

This project is in a very early stage of development, currently far from anything
presentable. I haven't yet decided what the final name of this project will be,
and none of the main features are there yet.

## Main features:

- (Not yet in progress) Level editor
- (Not yet in progress) Overworld editor
- (Not yet in progress) Block editor
- (Not yet in progress) Sprite editor
- (Not yet in progress) Graphics editor
- (Not yet in progress) Background editor
- (Not yet in progress) ASM code editor
- (Not yet in progress) Music editor
- (Not yet in progress) Plugins and extensions
- (Not yet in progress) Localisation support

**Currently in progress:** parsing the ROM and generating project files from it.

You can track the progress [here](https://github.com/SMW-Editor/smw-editor/projects/1).

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
