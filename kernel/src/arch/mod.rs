#[cfg(target_arch = "x86_64")]
pub mod x86;

pub fn init() {
    #[cfg(target_arch = "x86_64")]
    x86::init();
}
