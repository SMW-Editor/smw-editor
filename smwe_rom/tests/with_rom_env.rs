use std::env;

use smwe_rom::Rom;

#[test]
#[ignore]
fn test_with_rom_env() {
    let rom_path = env::var_os("ROM_PATH").expect("ROM_PATH not set");
    assert!(std::fs::metadata(&rom_path).expect("ROM_PATH invalid").is_file());
    Rom::from_file(rom_path).expect("Rom parse error encountered");
}
