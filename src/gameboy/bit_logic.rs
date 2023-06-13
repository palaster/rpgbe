// BIT CHECKING
// FIRST METHOD byte, bitPosition = byte & (1 << bitPosition)
// SECOND METHOD byte, bitPosition = (byte >> bitposition) & 1

pub(crate) const fn check_bit(byte: u8, position: u8) -> bool { bit_value(byte, position) != 0 }
pub(crate) const fn bit_value(byte: u8, position: u8) -> u8 { (byte >> position) & 1 }

pub(crate) const fn set_bit(byte: u8, position: u8) -> u8 { byte | (1 << position) }
pub(crate) const fn reset_bit(byte: u8, position: u8) -> u8 { byte & (!(1 << position)) }
pub(crate) const fn set_bit_to(on: bool, byte: u8, position: u8) -> u8 {
    if on {
        set_bit(byte, position)
    } else {
        reset_bit(byte, position)
    }
}

pub(crate) const fn compose_bytes(lower: u8, upper: u8) -> u16 {
    ((upper as u16) << 8) | (lower as u16)
}

pub(crate) const fn decompose_bytes(bytes: u16) -> (u8, u8) {
    (bytes as u8, (bytes >> 8) as u8)
}