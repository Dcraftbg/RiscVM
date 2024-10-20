pub const AUIPC_OP   : i32 = 0b0010111;
pub const LUI_OP     : i32 = 0b0110111;
pub const IMM_MATH_OP: i32 = 0b0010011;
pub mod imm_math {
    pub const ADDI: i32 = 0x0;
}
pub const REG_MATH_OP: i32 = 0b0110011;
pub mod reg_math {
    pub const ADD: (i32, i32) = (0x0, 0x0);
}
pub const STORE_OP   : i32 = 0b0100011;
pub mod store {
    pub const SB: i32 = 0x0;
    pub const SH: i32 = 0x1;
    pub const SW: i32 = 0x2;
}
pub const LOAD_OP    : i32 = 0b0000011;
pub mod load {
    pub const LB : i32 = 0x0;
    pub const LH : i32 = 0x1;
    pub const LW : i32 = 0x2;
    pub const LBU: i32 = 0x4;
    pub const LHU: i32 = 0x5;
}
pub const JUMP_OP    : i32 = 0b1101111;
pub const JUMP_REG_OP: i32 = 0b1100111;
pub mod jump_reg {
    pub const JALR: i32 = 0x0;
}
pub const BRANCH_OP  : i32 = 0b1100011;
pub mod branch {
    pub const BEQ : i32 = 0x0;
    pub const BNE : i32 = 0x1;
    pub const BLT : i32 = 0x4;
    pub const BGE : i32 = 0x5;
    pub const BLTU: i32 = 0x6;
    pub const BGEU: i32 = 0x7;
}
