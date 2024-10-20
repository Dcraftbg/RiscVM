use crate::ops::{self, branch, imm_math, load, reg_math, store};
use crate::region::RegionList;
use crate::inst::{inst_len, Inst32};
use crate::disasm::Disasm32;

pub struct VM<'a> {
    pub regions: RegionList,
    pub ram: &'a mut [u8],
    pub regs: [i32; 32],
    pub ip: i32
}

impl <'a> VM <'a> {
    #[inline]
    pub fn set_rsp(&mut self, rsp: usize)  {
        assert!(rsp < u32::MAX as usize);
        self.regs[2] = rsp as i32;
    }
    pub fn new(regions: RegionList, ram: &'a mut [u8]) -> Self {
        Self { ram, regions, ip: 0, regs: [0; 32] }
    }
    pub fn ip(&self) -> usize {
        self.ip as u32 as usize
    }
    pub fn write(&mut self, mut addr: usize, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let region = match self.regions.find_region(addr) {
                Some(v) => {
                    v
                }
                None => {
                    panic!("Exception: Out of bounds write at {:08X} (ip=0x{:08X}", addr, self.ip);
                }
            }.clone();
            let to_write = bytes.len().min(region.size);
            let (bytes_to_write, left) = bytes.split_at(to_write);
            bytes = left;
            let off = addr-region.addr;
            addr += to_write;
            region.write(self, off, bytes_to_write).expect("Failed to write");
        }
    }

    pub fn read(&mut self, mut addr: usize, mut bytes: &mut [u8]) {
        while !bytes.is_empty() {
            let region = match self.regions.find_region(addr) {
                Some(v) => v,
                None => panic!("Exception: Out of bounds write at {:08X} (ip=0x{:08X})", addr, self.ip),
            }.clone();
            let to_read = bytes.len().min(region.size);
            let (bytes_to_read, left) = bytes.split_at_mut(to_read);
            bytes = left;
            let off = addr-region.addr;
            addr += to_read;
            region.read(self, off, bytes_to_read).expect("Failed to read");
        }
    }
    #[inline]
    pub fn read_u16(&mut self, addr: usize) -> u16 {
        let mut tag_bytes: [u8; 2] = [0; 2];
        self.read(addr, &mut tag_bytes);
        u16::from_le_bytes(tag_bytes)
    }

    #[inline]
    pub fn read_u32(&mut self, addr: usize) -> u32 {
        let mut tag_bytes: [u8; 4] = [0; 4];
        self.read(addr, &mut tag_bytes);
        u32::from_le_bytes(tag_bytes)
    }

    pub fn disasm(&mut self, addr: usize) -> usize {
        let tag = self.read_u16(addr);
        let len = inst_len(tag);
        match len {
            2 => {
                let inst = Inst32::new(self.read_u32(addr));
                println!("{}",Disasm32(inst));
            }
            _ => panic!("Unsupported {} bit Instruction",len*16),
        }
        len
    }
    #[inline]
    pub fn set_reg(&mut self, reg: usize, v: i32) {
        if reg == 0 { return; }
        self.regs[reg] = v;
    }

    #[inline]
    pub fn get_reg(&self, reg: usize) -> i32 {
        if reg == 0 { return 0; }
        self.regs[reg]
    }
    pub fn run(&mut self) {
        let tag = self.read_u16(self.ip());
        let len = inst_len(tag);
        match len {
            2 => {
                let inst = Inst32::new(self.read_u32(self.ip()));
                match inst.opcode() {
                    ops::LUI_OP      => self.set_reg(inst.rd() as usize, inst.imm_U() << 12),
                    ops::AUIPC_OP    => self.set_reg(inst.rd() as usize, self.ip + inst.imm_U() << 12),
                    ops::IMM_MATH_OP => {
                         match inst.funct3() {
                             imm_math::ADDI => {
                                 self.set_reg(inst.rd() as usize, self.get_reg(inst.r1() as usize) + inst.imm_I());
                             }
                             funct3 => {
                                 todo!("Immediate math op 0x{:01X}",funct3)
                             }
                         }
                    }
                    ops::REG_MATH_OP => {
                        match (inst.funct3(), inst.funct7()) {
                            reg_math::ADD => {
                                self.set_reg(inst.rd() as usize, self.get_reg(inst.r1() as usize) + self.get_reg(inst.r2() as usize));
                            }
                            (funct3, funct7) => {
                                panic!("Register math op funct3=0x{:01X} funct7=0x{:02X}", funct3, funct7)
                            }
                        }
                    }
                    ops::STORE_OP => {
                        match inst.funct3() {
                            store::SB => {
                                let v = (self.get_reg(inst.r2() as usize) & 0xFF) as u8;
                                self.write((self.get_reg(inst.r1() as usize) + inst.imm_S()) as u32 as usize, &[v]);
                            }
                            store::SH => {
                                let v = (self.get_reg(inst.r2() as usize) & 0xFFFF) as u16;
                                self.write((self.get_reg(inst.r1() as usize) + inst.imm_S()) as u32 as usize, &v.to_le_bytes());
                            }
                            store::SW => {
                                let v = self.get_reg(inst.r2() as usize);
                                self.write((self.get_reg(inst.r1() as usize) + inst.imm_S()) as u32 as usize, &v.to_le_bytes());
                            }
                            funct3 => {
                                todo!("Unhandled store op funct3=0x{:01X}",funct3);
                            }
                        }
                    }
                    ops::JUMP_OP => {
                        self.set_reg(inst.rd() as usize, self.ip + 4);
                        self.ip += inst.imm_J();
                        return;
                    }
                    ops::JUMP_REG_OP => {
                        self.set_reg(inst.rd() as usize, self.ip + 4);
                        self.ip = self.get_reg(inst.r1() as usize) + inst.imm_I();
                        return;
                    }
                    ops::LOAD_OP => {
                        match inst.funct3() {
                            load::LW => {
                                let v = self.get_reg(inst.r1() as usize);
                                let mut data = [0; 4];
                                self.read((v+inst.imm_I()) as usize, &mut data);
                                self.set_reg(inst.rd() as usize, i32::from_le_bytes(data));
                            }
                            load::LBU => {
                                let v = self.get_reg(inst.r1() as usize);
                                let mut data = [0; 4];
                                self.read((v+inst.imm_I()) as usize, &mut data);
                                self.set_reg(inst.rd() as usize, (u32::from_le_bytes(data) as u8) as i32);
                            }
                            funct3 => todo!("load op with funct3={:01X}", funct3)
                        }
                    }
                    ops::BRANCH_OP => {
                        match inst.funct3() {
                            branch::BEQ => {
                                if self.get_reg(inst.r1() as usize) == self.get_reg(inst.r2() as usize) {
                                    self.ip += inst.imm_B();
                                    return;
                                }
                            }
                            branch::BNE => {
                                if self.get_reg(inst.r1() as usize) != self.get_reg(inst.r2() as usize) {
                                    self.ip += inst.imm_B();
                                    return;
                                }
                            }
                            branch::BGE => {
                                if self.get_reg(inst.r1() as usize) >= self.get_reg(inst.r2() as usize) {
                                    self.ip += inst.imm_B();
                                    return;
                                }
                            }
                            funct3 => {
                                todo!("branch op funct3=0x{:01X}",funct3)
                            }
                        }
                    }
                    op => todo!("op={:07b}",op)
                }
            }
            _ => panic!("Unsupported {} bit Instruction",len*16),
        }
        self.ip += (len * 2) as i32;
    }
    pub fn next(&mut self) {
        self.run();
        // self.ip += (self.disasm(self.ip())*2) as u32;
    }
}
