#[derive(Debug)]
pub enum Instruction {
    NOP,
    HALT,
    STOP,
    DI,
    EI,

    JR(Condition),
    JP(Condition),
    JPHL,
    RET(Condition),
    RETI,
    CALL(Condition),

    PUSH(Operand),
    POP(Operand),

    LD(Operand, Operand),
    LD16(Operand, Operand),
    LDOffset(Operand, Operand),

    INC(Operand),
    DEC(Operand),

    ADD(Operand),
    ADDHL(Operand),
    ADDSP,
    ADC(Operand),
    SUB(Operand),
    SBC(Operand),
    AND(Operand),
    OR(Operand),
    XOR(Operand),
    CP(Operand),
    CPL,

    CCF,
    SCF,

    RRA,
    RLA,
    RRCA,
    RLCA,

    RR(Operand),
    RL(Operand),
    RRC(Operand),
    RLC(Operand),

    DAA,

    BIT(u8, Operand),
    SET(u8, Operand),
    RES(u8, Operand),

    SLA(Operand),
    SRA(Operand),
    SWAP(Operand),
    SRL(Operand),

    RST(u8),
    PREFIX,
}

#[derive(Clone, Copy, Debug)]
pub enum Reg8Index {
    A,
    B,
    C,
    D,
    E,
    HL,
    H,
    L,
}
#[derive(Clone, Copy, Debug)]
pub enum Reg16Index {
    BC,
    DE,
    HL,
    HLP,
    HLM,
    SP,
    AF,
}

#[derive(Clone, Copy, Debug)]
pub enum Operand {
    Register8(Reg8Index),
    Register16(Reg16Index),
    Value8,
    Value16,
}

#[derive(Clone, Copy, Debug)]
pub enum Condition {
    NZ = 0,
    Z,
    NC,
    C,
    ALWAYS,
}

const COND_GROUP: [Condition; 4] = [Condition::NZ, Condition::Z, Condition::NC, Condition::C];
const REG8_GROUP: [Operand; 8] = [
    Operand::Register8(Reg8Index::B),
    Operand::Register8(Reg8Index::C),
    Operand::Register8(Reg8Index::D),
    Operand::Register8(Reg8Index::E),
    Operand::Register8(Reg8Index::H),
    Operand::Register8(Reg8Index::L),
    Operand::Register8(Reg8Index::HL),
    Operand::Register8(Reg8Index::A),
];
const REG16_GROUP1: [Operand; 4] = [
    Operand::Register16(Reg16Index::BC),
    Operand::Register16(Reg16Index::DE),
    Operand::Register16(Reg16Index::HL),
    Operand::Register16(Reg16Index::SP),
];
const REG16_GROUP2: [Operand; 4] = [
    Operand::Register16(Reg16Index::BC),
    Operand::Register16(Reg16Index::DE),
    Operand::Register16(Reg16Index::HLP),
    Operand::Register16(Reg16Index::HLM),
];
const REG16_GROUP3: [Operand; 4] = [
    Operand::Register16(Reg16Index::BC),
    Operand::Register16(Reg16Index::DE),
    Operand::Register16(Reg16Index::HL),
    Operand::Register16(Reg16Index::AF),
];

