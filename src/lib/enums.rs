// Taken from https://github.com/idubrov/lcd

#[derive(Copy, Clone, Debug)]
pub enum FunctionMode {
    /// Send data 4 bits at the time
    Bit4 = 0x00,
    /// Send data 8 bits at the time
    Bit8 = 0x10
}

#[derive(Copy, Clone, Debug)]
pub enum FunctionDots {
    Dots5x8 = 0x00,
    Dots5x10 = 0x04
}

#[derive(Copy, Clone, Debug)]
pub enum FunctionLine {
    Line1 = 0x00,
    Line2 = 0x08
}

#[derive(Copy, Clone, Debug)]
pub enum DisplayBlink {
    BlinkOff = 0x00,
    BlinkOn = 0x01
}

#[derive(Copy, Clone, Debug)]
pub enum DisplayCursor {
    CursorOff = 0x00,
    CursorOn = 0x02
}

#[derive(Copy, Clone, Debug)]
pub enum DisplayMode {
    DisplayOff = 0x00,
    DisplayOn = 0x04
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Left = 0x00,
    Right = 0x04
}

#[derive(Copy, Clone, Debug)]
pub enum Scroll {
    CursorMove = 0x00,
    DisplayMove = 0x08
}

#[derive(Copy, Clone, Debug)]
pub enum EntryModeDirection {
    EntryLeft = 0x00,
    EntryRight = 0x02
}

#[derive(Copy, Clone, Debug)]
pub enum EntryModeShift {
    NoShift = 0x00,
    Shift = 0x01
}

#[derive(Copy, Clone, Debug)]
pub enum Command {
    ClearDisplay = 0x01,
    ReturnHome = 0x02,
    EntryModeSet = 0x04,
    DisplayControl = 0x08,
    CursorShift = 0x10,
    FunctionSet = 0x20,
    SetCGRamAddr = 0x40,
    SetDDRamAddr = 0x80
}

pub const En: u8 = 0b00000100;