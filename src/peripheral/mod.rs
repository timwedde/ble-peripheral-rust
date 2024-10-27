#[cfg(any(target_os = "macos", target_os = "ios"))]
mod corebluetooth;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use self::corebluetooth::Peripheral;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod bluez;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::bluez::Peripheral;

#[cfg(target_os = "windows")]
mod winrt;
#[cfg(target_os = "windows")]
pub use self::winrt::Peripheral;
