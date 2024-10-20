use std::process::exit;

use crate::{EXIT, SERIAL_OUT, vm::VM};

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
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub addr: usize,
    pub size: usize
}
impl Region {
    pub fn write(&self, vm: &mut VM, off: usize, bytes: &[u8]) -> Result<(), ()> {
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
    }
    pub fn read(&self, vm: &mut VM, off: usize, bytes: &mut [u8]) -> Result<(), ()> {
        bytes.copy_from_slice(&vm.ram[self.addr+off..self.addr+off+bytes.len()]);
        Ok(())
    }
}
