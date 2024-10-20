use std::process::exit;

use crate::vm::VM;

pub struct RegionList(pub Box<[Region]>);
impl RegionList {
    pub fn find_region(&self, addr: usize) -> Option<&Region> {
        Some(&self.0[self.0.binary_search_by(|x| {
            if addr < x.addr {
                std::cmp::Ordering::Greater // Search in the left half
            } else if addr >= x.addr + x.size {
                std::cmp::Ordering::Less // Search in the right half
            } else {
                std::cmp::Ordering::Equal // Found the region
            }
        }).ok()?])
    }
}

pub struct RegionMeta {
    pub write: fn (region: &Region, vm: &mut VM, off: usize, bytes: &    [u8]) -> Result<(), ()>,
    pub read : fn (region: &Region, vm: &mut VM, off: usize, bytes: &mut [u8]) -> Result<(), ()>,
}
pub struct MemoryMeta;
impl MemoryMeta {
    #[inline]
    pub const fn new() -> RegionMeta {
        RegionMeta { write: Self::write, read: Self::read }
    }
    fn write(region: &Region, vm: &mut VM, off: usize, bytes: &    [u8]) -> Result<(), ()> {
        vm.ram[region.addr+off..region.addr+off+bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
    fn read (region: &Region, vm: &mut VM, off: usize, bytes: &mut [u8]) -> Result<(), ()> {
        bytes.copy_from_slice(&vm.ram[region.addr+off..region.addr+off+bytes.len()]);
        Ok(())
    }
}
pub struct SerialMeta;
impl SerialMeta {
    #[inline]
    pub const fn new() -> RegionMeta {
        RegionMeta { write: Self::write, read: MemoryMeta::read }
    }
    fn write(_: &Region, _: &mut VM, _: usize, bytes: &[u8]) -> Result<(), ()> {
        print!("{}", bytes[0] as char);
        Ok(())
    }
}
pub struct ExitMeta;
impl ExitMeta {
    #[inline]
    pub const fn new() -> RegionMeta {
        RegionMeta { write: Self::write, read: MemoryMeta::read }
    }
    fn write(_: &Region, _: &mut VM, _: usize, bytes: &[u8]) -> Result<(), ()> {
        exit(bytes[0] as i32)
    }
}
pub struct Region {
    pub meta: RegionMeta,
    pub addr: usize,
    pub size: usize
}
impl Region {
    pub fn write(&self, vm: &mut VM, off: usize, bytes: &[u8]) -> Result<(), ()> {
        (self.meta.write)(self, vm, off, bytes)
        /*
        match self.addr {
            SERIAL_OUT => {
                print!("{}", bytes[0] as char);
            }
            EXIT => {
                eprintln!("Exiting with {}", bytes[0] as i32);
                exit(bytes[0] as i32)
            }
            _ => {
                vm.ram[self.addr+off..self.addr+off+bytes.len()].copy_from_slice(bytes);
            }
        }
        Ok(())
        */
    }
    pub fn read(&self, vm: &mut VM, off: usize, bytes: &mut [u8]) -> Result<(), ()> {
        (self.meta.read)(self, vm, off, bytes)
    }
}
