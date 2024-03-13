#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum Command {
    ApplyEnergy = 0xbe,
    GetDeviceState = 0xa3,
    GetDeviceInfo = 0xa8,
    UpdateDevice = 0xa9,
    SetDpi = 0xa4,
    Lattice = 0xa6,
    Retract = 0xa0,
    Feed = 0xa1,
    Speed = 0xbd,
    Energy = 0xaf,
    Bitmap = 0xa2,
}


pub enum CommandType {
    Transfer = 0,
    Response = 1,
}