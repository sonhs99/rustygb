use crate::inst::{Condition, Instruction, Operand, Reg16Index, Reg8Index};
use crate::mmu::MemoryBus;
use crate::register::{Flag, Registers};

const IO_IF: usize = 0x0F;

pub struct CPU {
    reg: Registers,
    pub bus: MemoryBus,
    cycles: u16,
    IME: bool,
    halt: bool,
}

impl CPU {
    pub fn new(rom: Vec<u8>, save: Vec<u8>) -> CPU {
        CPU {
            reg: Registers::new(),
            bus: MemoryBus::new(rom, save),
            cycles: 0,
            IME: true,
            halt: false,
        }
    }
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            // Single Inst
            Instruction::NOP => {}
            Instruction::HALT => self.halt = true,
            Instruction::STOP => self.halt = true,
            Instruction::DI => self.IME = false,
            Instruction::EI => {
                self.IME = true;
            }

            // Branch / Function Inst
            Instruction::JR(cond) => {
                let dest = self.fetch() as i8 as i16;
                let branch = match cond {
                    Condition::ALWAYS => true,
                    Condition::NZ => !self.reg.zero(),
                    Condition::Z => self.reg.zero(),
                    Condition::NC => !self.reg.carry(),
                    Condition::C => self.reg.carry(),
                };
                if branch {
                    self.reg.pc = self.reg.pc.wrapping_add(dest as u16);
                    self.tick();
                }
            }
            Instruction::JP(cond) => {
                let dest = self.read_word_pc();
                let branch = match cond {
                    Condition::ALWAYS => true,
                    Condition::NZ => !self.reg.zero(),
                    Condition::Z => self.reg.zero(),
                    Condition::NC => !self.reg.carry(),
                    Condition::C => self.reg.carry(),
                };
                if branch {
                    self.reg.pc = dest;
                    self.tick();
                }
            }
            Instruction::JPHL => self.reg.pc = self.reg.hl(),
            Instruction::RET(cond) => {
                let branch = match cond {
                    Condition::ALWAYS => true,
                    Condition::NZ => !self.reg.zero(),
                    Condition::Z => self.reg.zero(),
                    Condition::NC => !self.reg.carry(),
                    Condition::C => self.reg.carry(),
                };
                if branch {
                    self.reg.pc = self.pop();
                }
                self.tick();
            }
            Instruction::RETI => {
                self.IME = true;
                self.reg.pc = self.pop();
                self.tick()
            }
            Instruction::CALL(cond) => {
                let dest = self.read_word_pc();
                let branch = match cond {
                    Condition::ALWAYS => true,
                    Condition::NZ => !self.reg.zero(),
                    Condition::Z => self.reg.zero(),
                    Condition::NC => !self.reg.carry(),
                    Condition::C => self.reg.carry(),
                };
                if branch {
                    self.push(self.reg.pc);
                    self.reg.pc = dest;
                }
            }
            Instruction::PUSH(op) => {
                let value = self.read_operand(op);
                self.push(value);
            }
            Instruction::POP(op) => {
                let value = self.pop();
                self.write_operand16(op, value);
            }

