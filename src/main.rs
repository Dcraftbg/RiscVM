use std::{env, fs, io::{self, BufRead, Write}, process::ExitCode};
use region::RegionList;
mod region;
mod inst;
mod off;
mod ops;
mod vm;
mod disasm;
mod dbg;
mod setup;
mod simple;

#[allow(dead_code)]
struct Build {
    exe: String,
    ipath: String,
    dbg: bool,
}

enum Machine {
    Simple,
}
fn main() -> ExitCode {
    let mut args = env::args();
    let mut build = Build {
        exe: args.next().expect("exe"),
        ipath: String::new(),
        dbg: false
    };
    let mut machine = Machine::Simple;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-dbg" => build.dbg = true,
            arg if arg.starts_with("-m") => {
                let arg = arg.strip_prefix("-m").unwrap();
                machine = match arg {
                    "s" | "simple" => Machine::Simple,
                    _ => {
                        eprintln!("ERROR: Unknown machine: `{}`", arg);
                        return ExitCode::FAILURE;
                    }
                }
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
    let setup = match machine {
        Machine::Simple => simple::setup(&mut data),
    };
    let mut vm = vm::VM::new(&setup.layout, &mut data);
    vm.set_rsp(setup.sp);
    if build.dbg {
        let mut debugger = dbg::Dbg::new(vm);
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
