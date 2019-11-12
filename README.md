# mdp-link

That's my attempt to write the firmware for the MDP-link with rust.

See [mdp-link-sdk](https://github.com/chris-zen/mdp-link-sdk) and [mdp-link-mynewt](https://github.com/chris-zen/mdp-link-mynewt) for other explorations using C.

## mdp-link-rs

The [MDP-XP](http://www.miniware.com.cn/product/mdp-xp-digital-power-supply-set/) is a Digital Power supply composed of two modules, the M01 screen and the P905 power module.

My goal is to be able to communicate with the power modules from a laptop using USB, without requiring the M01 screen, and be able to record different parameters (V, I, W) over time to build power profiles, or even to graph them on the bigger screen.

The MDP modules use the `nrf24L01+` device for the 2.4GHz wireless communications, in ESB mode. More information about the chipset and the protocol can be found [here](https://infocenter.nordicsemi.com/pdf/nRF24L01P_PS_v1.0.pdf).

For more information about the MDP you can have a look [here](https://www.eevblog.com/forum/testgear/miniware-mdp-xp-digital-power-supply-set/).

I'm using an [nrf52840-mdk](https://wiki.makerdiary.com/nrf52840-mdk/) development kit, it includes a `nrf52840` microcontroller which has a radio that supports ESB.

## nrf52-radio

As part of this project I am developing a HAL for the nrf52840's RADIO peripheral. [See here](nrf52-radio)

It is still in progress and I am working on finding the right interface, while figuring out how to make it work for my purpose.

## nrf52-esb

The MDP devices use a proprietary protocol from Nordic Semiconductors called Enhanced Shock Burst (ESB). I am also working on a crate on top of the radio one to implement that protocol. [See here](nrf52-esb)
