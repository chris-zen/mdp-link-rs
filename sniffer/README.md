# Sniffer

It listens for the ESB packets and sends them to the UART

To build the hex:

```bash
cargo build --release
arm-none-eabi-objcopy -O ihex ../target/thumbv7em-none-eabihf/release/sniffer sniffer.hex
```

Then upload it into the nrf52840-mdk USB dongle using nrf Connect.