            // Load Inst
            Instruction::LD(dest, src) => {
                let mut value = self.read_operand(src);
                match src {
                    Operand::Register16(_) | Operand::Value16 => {
                        value = self.read_byte(value) as u16;
                    }
                    _ => {}
                }
                self.write_operand8(dest, value as u8);
            }
            Instruction::LD16(dest, src) => {
                let value = self.read_operand(src);
                self.write_operand16(dest, value);
                if let Operand::Register16(Reg16Index::SP) = dest {
                    if let Operand::Register16(Reg16Index::HL) = src {
                        self.tick();
                    }
                }
            }
            Instruction::LDOffset(dest, src) => {
                let value = self.read_operand(src);
                match dest {
                    Operand::Register16(Reg16Index::HL) => {
                        let value = self.reg.sp.wrapping_add(value as u8 as i8 as i16 as u16);
                        self.reg.set_hl(value);
                        self.tick();
                    }
                    Operand::Register8(Reg8Index::A) => {
                        let value = value.wrapping_add(0xFF00);
                        let value = self.read_byte(value);
                        self.reg.a = value;
                    }
                    Operand::Register8(Reg8Index::C) | Operand::Value8 => {
                        let addr = self.read_operand(dest);
                        let addr = addr.wrapping_add(0xFF00);
                        self.write_byte(addr, value as u8);
                    }
                    _ => {}
                }
            }
            Instruction::INC(op) => {
                let value = self.read_operand(op);
                match op {
                    Operand::Register16(_) => {
                        self.write_operand16(op, value.wrapping_add(1));
                        self.tick();
                    }
                    Operand::Register8(_) => {
                        let value = (value as u8).wrapping_add(1);
                        self.write_operand8(op, value);
                        self.reg.set_flag(Flag::Z, value == 0);
                        self.reg.set_flag(Flag::S, false);
                        self.reg.set_flag(Flag::H, value & 0xF == 0);
                    }
                    _ => {}
                }
            }
            Instruction::DEC(op) => {
                let value = self.read_operand(op);
                match op {
                    Operand::Register16(_) => {
                        self.write_operand16(op, value.wrapping_sub(1));
                        self.tick();
                    }
                    Operand::Register8(_) => {
                        let value = (value as u8).wrapping_sub(1);
                        self.write_operand8(op, value);
                        self.reg.set_flag(Flag::Z, value == 0);
                        self.reg.set_flag(Flag::S, true);
                        self.reg.set_flag(Flag::H, value & 0xF == 0xF);
                    }
                    _ => {}
                }
            }

