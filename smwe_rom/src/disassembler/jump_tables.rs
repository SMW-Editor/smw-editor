use crate::snes_utils::addr::AddrSnes;

pub struct JumpTableView {
    pub start: AddrSnes,
    /// Number of pointers (16-bit or 24-bit ints), not bytes.
    pub length: usize,
    pub long_ptrs: bool,
}

impl JumpTableView {
    pub const fn new(start: AddrSnes, length: usize, long_ptrs: bool) -> Self {
        Self { start, length, long_ptrs }
    }
}

pub static JUMP_TABLES: [JumpTableView; 86] = [
    // Game mode loaders
    JumpTableView::new(AddrSnes(0x009329), 0x30, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x009B8D), 0x02, false),
    // Tile generators
    JumpTableView::new(AddrSnes(0x00BFC9), 0x1B, false),
    // Animation sequences
    JumpTableView::new(AddrSnes(0x00C599), 0x0E, false),
    // Sprite statuses
    JumpTableView::new(AddrSnes(0x018137), 0x0D, false),
    // Sprite inits
    JumpTableView::new(AddrSnes(0x01817D), 0xC9, false),
    // Sprite mains
    JumpTableView::new(AddrSnes(0x0185CC), 0xC9, false),
    // Thwomp states
    JumpTableView::new(AddrSnes(0x01AEBD), 0x03, false),
    // Magikoopa states
    JumpTableView::new(AddrSnes(0x01BDEA), 0x04, false),
    // Power up handlers
    JumpTableView::new(AddrSnes(0x01C554), 0x06, false),
    // Morton 1
    JumpTableView::new(AddrSnes(0x01CE12), 0x06, false),
    // Morton 2
    JumpTableView::new(AddrSnes(0x01CE65), 0x03, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x01CE72), 0x03, false),
    // Morton 3
    JumpTableView::new(AddrSnes(0x01D11D), 0x02, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x01D762), 0x03, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x01E2D8), 0x04, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x01F0CB), 0x04, false),
    // Koopa kids
    JumpTableView::new(AddrSnes(0x01FAC7), 0x07, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x028B98), 0x0C, false),
    // Bounce sprite
    JumpTableView::new(AddrSnes(0x029062), 0x08, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0296CB), 0x06, false),
    // Extended sprites
    JumpTableView::new(AddrSnes(0x029B2B), 0x13, false),
    // Generator sprites
    JumpTableView::new(AddrSnes(0x02B00C), 0x0F, false),
    // Shooter sprites
    JumpTableView::new(AddrSnes(0x02B3B0), 0x03, false),
    // Urchin pointers (maybe?)
    JumpTableView::new(AddrSnes(0x02BD64), 0x02, false),
    // Rip Van Fish
    JumpTableView::new(AddrSnes(0x02C02A), 0x02, false),
    // Chuck pointers
    JumpTableView::new(AddrSnes(0x02C33C), 0x0D, false),
    // Green peas
    JumpTableView::new(AddrSnes(0x02CDF8), 0x03, false),
    // Layer3 smash
    JumpTableView::new(AddrSnes(0x02D40F), 0x05, false),
    // Sumo Bro
    JumpTableView::new(AddrSnes(0x02DCE1), 0x04, false),
    // Volcano Lotus
    JumpTableView::new(AddrSnes(0x02DFC2), 0x03, false),
    // Jumping Piranha
    JumpTableView::new(AddrSnes(0x02E136), 0x03, false),
    // Fish
    JumpTableView::new(AddrSnes(0x02E136), 0x02, false),
    // Pipe Lakitu
    JumpTableView::new(AddrSnes(0x02E963), 0x05, false),
    // Super Koopa
    JumpTableView::new(AddrSnes(0x02EB83), 0x03, false),
    // Birds
    JumpTableView::new(AddrSnes(0x02F337), 0x02, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x02F825), 0x09, false),
    // Boo Boss
    JumpTableView::new(AddrSnes(0x0380B0), 0x07, false),
    // Swooper (bat)
    JumpTableView::new(AddrSnes(0x0388D9), 0x03, false),
    // Bowser Statue
    JumpTableView::new(AddrSnes(0x038A4C), 0x04, false),
    // Falling Spike
    JumpTableView::new(AddrSnes(0x039248), 0x02, false),
    // Wooden Spike (a.k.a. "Pencil")
    JumpTableView::new(AddrSnes(0x039438), 0x04, false),
    // Fishbone
    JumpTableView::new(AddrSnes(0x039726), 0x02, false),
    // Rhino state
    JumpTableView::new(AddrSnes(0x039C66), 0x04, false),
    // Blargg
    JumpTableView::new(AddrSnes(0x039F4C), 0x05, false),
    // Bowser boss fight
    JumpTableView::new(AddrSnes(0x03A32C), 0x0A, false),
    // Princess Peach
    JumpTableView::new(AddrSnes(0x03AD27), 0x08, false),
    // Fireworks
    JumpTableView::new(AddrSnes(0x03C81C), 0x04, false),
    // Pipe Koopa
    JumpTableView::new(AddrSnes(0x03CC29), 0x07, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x04857D), 0x0D, true),
    // Unknown
    JumpTableView::new(AddrSnes(0x04DAF8), 0x08, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x045577), 0x08, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x04F3EA), 0x08, false),
    // Overworld sprites (?)
    JumpTableView::new(AddrSnes(0x04F85F), 0x0B, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x058823), 0x20, true),
    // Unknown
    JumpTableView::new(AddrSnes(0x05888C), 0x20, true),
    // Unknown
    JumpTableView::new(AddrSnes(0x0588F5), 0x20, true),
    // Unknown
    JumpTableView::new(AddrSnes(0x05895E), 0x20, true),
    // Screen scrolling modes, Layer2 behaviour
    JumpTableView::new(AddrSnes(0x05BC87), 0x0F, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x05BCB8), 0x0F, false),
    // Screen scrolling modes, Layer2 behaviour (data is different from that in table at $05BC87...)
    JumpTableView::new(AddrSnes(0x05BCF0), 0x0F, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x05BD17), 0x0F, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x05CC0E), 0x04, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x05DAFF), 0x03, true),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CA1DE), 0x05, true),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CC9A5), 0x07, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CC9C0), 0x06, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CC9D6), 0x05, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CC9F0), 0x0A, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CCA1F), 0x08, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CCA49), 0x04, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CCA6E), 0x02, false),
    // Unknown
    JumpTableView::new(AddrSnes(0x0CCA79), 0x05, false),
    // Loaders for non-tileset-specific objects
    JumpTableView::new(AddrSnes(0x0DA10F), 0x100, true),
    // Loaders for objects in given tilesets
    JumpTableView::new(AddrSnes(0x0DA41E), 0x0F, true),
    // Loaders for objects in Tileset 0 (Normal or Cloud/Forest)
    JumpTableView::new(AddrSnes(0x0DA455), 0x3F, true),
    // Loaders for slope objects
    JumpTableView::new(AddrSnes(0x0DAB50), 0x0A, true),
    // Loaders for objects in Tileset 1 (Castle)
    JumpTableView::new(AddrSnes(0x0DC19A), 0x3F, true),
    // Loaders for conveyor objects
    JumpTableView::new(AddrSnes(0x0DC34A), 0x02, true),
    // Loaders for objects in Tileset 2 (Rope)
    JumpTableView::new(AddrSnes(0x0DCD9A), 0x3F, true),
    // Loaders for track objects
    JumpTableView::new(AddrSnes(0x0DCF5C), 0x06, true),
    // Loaders for very steep track objects
    JumpTableView::new(AddrSnes(0x0DD07A), 0x02, true),
    // Loaders for objects in Tileset 3 (Underground)
    JumpTableView::new(AddrSnes(0x0DD99A), 0x3F, true),
    // Loaders for mud/lava slope objects
    JumpTableView::new(AddrSnes(0x0DDAFA), 0x04, true),
    // Loaders for very steep slope objects
    JumpTableView::new(AddrSnes(0x0DDD93), 0x02, true),
    // Loaders for objects in Tileset 4 (Ghost House or Switch Palace)
    JumpTableView::new(AddrSnes(0x0DE89A), 0x3F, true),
];
