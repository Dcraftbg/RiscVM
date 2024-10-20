use core::fmt;
use std::{collections::HashSet, env, fs, io::{self, BufRead, Write}, process::{exit, ExitCode}};

// In 16 bit chunks
const fn inst_len(begin: u16) -> usize {
    if begin & 0b11 != 0b11 {
        1
    } else if (begin >> 2) & 0b11 != 0b111 {
        2
    } else if (begin >> 5) & 0b1 != 0b1 {
        4
    } else { // Unsupported
        0
    }
}
// NOTE: Page 11 of manual for Base Instruction Formats
#[derive(Clone, Copy)]
struct Inst32 {
    data: i32
}
#[allow(non_snake_case)]
#[allow(dead_code)]
impl Inst32 {
    #[inline]
    const fn new(data: u32) -> Self {
        Self { data: data as i32 }
    }
    #[inline]
    const fn opcode(self) -> i32 {
        self.data & 0b111_111_1
    }

    #[inline]
    const fn funct3(self) -> i32 {
        (self.data >> 12) & 0b111
    }

    #[inline]
    const fn funct7(self) -> i32 {
        (self.data >> 25) & 0b111_111_1
    }

    #[inline]
    const fn rd(self) -> i32 {
        (self.data >> 7) & 0b11111
    }

    #[inline]
    const fn r1(self) -> i32 {
        (self.data >> 15) & 0b11111
    }

    #[inline]
    const fn r2(self) -> i32 {
        (self.data >> 20) & 0b11111
    }

    #[inline]
    const fn imm_U(self) -> i32 {
        self.data >> 12
    }
    #[inline]
    const fn imm_I(self) -> i32 {
        self.data >> 20
        // self.imm_sign() | ((self.data >> 20) & 0b11111_11111_1)
    }

    #[inline]
    const fn imm_S(self) -> i32 {
        (((self.data >> 7) & 0b11111) ) |
        (((self.data >> 25)) << 5)
    }
    #[inline]
    const fn imm_J(self) -> i32 {
        // NOTE: (<< 12) >> 12 to sign extend the whole integer
        (( 
            (((self.data >> 20 ) & 0b11111111110)      )|
            (((self.data >> 20 ) & 0b00000000001) << 11)|
            (((self.data >> 12 ) & 0b00011111111) << 12)|
            (((self.data >> 30 ) & 0b00000000001) << 20)
        ) << 11) >> 11
        // self.imm_sign() | ((self.data >> 12) & 0b11111_11111_11111_1111)
        // self.data >> 12
        // (((self.data >> 12) & 0b11111111) << 12) | (((self.data >> 18) & 0b1) << 10) | ((self.data >> 19) & 0b1111111111)
    }

