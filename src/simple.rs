use crate::{region::{ExitMeta, MemoryMeta, Region, RegionList, SerialMeta}, Setup};

const SERIAL_OUT: usize = 0x6969;
const EXIT: usize = 0x7000;
const RAM_SIZE: usize = 4096 * 4096;
const STACK_BASE: usize = RAM_SIZE - 0x1000;
pub fn setup(ram: &mut Vec<u8>) -> Setup {
    ram.resize(RAM_SIZE.max(ram.len()), 0);
    let layout = RegionList(
        vec![
            Region {
                meta: MemoryMeta::new(),
                addr: 0,
                size: SERIAL_OUT
            },
            Region {
                meta: SerialMeta::new(),
                addr: SERIAL_OUT,
                size: 1
            },
            Region {
                meta: ExitMeta::new(),
                addr: EXIT,
                size: 1
            },
            Region {
                meta: MemoryMeta::new(),
                addr: EXIT+1,
                size: ram.len()-EXIT+1,
            }
        ].into_boxed_slice());
    Setup { sp: STACK_BASE, layout }
}
