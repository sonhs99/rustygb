pub enum Flag {
    Z = 0x80,
    S = 0x40,
    H = 0x20,
    C = 0x10,
}

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8,
    pub pc: u16,
    pub sp: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            f: 0xB0,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }
    pub fn set_flag(&mut self, mask: Flag, flag: bool) {
        if flag {
            self.f |= mask as u8;
        } else {
            self.f &= mask as u8 ^ 0xFF;
        }
    }
    pub fn zero(&self) -> bool {
        self.f & Flag::Z as u8 != 0
    }
    pub fn subtract(&self) -> bool {
        self.f & Flag::S as u8 != 0
    }
    pub fn half_carry(&self) -> bool {
        self.f & Flag::H as u8 != 0
    }
    pub fn carry(&self) -> bool {
        self.f & Flag::C as u8 != 0
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = value as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_register_test() {
        let mut reg = Registers::new();

        assert_eq!(reg.af(), 0x01B0);
        assert_eq!(reg.bc(), 0x0013);
        assert_eq!(reg.de(), 0x00D8);
        assert_eq!(reg.hl(), 0x014D);

        reg.set_af(0x2030);
        assert_eq!(reg.af(), 0x2030);
        assert_eq!(reg.a, 0x20);
        assert_eq!(reg.f, 0x30);

        reg.a = 0x40;
        reg.f = 0x50;
        assert_eq!(reg.af(), 0x4050);
    }

    #[test]
    fn flag_register_test() {
        let mut reg = Registers::new();

        assert!(reg.zero());
        assert!(!reg.subtract());
        assert!(reg.half_carry());
        assert!(reg.carry());

        reg.set_flag(Flag::C, false);
        assert!(reg.zero());
        assert!(!reg.subtract());
        assert!(reg.half_carry());
        assert!(!reg.carry());
        assert_eq!(reg.f, 0xA0);
    }
}
