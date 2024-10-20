use core::fmt;

use crate::{inst::Inst32, off::Off32, ops::{self, branch, imm_math, jump_reg, load, reg_math, store}};

pub struct Disasm32(pub Inst32);
impl fmt::Display for Disasm32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inst = self.0;
        // TODO: Lookup tables!
        match inst.opcode() {
            ops::AUIPC_OP => {
                write!(f, "auipc x{}, 0x{:X}",inst.rd(),inst.imm_U())
            }
            ops::LUI_OP => {
                write!(f, "lui x{}, 0x{:X}",inst.rd(),inst.imm_U())
                // write!(f, "lui {}, 0x{:X}",inst.rd(),inst.imm_U())
            }
            ops::IMM_MATH_OP => {
                write!(f,
                    "{} x{}, x{}, {}",
                    match inst.funct3() {
                        imm_math::ADDI => "addi",
                        funct3 => return write!(f, "Undisassemblable Immediate math op 0x{:01X}",funct3)
                    }, inst.rd(), inst.r1(), inst.imm_I()
                )
            }
            ops::REG_MATH_OP => {
                write!(f,
                    "{} x{}, x{}, x{}",
                    match (inst.funct3(), inst.funct7()) {
                        reg_math::ADD => "add",
                        (funct3, funct7) => return write!(f, "Undisassemblable Register math op funct3=0x{:01X} funct7=0x{:02X}", funct3, funct7)
                    }, inst.rd(), inst.r1(), inst.r2()
                )
            }
            ops::STORE_OP => {
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
            ops::LOAD_OP => {
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
            ops::BRANCH_OP => {
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
            ops::JUMP_OP => {
                write!(f, "jal x{}, 0x{:X}", inst.rd(), inst.imm_J())
            }

            ops::JUMP_REG_OP => {
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
