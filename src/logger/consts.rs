pub const BUF_SIZE: usize = 1024;
pub const MODE_MASK: usize = 0b11;
/// Block the application if the RTT buffer is full, wait for the host to read data.
pub const MODE_BLOCK_IF_FULL: usize = 2;
/// Don't block if the RTT buffer is full. Truncate data to output as much as fits.
pub const MODE_NON_BLOCKING_TRIM: usize = 1;
