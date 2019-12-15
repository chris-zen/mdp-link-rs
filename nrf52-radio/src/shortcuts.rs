
bitflags! {
    pub struct Shortcuts: u32 {
        const READY_START = 1 << 0;
        const END_DISABLE = 1 << 1;
        const DISABLED_TXEN = 1 << 2;
        const DISABLED_RXEN = 1 << 3;
        const ADDRESS_RSSISTART = 1 << 4;
        const END_START = 1 << 5;
        const ADDRESS_BCSTART = 1 << 6;
        const DISABLED_RSSISTOP = 1 << 8;
        const RXREADY_CCASTART = 1 << 11;
        const CCAIDLE_TXEN = 1 << 12;
        const CCABUSY_DISABLE = 1 << 13;
        const FRAMESTART_BCSTART = 1 << 14;
        const READY_EDSTART = 1 << 15;
        const EDEND_DISABLE = 1 << 16;
        const CCAIDLE_STOP = 1 << 17;
        const TXREADY_START = 1 << 18;
        const RXREADY_START = 1 << 19;
        const PHYEND_DISABLE = 1 << 20;
        const PHYEND_START = 1 << 21;
    }
}