    #[inline]
    const fn imm_B(self) -> i32 {
        // NOTE: (<< 20) >> 20 to sign extend the whole integer
        ((
            (((self.data >> 7 ) & 0b011110)      )|
            (((self.data >> 25) & 0b111111) << 5 )|
            (((self.data >> 7 ) & 0b000001) << 11)|
            (((self.data >> 30) & 0b000001) << 12)
        ) << 20 ) >> 20
    }
}
const AUIPC_OP   : i32 = 0b0010111;
const LUI_OP     : i32 = 0b0110111;
const IMM_MATH_OP: i32 = 0b0010011;
mod imm_math {
    pub const ADDI: i32 = 0x0;
}
const REG_MATH_OP: i32 = 0b0110011;
mod reg_math {
    pub const ADD: (i32, i32) = (0x0, 0x0);
}
const STORE_OP   : i32 = 0b0100011;
mod store {
    pub const SB: i32 = 0x0;
    pub const SH: i32 = 0x1;
    pub const SW: i32 = 0x2;
}
const LOAD_OP    : i32 = 0b0000011;
mod load {
    pub const LB : i32 = 0x0;
    pub const LH : i32 = 0x1;
    pub const LW : i32 = 0x2;
    pub const LBU: i32 = 0x4;
    pub const LHU: i32 = 0x5;
}
const JUMP_OP    : i32 = 0b1101111;
const JUMP_REG_OP: i32 = 0b1100111;
mod jump_reg {
    pub const JALR: i32 = 0x0;
}
const BRANCH_OP  : i32 = 0b1100011;
mod branch {
    pub const BEQ : i32 = 0x0;
    pub const BNE : i32 = 0x1;
    pub const BLT : i32 = 0x4;
    pub const BGE : i32 = 0x5;
    pub const BLTU: i32 = 0x6;
    pub const BGEU: i32 = 0x7;
}
struct Off32(i32);
impl fmt::Display for Off32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 >= 0 {
            write!(f, "+{}",self.0)
        } else {
            write!(f, "{}",self.0)
        }
    }
}
struct Disasm32(Inst32);
impl fmt::Display for Disasm32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inst = self.0;
        // TODO: Lookup tables!
        match inst.opcode() {
            AUIPC_OP => {
                write!(f, "auipc x{}, 0x{:X}",inst.rd(),inst.imm_U())
            }
            LUI_OP => {
                write!(f, "lui x{}, 0x{:X}",inst.rd(),inst.imm_U())
                // write!(f, "lui {}, 0x{:X}",inst.rd(),inst.imm_U())
            }
            IMM_MATH_OP => {
                write!(f,
                    "{} x{}, x{}, {}",
                    match inst.funct3() {
                        imm_math::ADDI => "addi",
                        funct3 => return write!(f, "Undisassemblable Immediate math op 0x{:01X}",funct3)
                    }, inst.rd(), inst.r1(), inst.imm_I()
                )
            }
            REG_MATH_OP => {
                write!(f,
                    "{} x{}, x{}, x{}",
                    match (inst.funct3(), inst.funct7()) {
                        reg_math::ADD => "add",
                        (funct3, funct7) => return write!(f, "Undisassemblable Register math op funct3=0x{:01X} funct7=0x{:02X}", funct3, funct7)
                    }, inst.rd(), inst.r1(), inst.r2()
                )
            }
            STORE_OP => {
                write!(f,
                    "{} [x{}{}], x{}", 
                    match inst.funct3() {
                        store::SB => "sb",
                        store::SH => "sh",
                        store::SW => "sw",
                        funct3 => return write!(f, "Undisassemblable store op funct3=0x{:01X}",funct3)
                    }, inst.r1(), Off32(inst.imm_S()), inst.r2()
                )
            }
            LOAD_OP => {
                write!(f, 
                    "{} x{}, [x{}{}]",
                    match inst.funct3() {
                        load::LB  => "lb",
                        load::LH  => "lh",
                        load::LW  => "lw",
                        load::LBU => "lbu",
                        load::LHU => "lhu",
                        funct3 => return write!(f, "Undisassemblable load op funct3=0x{:01X}",funct3)
                    }, inst.rd(), inst.r1(), Off32(inst.imm_I())
                )
            }
            BRANCH_OP => {
                write!(f, 
                    "{} x{}, x{}, {}",
                    match inst.funct3() {
                        branch::BEQ  => "beq",
                        branch::BNE  => "bne",
                        branch::BLT  => "blt",
                        branch::BGE  => "bge",
                        branch::BLTU => "bltu",
                        branch::BGEU => "bgeu",
                        funct3 => return write!(f, "Undisassemblable branch op funct3=0x{:01X}",funct3)
                    }, inst.r1(), inst.r2(), Off32(inst.imm_B())
                )
            }
            JUMP_OP => {
                write!(f, "jal x{}, 0x{:X}", inst.rd(), inst.imm_J())
            }

            JUMP_REG_OP => {
                write!(f,
                    "{} x{} {}",
                    match inst.funct3() {
                        jump_reg::JALR => "jalr",
                        funct3 => return write!(f, "Unknown JALR with funct3={}",funct3)
                    }, inst.r1(), inst.imm_I()
                )
            }
            op => write!(f, "Undisassemblable Opcode {:07b}",op),
        }
    }
}
struct VM<'a> {
    regions: RegionList,
    ram: &'a mut [u8],
    regs: [i32; 32],
    ip: i32
}
impl <'a> VM <'a> {
    #[inline]
    fn set_rsp(&mut self, rsp: usize)  {
        assert!(rsp < u32::MAX as usize);
        self.regs[2] = rsp as i32;
    }
    fn new(ram: &'a mut [u8]) -> Self {
        let ram_len = ram.len();
        Self { ram, regions: RegionList(
                vec![
                    Region {
                        addr: 0,
                        size: ram_len
                    }
                ].into_boxed_slice()
            ), ip: 0, regs: [0; 32] }
    }
    fn ip(&self) -> usize {
        self.ip as u32 as usize
    }
    fn write(&mut self, mut addr: usize, mut bytes: &[u8]) {
        const SERIAL_OUT: usize = 0x6969;
        const EXIT: usize = 0x7000;
        match (addr, bytes.len()) {
            (SERIAL_OUT, 1) => {
                print!("{}", bytes[0] as char);
            }
            (EXIT, 1) => {
                eprintln!("Exiting with {}", bytes[0] as i32);
                exit(bytes[0] as i32)
            }
            _ => {
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
                    let offset = addr-region.addr;
                    addr += to_write;
                    self.ram[region.addr+offset..region.addr+offset+bytes_to_write.len()].copy_from_slice(bytes_to_write);
                }
            }
        }

    }

    fn read(&mut self, mut addr: usize, mut bytes: &mut [u8]) {
        assert!(addr+bytes.len() <= self.ram.len(), "Trying to read to address 0x{:08X}, {} bytes, but RAM is only {} bytes", addr, bytes.len(), self.ram.len());
        while !bytes.is_empty() {
            let region = match self.regions.find_region(addr) {
                Some(v) => {
                    v
                }
                None => {
                    panic!("Exception: Out of bounds write at {:08X} (ip=0x{:08X})", addr, self.ip);
                }
            }.clone();
            let to_read = bytes.len().min(region.size);
            let (bytes_to_read, left) = bytes.split_at_mut(to_read);
            bytes = left;
            let offset = addr-region.addr;
            addr += to_read;
            bytes_to_read.copy_from_slice(&self.ram[region.addr+offset..region.addr+offset+bytes_to_read.len()]);
        }
        bytes.copy_from_slice(&self.ram[addr..addr+bytes.len()])
    }
    #[inline]
    fn read_u16(&mut self, addr: usize) -> u16 {
        let mut tag_bytes: [u8; 2] = [0; 2];
        self.read(addr, &mut tag_bytes);
        u16::from_le_bytes(tag_bytes)
    }

    #[inline]
    fn read_u32(&mut self, addr: usize) -> u32 {
        let mut tag_bytes: [u8; 4] = [0; 4];
        self.read(addr, &mut tag_bytes);
        u32::from_le_bytes(tag_bytes)
    }

    fn disasm(&mut self, addr: usize) -> usize {
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
    fn set_reg(&mut self, reg: usize, v: i32) {
        if reg == 0 { return; }
        self.regs[reg] = v;
    }

    #[inline]
    fn get_reg(&self, reg: usize) -> i32 {
        if reg == 0 { return 0; }
        self.regs[reg]
    }
    fn run(&mut self) {
        let tag = self.read_u16(self.ip());
        let len = inst_len(tag);
        match len {
            2 => {
                let inst = Inst32::new(self.read_u32(self.ip()));
                match inst.opcode() {
                    LUI_OP      => self.set_reg(inst.rd() as usize, inst.imm_U() << 12),
                    AUIPC_OP    => self.set_reg(inst.rd() as usize, self.ip + inst.imm_U() << 12),
                    IMM_MATH_OP => {
                         match inst.funct3() {
                             imm_math::ADDI => {
                                 self.set_reg(inst.rd() as usize, self.get_reg(inst.r1() as usize) + inst.imm_I());
                             }
                             funct3 => {
                                 todo!("Immediate math op 0x{:01X}",funct3)
                             }
                         }
                    }
                    REG_MATH_OP => {
                        match (inst.funct3(), inst.funct7()) {
                            reg_math::ADD => {
                                self.set_reg(inst.rd() as usize, self.get_reg(inst.r1() as usize) + self.get_reg(inst.r2() as usize));
                            }
                            (funct3, funct7) => {
                                panic!("Register math op funct3=0x{:01X} funct7=0x{:02X}", funct3, funct7)
                            }
                        }
                    }
                    STORE_OP => {
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
                    JUMP_OP => {
                        self.set_reg(inst.rd() as usize, self.ip + 4);
                        self.ip += inst.imm_J();
                        return;
                    }
                    JUMP_REG_OP => {
                        self.set_reg(inst.rd() as usize, self.ip + 4);
                        self.ip = self.get_reg(inst.r1() as usize) + inst.imm_I();
                        return;
                    }
                    LOAD_OP => {
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
                    BRANCH_OP => {
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
    fn next(&mut self) {
        self.run();
        // self.ip += (self.disasm(self.ip())*2) as u32;
    }
}
struct Dbg<'a> {
    breakpoints: HashSet<u32>,
    vm: VM<'a>
}
impl <'a> Dbg <'a> {
    #[inline]
    fn new(vm: VM<'a>) -> Self {
        Self { vm, breakpoints: HashSet::new() }
    }

    fn next(&mut self) {
        self.vm.next();
    }
    fn r#continue(&mut self) {
        loop {
            if self.breakpoints.contains(&(self.vm.ip as u32)) {
                return;
            }
            self.next();
        }
    }
    fn disasm(&mut self) {
        eprint!("{:08X}>",self.vm.ip);
        self.vm.disasm(self.vm.ip());
    }
}
const RAM_SIZE: usize = 4096 * 4096;
const STACK_BASE: usize = RAM_SIZE - 0x1000;
#[allow(dead_code)]
struct Build {
    exe: String,
    ipath: String,
    dbg: bool,
}

#[derive(Debug, Clone, Copy)]
struct Region {
    addr: usize,
    size: usize
}
struct RegionList(Box<[Region]>);
impl RegionList {
    fn find_region(&self, addr: usize) -> Option<&Region> {
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
fn main() -> ExitCode {
    let mut args = env::args();
    let mut build = Build {
        exe: args.next().expect("exe"),
        ipath: String::new(),
        dbg: false
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-dbg" => {
                build.dbg = true;
            }
            _ => {
                if build.ipath.is_empty() { build.ipath = arg; }
                else {
                    eprintln!("ERROR: Unknown argument: {}",arg);
                    return ExitCode::FAILURE;
                }
            } 
        }
    }
    if build.ipath.is_empty() {
        eprintln!("ERROR: Missing input path");
        return ExitCode::FAILURE;
    }
    let mut data = match fs::read(&build.ipath) {
        Err(e) => {
            eprintln!("ERROR: Failed to read {}: {}",build.ipath,e);
            return ExitCode::FAILURE;
        }
        Ok(v) => v,
    };
    data.resize(RAM_SIZE.max(data.len()), 0);
    let mut vm = VM::new(&mut data);
    vm.set_rsp(STACK_BASE);
    if build.dbg {
        let mut debugger = Dbg::new(vm);
        let stdin = io::stdin();
        debugger.disasm();
        eprint!(":");
        io::stderr().flush().unwrap();
        let mut lastline = String::new();
        for line in stdin.lock().lines() {
            if let Ok(mut l) = line {
                if l.is_empty() {
                    if lastline.is_empty() { continue; }
                    l = lastline 
                }
                let line = l.as_str();
                let (cmd, arg) = line.split_at(line.find(' ').unwrap_or(line.len()));
                let arg = arg.trim_start();
                match cmd {
                    "n" | "next" => {
                        debugger.next();
                    }
                    "c" | "continue" => {
                        debugger.r#continue();
                    }
                    "b" | "bp" | "break" => {
                        if let Some(hex) = arg.strip_prefix("0x") {
                            match u32::from_str_radix(hex, 16) {
                                Err(e) => {
                                    eprintln!("ERROR: Failed to parse hex literal: {}", e)
                                }
                                Ok(v) => {
                                    eprintln!("Set breakpoint at 0x{:08X}", v);
                                    debugger.breakpoints.insert(v);
                                }
                            }
                        } else {
                            eprintln!("ERROR: Invalid usage of break command:");
                            eprintln!(" b|bp|break <address>");
                            eprintln!("But got argument: {}", arg)
                        }
                    } 
                    "rb" | "delbreakpoint" | "db" => {
                        if let Some(hex) = arg.strip_prefix("0x") {
                            match u32::from_str_radix(hex, 16) {
                                Err(e) => {
                                    eprintln!("ERROR: Failed to parse hex literal: {}", e)
                                }
                                Ok(v) => {
                                    if !debugger.breakpoints.remove(&v) {
                                        eprintln!("ERROR: Breakpoint: 0x{:08X} does not exist",v);
                                    }
                                }
                            }
                        } else {
                            eprintln!("ERROR: Invalid usage of remove break command:");
                            eprintln!(" rb|db|delbreakpoint <address>");
                        }
                    }
                    "d" | "disasm" => {
                        if let Some(hex) = arg.strip_prefix("0x") {
                            match u32::from_str_radix(hex, 16) {
                                Err(e) => {
                                    eprintln!("ERROR: Failed to parse hex literal: {}", e)
                                }
                                Ok(v) => {
                                    eprint!("{:08X}>",v);
                                    debugger.vm.disasm(v as usize);
                                }
                            }
                        } else {
                            eprintln!("ERROR: Invalid usage of disasm command:");
                            eprintln!(" d|disasm <address>");
                        }
                    }
                    "q" | "quit" | "exit" => {
                        break;
                    }
                    "i" | "info" => {
                        match arg {
                            "regs" => {
                                eprintln!("IP={:08X}", debugger.vm.ip);
                                for (i, reg) in debugger.vm.regs.iter().copied().enumerate() {
                                    if i > 0 {
                                        eprint!(" ");
                                        if i % 8 == 0 {
                                            eprintln!()
                                        }
                                    }
                                    eprint!("x{:<2}={:08X}", i, reg);
                                }
                                eprintln!()
                            }
                            _ => {
                                eprintln!("ERROR: Invalid usage of info command with arg: {}", arg);
                                eprintln!(" i|info <regs>");
                            }
                        }
                    }
                    _ => eprintln!("Unknown cmd {}",cmd)
                }
                debugger.disasm();
                eprint!(":");
                io::stderr().flush().unwrap();
                lastline = l;
            }
        }
    } else {
        while vm.ip() < vm.ram.len() {
            vm.next();
        }
    }
    /*
    while data.has_remaining() {
        let tag = data.get_u16_le();
        let len = inst_len(tag);
        assert!(len != 0, "Unsupported Instruction Size");
        match len {
            2 => {
                let high = data.get_u16_le();
                let inst = Inst32::new(((high as u32) << 16) | (tag as u32));
                // println!("Inst entirely: {:032b}",inst.data);
                //println!("Inst with opcode: {:07b} funct3: {:X}",inst.opcode(), inst.funct3());
                println!("Inst: {}",Disasm32(inst));
            }
            _ => panic!("Unsupported {} bit Instruction",len*16),
        }
    }
    */
    ExitCode::SUCCESS
}