const REG_A: Operand = Operand::Register8(Reg8Index::A);
const REG_HL: Operand = Operand::Register16(Reg16Index::HL);

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::NOP),
            0x10 => Some(Instruction::STOP),
            0x76 => Some(Instruction::HALT),
            0xCB => Some(Instruction::PREFIX),
            0xF3 => Some(Instruction::DI),
            0xFB => Some(Instruction::EI),
            0x01 | 0x11 | 0x21 | 0x31 => Some(Instruction::LD16(
                REG16_GROUP1[(byte >> 4) as usize],
                Operand::Value16,
            )),
            0x02 | 0x12 | 0x22 | 0x32 => {
                Some(Instruction::LD(REG16_GROUP2[(byte >> 4) as usize], REG_A))
            }
            0x03 | 0x13 | 0x23 | 0x33 => Some(Instruction::INC(REG16_GROUP1[(byte >> 4) as usize])),
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                Some(Instruction::INC(REG8_GROUP[(byte >> 3) as usize]))
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                Some(Instruction::DEC(REG8_GROUP[(byte >> 3) as usize]))
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => Some(Instruction::LD(
                REG8_GROUP[(byte >> 3) as usize],
                Operand::Value8,
            )),
            0x07 => Some(Instruction::RLCA),
            0x17 => Some(Instruction::RLA),
            0x27 => Some(Instruction::DAA),
            0x37 => Some(Instruction::SCF),
            0x08 => Some(Instruction::LD16(
                Operand::Value16,
                Operand::Register16(Reg16Index::SP),
            )),
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => Some(Instruction::JR(if byte == 0x18 {
                Condition::ALWAYS
            } else {
                COND_GROUP[((byte >> 3) & 0x03) as usize]
            })),
            0x09 | 0x19 | 0x29 | 0x39 => {
                Some(Instruction::ADDHL(REG16_GROUP1[(byte >> 4) as usize]))
            }
            0x0A | 0x1A | 0x2A | 0x3A => {
                Some(Instruction::LD(REG_A, REG16_GROUP2[(byte >> 4) as usize]))
            }
            0x0B | 0x1B | 0x2B | 0x3B => Some(Instruction::DEC(REG16_GROUP1[(byte >> 4) as usize])),
            0x0F => Some(Instruction::RRCA),
            0x1F => Some(Instruction::RRA),
            0x2F => Some(Instruction::CPL),
            0x3F => Some(Instruction::CCF),
            0x40..=0x75 | 0x77..=0x7F => Some(Instruction::LD(
                REG8_GROUP[((byte >> 3) & 0x07) as usize],
                REG8_GROUP[(byte & 0x07) as usize],
            )),
            0x80..=0x87 | 0xC6 => Some(Instruction::ADD(if byte == 0xC6 {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0x88..=0x8f | 0xCE => Some(Instruction::ADC(if byte == 0xCE {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0x90..=0x97 | 0xD6 => Some(Instruction::SUB(if byte == 0xD6 {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0x98..=0x9F | 0xDE => Some(Instruction::SBC(if byte == 0xDE {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0xA0..=0xA7 | 0xE6 => Some(Instruction::AND(if byte == 0xE6 {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0xA8..=0xAF | 0xEE => Some(Instruction::XOR(if byte == 0xEE {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0xB0..=0xB7 | 0xF6 => Some(Instruction::OR(if byte == 0xF6 {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0xB8..=0xBF | 0xFE => Some(Instruction::CP(if byte == 0xFE {
                Operand::Value8
            } else {
                REG8_GROUP[(byte & 0x07) as usize]
            })),
            0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xC9 => Some(Instruction::RET(if byte == 0xC9 {
                Condition::ALWAYS
            } else {
                COND_GROUP[((byte >> 3) & 0x03) as usize]
            })),
            0xC1 | 0xD1 | 0xE1 | 0xF1 => Some(Instruction::POP(
                REG16_GROUP3[((byte >> 4) & 0x03) as usize],
            )),
            0xC2 | 0xCA | 0xD2 | 0xDA | 0xC3 => Some(Instruction::JP(if byte == 0xC3 {
                Condition::ALWAYS
            } else {
                COND_GROUP[((byte >> 3) & 0x03) as usize]
            })),
            0xC4 | 0xCC | 0xD4 | 0xDC | 0xCD => Some(Instruction::CALL(if byte == 0xCD {
                Condition::ALWAYS
            } else {
                COND_GROUP[((byte >> 3) & 0x03) as usize]
            })),
            0xC5 | 0xD5 | 0xE5 | 0xF5 => Some(Instruction::PUSH(
                REG16_GROUP3[((byte >> 4) & 0x03) as usize],
            )),
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                Some(Instruction::RST(byte - 0xC7))
            }
            0xD9 => Some(Instruction::RETI),
            0xE0 => Some(Instruction::LDOffset(Operand::Value8, REG_A)),
            0xE2 => Some(Instruction::LDOffset(
                Operand::Register8(Reg8Index::C),
                REG_A,
            )),
            0xF0 => Some(Instruction::LDOffset(REG_A, Operand::Value8)),
            0xF2 => Some(Instruction::LDOffset(
                REG_A,
                Operand::Register8(Reg8Index::C),
            )),
            0xF9 => Some(Instruction::LD16(
                Operand::Register16(Reg16Index::SP),
                Operand::Register16(Reg16Index::HL),
            )),
            0xEA => Some(Instruction::LD(Operand::Value16, REG_A)),
            0xFA => Some(Instruction::LD(REG_A, Operand::Value16)),
            0xE8 => Some(Instruction::ADDSP),
            0xF8 => Some(Instruction::LDOffset(
                Operand::Register16(Reg16Index::HL),
                Operand::Value8,
            )),
            0xE9 => Some(Instruction::JPHL),
            _ => None,
        }
    }

    pub fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x00..=0x07 => Some(Instruction::RLC(REG8_GROUP[(byte & 0x07) as usize])),
            0x08..=0x0F => Some(Instruction::RRC(REG8_GROUP[(byte & 0x07) as usize])),
            0x10..=0x17 => Some(Instruction::RL(REG8_GROUP[(byte & 0x07) as usize])),
            0x18..=0x1F => Some(Instruction::RR(REG8_GROUP[(byte & 0x07) as usize])),
            0x20..=0x27 => Some(Instruction::SLA(REG8_GROUP[(byte & 0x07) as usize])),
            0x28..=0x2F => Some(Instruction::SRA(REG8_GROUP[(byte & 0x07) as usize])),
            0x30..=0x37 => Some(Instruction::SWAP(REG8_GROUP[(byte & 0x07) as usize])),
            0x38..=0x3F => Some(Instruction::SRL(REG8_GROUP[(byte & 0x07) as usize])),
            0x40..=0x7F => Some(Instruction::BIT(
                (byte >> 3) & 0x07,
                REG8_GROUP[(byte & 0x07) as usize],
            )),
            0x80..=0xBF => Some(Instruction::RES(
                (byte >> 3) & 0x07,
                REG8_GROUP[(byte & 0x07) as usize],
            )),
            0xC0..=0xFF => Some(Instruction::SET(
                (byte >> 3) & 0x07,
                REG8_GROUP[(byte & 0x07) as usize],
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_defined_inst_count() {
        let mut count = 0;
        for idx in 0x00 as u8..0xFF as u8 {
            if let None = Instruction::from_byte(idx) {
                println!("fail : {:#X}", idx);
                count += 1;
            }
        }
        assert_eq!(count, 11);
    }

    #[test]
    fn not_defined_prefix_inst_count() {
        let mut count = 0;
        for idx in 0x00 as u8..=0xFF as u8 {
            if let None = Instruction::from_byte_prefixed(idx) {
                println!("fail : {:#X}", idx);
                count += 1;
            }
        }
        assert_eq!(count, 0);
    }
}