            // ALU Inst
            Instruction::ADDHL(op) => {
                let value = self.read_operand(op);
                let (new_value, carry) = self.reg.hl().overflowing_add(value);
                self.reg.set_flag(Flag::S, false);
                self.reg
                    .set_flag(Flag::H, (value & 0xF00) + (self.reg.hl() & 0xF00) > 0xF00);
                self.reg.set_flag(Flag::C, carry);
                self.reg.set_hl(new_value);
                self.tick();
            }
            Instruction::ADDSP => {
                let value = self.fetch() as i8 as i16 as u16;
                let (new_value, carry) = self.reg.sp.overflowing_add(value);
                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::S, false);
                self.reg
                    .set_flag(Flag::H, (value & 0xF00) + (self.reg.sp & 0xF00) > 0xF00);
                self.reg.set_flag(Flag::C, carry);
                self.reg.sp = new_value;
            }
            Instruction::ADD(op) => {
                let value = self.read_operand(op) as u8;
                let (new_value, carry) = self.reg.a.overflowing_add(value);
                self.reg.set_flag(Flag::Z, new_value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg
                    .set_flag(Flag::H, (value & 0x0F) + (self.reg.a & 0x0F) > 0xF);
                self.reg.set_flag(Flag::C, carry);
                self.reg.a = new_value;
            }
            Instruction::ADC(op) => {
                let value = self.read_operand(op) as u8;
                let (new_value, carry) = self.reg.a.overflowing_add(value);
                self.reg.set_flag(Flag::Z, new_value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(
                    Flag::H,
                    (value & 0x0F) + (self.reg.a & 0x0F) + carry as u8 > 0xF,
                );
                self.reg.set_flag(Flag::C, carry);
                self.reg.a = new_value.wrapping_add(carry as u8);
            }
            Instruction::SUB(op) => {
                let value = self.read_operand(op) as u8;
                let (new_value, carry) = self.reg.a.overflowing_sub(value);
                self.reg.set_flag(Flag::Z, new_value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg
                    .set_flag(Flag::H, (value & 0x0F) > (self.reg.a & 0x0F));
                self.reg.set_flag(Flag::C, carry);
                self.reg.a = new_value
            }
            Instruction::SBC(op) => {
                let value = self.read_operand(op) as u8;
                let (new_value, carry) = self.reg.a.overflowing_sub(value);
                self.reg.set_flag(Flag::Z, new_value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg
                    .set_flag(Flag::H, (value & 0x0F) > (self.reg.a & 0x0F));
                self.reg.set_flag(Flag::C, carry);
                self.reg.a = new_value.wrapping_sub(carry as u8);
            }
            Instruction::AND(op) => {
                let value = self.read_operand(op) as u8;
                self.reg.a &= value;
                self.reg.set_flag(Flag::Z, self.reg.a == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, true);
                self.reg.set_flag(Flag::C, false);
            }
            Instruction::XOR(op) => {
                let value = self.read_operand(op) as u8;
                self.reg.a ^= value;
                self.reg.set_flag(Flag::Z, self.reg.a == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, false);
            }
            Instruction::OR(op) => {
                let value = self.read_operand(op) as u8;
                self.reg.a |= value;
                self.reg.set_flag(Flag::Z, self.reg.a == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, false);
            }
            Instruction::CP(op) => {
                let value = self.read_operand(op) as u8;
                let (new_value, carry) = self.reg.a.overflowing_sub(value);
                self.reg.set_flag(Flag::Z, new_value == 0);
                self.reg.set_flag(Flag::S, true);
                self.reg
                    .set_flag(Flag::H, (value & 0x0F) > (self.reg.a & 0x0F));
                self.reg.set_flag(Flag::C, carry);
            }
            Instruction::CPL => {
                self.reg.a = !self.reg.a;
                self.reg.set_flag(Flag::S, true);
                self.reg.set_flag(Flag::H, true);
            }

            // Carry Inst
            Instruction::CCF => {
                self.reg.set_flag(Flag::C, !self.reg.carry());
            }
            Instruction::SCF => {
                self.reg.set_flag(Flag::C, true);
            }

            // Shift Inst
            Instruction::RLCA => {
                let carry = self.reg.a >> 7;
                self.reg.a = (self.reg.a << 1) | carry;
                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
            }
            Instruction::RRCA => {
                let carry = self.reg.a & 0x01;
                self.reg.a = (self.reg.a >> 1) | (carry << 7);
                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
            }
            Instruction::RLA => {
                let carry = self.reg.a >> 7;
                self.reg.a = (self.reg.a << 1) + self.reg.carry() as u8;
                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
            }
            Instruction::RRA => {
                let carry = self.reg.a & 0x01;
                self.reg.a = (self.reg.a >> 1) + ((self.reg.carry() as u8) << 7);
                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
            }
            Instruction::RLC(op) => {
                let value = self.read_operand(op) as u8;
                let carry = value >> 7;
                let value = (value << 1) | carry;
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
                self.write_operand8(op, value);
            }
            Instruction::RRC(op) => {
                let value = self.read_operand(op) as u8;
                let carry = value & 0x01;
                let value = (value >> 1) | (carry << 7);
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
                self.write_operand8(op, value);
            }
            Instruction::RL(op) => {
                let value = self.read_operand(op) as u8;
                let carry = value >> 7;
                let value = (value << 1) + self.reg.carry() as u8;
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
                self.write_operand8(op, value);
            }
            Instruction::RR(op) => {
                let value = self.read_operand(op) as u8;
                let carry = value & 0x01;
                let value = (value >> 1) + ((self.reg.carry() as u8) << 7);
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
                self.write_operand8(op, value);
            }
            Instruction::SLA(op) => {
                let value = self.read_operand(op) as u8;
                let carry = value >> 7;
                let value = value << 1;
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
                self.write_operand8(op, value);
            }
            Instruction::SRA(op) => {
                let value = self.read_operand(op) as u8;
                let carry = value & 0x01;
                let value = (value as i8) >> 1;
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
                self.write_operand8(op, value as u8);
            }
            Instruction::SRL(op) => {
                let value = self.read_operand(op) as u8;
                let carry = value & 0x01;
                let value = value >> 1;
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry != 0);
                self.write_operand8(op, value);
            }
            Instruction::SWAP(op) => {
                let value = self.read_operand(op) as u8;
                let value = (value >> 4) | (value << 4);
                self.reg.set_flag(Flag::Z, value == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, false);
                self.write_operand8(op, value);
            }

            // Bit Inst
            Instruction::BIT(bit, op) => {
                let value = self.read_operand(op) as u8;
                self.reg.set_flag(Flag::Z, value & (0x01 << bit) == 0);
                self.reg.set_flag(Flag::S, false);
                self.reg.set_flag(Flag::H, true);
            }
            Instruction::RES(bit, op) => {
                let value = self.read_operand(op) as u8;
                self.write_operand8(op, value & !(0x01 << bit));
            }
            Instruction::SET(bit, op) => {
                let value = self.read_operand(op) as u8;
                self.write_operand8(op, value | (0x01 << bit));
            }

            // Etc
            Instruction::RST(addr) => {
                self.push(self.reg.pc);
                self.reg.pc = addr as u16;
            }
            Instruction::DAA => {
                let (mut offset, mut carry) = (0 as u8, false);
                if self.reg.half_carry() || (!self.reg.subtract() && self.reg.a & 0x0F > 0x09) {
                    offset = 0x06;
                }
                if self.reg.carry() || (!self.reg.subtract()) && self.reg.a > 0x99 {
                    offset |= 0x60;
                    carry = true;
                }
                self.reg.a = if self.reg.subtract() {
                    self.reg.a.wrapping_add(offset)
                } else {
                    self.reg.a.wrapping_sub(offset)
                };
                self.reg.set_flag(Flag::Z, self.reg.a == 0);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, carry);
            }
            _ => {}
        }
        self.reg.pc
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        self.tick();
        self.bus
            .read_byte(address)
            .unwrap_or_else(|| panic!("Cannot read Memory from {:#04X}", address))
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.tick();
        self.bus
            .write_byte(address, value)
            .unwrap_or_else(|| panic!("Cannot write Memory to {:#04X}", address))
    }

    fn read_word(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address.wrapping_add(1)) as u16;
        high << 8 | low
    }

    fn read_word_pc(&mut self) -> u16 {
        let byte = self.read_word(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(2);
        byte
    }

    fn fetch(&mut self) -> u8 {
        let byte = self.read_byte(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        byte
    }

    fn push(&mut self, value: u16) {
        self.write_byte(self.reg.sp.wrapping_sub(2), (value & 0xFF) as u8);
        self.write_byte(self.reg.sp.wrapping_sub(1), (value >> 8) as u8);
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        self.tick();
    }
    fn pop(&mut self) -> u16 {
        let word = self.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        word
    }

    fn read_operand(&mut self, operand: Operand) -> u16 {
        match operand {
            Operand::Register16(reg) => match reg {
                Reg16Index::AF => self.reg.af(),
                Reg16Index::BC => self.reg.bc(),
                Reg16Index::DE => self.reg.de(),
                Reg16Index::HL => self.reg.hl(),
                Reg16Index::HLP => {
                    let value = self.reg.hl();
                    self.reg.set_hl(self.reg.hl().wrapping_add(1));
                    value
                }
                Reg16Index::HLM => {
                    let value = self.reg.hl();
                    self.reg.set_hl(self.reg.hl().wrapping_sub(1));
                    value
                }
                Reg16Index::SP => self.reg.sp,
            },
            Operand::Value16 => self.read_word_pc(),
            Operand::Register8(reg) => {
                (match reg {
                    Reg8Index::A => self.reg.a,
                    Reg8Index::B => self.reg.b,
                    Reg8Index::C => self.reg.c,
                    Reg8Index::D => self.reg.d,
                    Reg8Index::E => self.reg.e,
                    Reg8Index::H => self.reg.h,
                    Reg8Index::L => self.reg.l,
                    Reg8Index::HL => self.read_byte(self.reg.hl()),
                }) as u16
            }
            Operand::Value8 => self.fetch() as u16,
        }
    }

    fn write_operand8(&mut self, operand: Operand, value: u8) {
        match operand {
            Operand::Register16(reg) => match reg {
                Reg16Index::AF => self.write_byte(self.reg.af(), value),
                Reg16Index::BC => self.write_byte(self.reg.bc(), value),
                Reg16Index::DE => self.write_byte(self.reg.de(), value),
                Reg16Index::HL => self.write_byte(self.reg.hl(), value),
                Reg16Index::HLP => {
                    self.write_byte(self.reg.hl(), value);
                    self.reg.set_hl(self.reg.hl().wrapping_add(1));
                }
                Reg16Index::HLM => {
                    self.write_byte(self.reg.hl(), value);
                    self.reg.set_hl(self.reg.hl().wrapping_sub(1));
                }
                Reg16Index::SP => self.write_byte(self.reg.sp, value),
            },
            Operand::Value16 => {
                let address = self.read_word_pc();
                self.write_byte(address, value);
            }
            Operand::Register8(reg) => match reg {
                Reg8Index::A => self.reg.a = value,
                Reg8Index::B => self.reg.b = value,
                Reg8Index::C => self.reg.c = value,
                Reg8Index::D => self.reg.d = value,
                Reg8Index::E => self.reg.e = value,
                Reg8Index::H => self.reg.h = value,
                Reg8Index::L => self.reg.l = value,
                Reg8Index::HL => self.write_byte(self.reg.hl(), value as u8),
            },
            Operand::Value8 => panic!("Write to byte address is not defined!"),
        }
    }

    fn write_operand16(&mut self, operand: Operand, value: u16) {
        match operand {
            Operand::Register16(reg) => match reg {
                Reg16Index::AF => self.reg.set_af(value),
                Reg16Index::BC => self.reg.set_bc(value),
                Reg16Index::DE => self.reg.set_de(value),
                Reg16Index::HL => self.reg.set_hl(value),
                Reg16Index::HLP => self.reg.set_hl(value),
                Reg16Index::HLM => self.reg.set_hl(value),
                Reg16Index::SP => self.reg.sp = value,
            },
            Operand::Value16 => {
                let address = self.read_word_pc();
                self.write_byte(address, value as u8);
                self.write_byte(address.wrapping_add(1), (value >> 8) as u8);
            }
            _ => {}
        }
    }

    pub fn step(&mut self) -> u16 {
        self.cycles = 0;
        // println!(
        //     "IME {}, IF {:02X}, IE {:02X}",
        //     self.IME,
        //     self.bus.io[IO_IF],
        //     self.bus.get_ie()
        // );
        if self.IME && (self.bus.io[IO_IF] & self.bus.get_ie()) != 0 {
            // println!("Interrupt Recieved");/
            self.IME = false;
            self.halt = false;
            self.bus.io[IO_IF] = 0;
            self.cycles += 8;
            self.push(self.reg.pc);
            self.reg.pc = 64;
        } else if self.halt {
            self.tick();
        } else {
            let prev_pc = self.reg.pc;
            let mut instruction_byte = self.fetch();
            let mut instruction = Instruction::from_byte(instruction_byte).unwrap_or_else(|| {
                panic!("Unknown instrcution found for 0x{:x}", instruction_byte)
            });
            if let Instruction::PREFIX = instruction {
                instruction_byte = self.fetch();
                instruction =
                    Instruction::from_byte_prefixed(instruction_byte).unwrap_or_else(|| {
                        panic!("Unknown instrcution found for 0x{:x}", instruction_byte)
                    });
            }
            // println!(
            //     "pc = {:04X}, sp = {:04X}, af = {:04X}, bc = {:04X}, de = {:04X}, hl = {:04X}, inst = {:02X} : {:?}",
            //     prev_pc,
            //     self.reg.sp,
            //     self.reg.af(),
            //     self.reg.bc(),
            //     self.reg.de(),
            //     self.reg.hl(),
            //     instruction_byte,
            //     instruction
            // );
            self.execute(instruction);
        }
        self.cycles
    }

    fn tick(&mut self) {
        self.cycles += self.cycles.wrapping_add(4);
    }
}
