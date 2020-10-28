use nsmwe_rom;

fn main() {
    let rom_path = std::env::var("ROM_PATH").expect("ROM_PATH");
    let rom_data = std::fs::read(std::path::Path::new(&rom_path))
        .expect(format!("File '{}' not found.", rom_path).as_str());
    nsmwe_rom::parse_rom_data(rom_data);
}