// In 16 bit chunks
pub const fn inst_len(begin: u16) -> usize {
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
pub struct Inst32 {
    pub data: i32
}
#[allow(non_snake_case)]
#[allow(dead_code)]
impl Inst32 {
    #[inline]
    pub const fn new(data: u32) -> Self {
        Self { data: data as i32 }
    }
    #[inline]
    pub const fn opcode(self) -> i32 {
        self.data & 0b111_111_1
    }

    #[inline]
    pub const fn funct3(self) -> i32 {
        (self.data >> 12) & 0b111
    }

    #[inline]
    pub const fn funct7(self) -> i32 {
        (self.data >> 25) & 0b111_111_1
    }

    #[inline]
    pub const fn rd(self) -> i32 {
        (self.data >> 7) & 0b11111
    }

    #[inline]
    pub const fn r1(self) -> i32 {
        (self.data >> 15) & 0b11111
    }

    #[inline]
    pub const fn r2(self) -> i32 {
        (self.data >> 20) & 0b11111
    }

    #[inline]
    pub const fn imm_U(self) -> i32 {
        self.data >> 12
    }
    #[inline]
    pub const fn imm_I(self) -> i32 {
        self.data >> 20
        // self.imm_sign() | ((self.data >> 20) & 0b11111_11111_1)
    }

    #[inline]
    pub const fn imm_S(self) -> i32 {
        (((self.data >> 7) & 0b11111) ) |
        (((self.data >> 25)) << 5)
    }
    #[inline]
    pub const fn imm_J(self) -> i32 {
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
    pub const fn imm_B(self) -> i32 {
        // NOTE: (<< 20) >> 20 to sign extend the whole integer
        ((
            (((self.data >> 7 ) & 0b011110)      )|
            (((self.data >> 25) & 0b111111) << 5 )|
            (((self.data >> 7 ) & 0b000001) << 11)|
            (((self.data >> 30) & 0b000001) << 12)
        ) << 20 ) >> 20
    }
}
