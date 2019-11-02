/*!

Possible states for the Radio

See [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf): 6.20.5 Radio states

*/

pub struct Disabled;

pub struct RxRumpUp<'a>(pub &'a mut [u8]);
pub struct RxIdle<'a>(pub &'a mut [u8]);
pub struct Rx<'a>(pub &'a mut [u8]);
pub struct RxDisable<'a>(pub &'a mut [u8]);

pub struct TxDisable;
pub struct TxRumpUp;
pub struct TxIdle;
pub struct Tx;
