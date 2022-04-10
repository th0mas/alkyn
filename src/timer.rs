use rp2040_hal as hal;

pub struct Timer {
    timer: Option<hal::Timer>,
}
