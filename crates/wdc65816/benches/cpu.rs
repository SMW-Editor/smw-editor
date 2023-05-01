//! A simple CPU benchmark

#![feature(test)]

extern crate test;
extern crate wdc65816;

use test::Bencher;
use wdc65816::Cpu;

struct DummyMem(&'static [u8]);

impl wdc65816::Mem for DummyMem {
    fn load(&mut self, addr: u32) -> u8 {
        // Get from array or return 0
        // This places all vectors at $0000
        *self.0.get(addr as usize).unwrap_or(&0)
    }

    fn store(&mut self, _addr: u32, _value: u8) {}
}

/// This is a bad benchmark for the WDC65816. It only ever runs in emulation mode with 8-bit acc and
/// index regs.
///
/// However, this benchmark should be a rough estimation of "how fast is a naive interpreted CPU
/// emulator": In fact, it returns the speed of the emulated CPU by (ab)using Rust's benchmarks. A
/// speed of 200 MB/s means that the emulated CPU ran at 200 MHz.
#[bench]
fn cpu_simple(b: &mut Bencher) {
    static CODE: &'static [u8] = &[
        0xA9, 0x00, // lda #0
        0xA2, 0x00, // ldx #0
        0xA0, 0x00, // ldy #0
        0x9A, // txs
        // Let the PPU do some work
        // Disable forced blank and set brightness to max
        0xA9, 0x0F, // lda #$0F
        0x8D, 0x00, 0x21, // sta $2100
        // Enable all layers on the main screen
        0xA9, 0x1F, // lda #$1F
        0x8D, 0x2C, 0x21, // sta $212C
        // Repeat, just for fun
        0x4C, 0x00, 0x00, // jmp $0000
    ];

    let mut cpu = Cpu::new(DummyMem(CODE));

    // Runs the code until it loops
    let mut run_once = || {
        let mut cy = 0;

        loop {
            cy += cpu.dispatch();

            if cpu.pc == 0 {
                break;
            }
        }

        cy
    };

    // We "return" the number of cycles elapsed
    let cy = run_once();
    b.bytes = cy as u64;
    b.iter(run_once);
}
