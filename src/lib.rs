// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! # picoboot
//! 
//! A crate for connecting to and communicating with RP2040/RP2350
//! microcontrollers using the PICOBOOT USB interface.
//! 
//! ## Getting Started
//! 
//! 1. Use [`Picoboot`] to find a PICOBOOT device.
//! 
//! 2. Use [`Picoboot::connect()`] to connect to the device and get a [`Connection`]
//! 
//! 3. Use the [`Connection`] to interact with the device, such as
//!    reading/writing/erasing flash memory.
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use picoboot::{Picoboot, Access, Error};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // Create connection to first RP2040/RP2350 found
//!     let mut picoboot = Picoboot::new(None).await?;
//!     let conn = picoboot.connect().await?;
//!
//!     // Claim exclusive access, ejecting the BOOTSEL mass storage device and
//!     // exit XIP mode (only necessary on RP2040)
//!     conn.set_exclusive_access(Access::ExclusiveAndEject).await?;
//!     conn.exit_xip().await?;
//! 
//!     // Erase first 4096 of flash (1 sector)
//!     conn.flash_erase_start(4096).await?;
//! 
//!     // Write 256 bytes of data to start of flash (1 page)
//!     conn.flash_write_start(&[0u8; 256]).await?;
//! 
//!     // Retrieve 256 bytes of data from start of flash
//!     let data = conn.flash_read_start(256).await?;
//!
//!     Ok(())
//! }
//! ```
//! 
//! ## Crate Overview
//! 
//! - High level APIs for common operations: reading/writing/erasing flash
//! - Low level APIs for sending/receiving PICOBOOT commands
//! - Support for both RP2040 and RP2350 targets
//! - Support for custom VID/PID targets (RP2040/RP2350 that have been OTP
//!   programmed)
//! - Supports `async`/`await` using `tokio` and `smol` runtimes
//! - Native Rust USB implementation using `nusb` - no `libusb` dependency
//! 
//! ## PICOBOOT Overview
//! 
//! The RP2040/RP2350 provide two mechanisms out of the box for flashing
//! firmware when in BOOTSEL mode:
//! - UF2 mass storage device interface - copy UF2 files to the mounted drive
//! - PICOBOOT USB interface - send commands to the device to read/write/erase
//!   flash, and other operations
//! 
//! PICOBOOT is more flexible than the UF2 mass storage interface, especially
//! for programmatic reading and writing of firmware images, as it:
//! - does not require superuser access to mount/unmount drives
//! - it enables platform independent code, as there are no filesystem access
//!   differences to handle
//! - gives greater control over the device, including reading back flash
//!   contents, erasing individual flash sectors, and writing individual flash
//!   pages, reading RAM, resetting the device, etc
//! 
//! For more details on PICOBOOT, see the RP2040 and RP2350 datasheets.
//! 
//! ## Feature Flags
//! 
//! Used to indicate the desired async runtime for `nusb`.
//! 
//! ```toml
//! default = ["tokio", "logging"]
//! tokio = ["nusb/tokio"]
//! smol = ["nusb/smol"]
//! logging = ["deku/logging"]
//! ```
//! 
//! ## Acknowledgement
//! 
//! This crate was based on [`picoboot-rs`](https://crates.io/crates/picoboot-rs) by Hickok-Dickson and
//! provides:
//! - additional, simplified APIs
//! - stronger typing
//! - async/await support
//! - native Rust USB using `nusb`
//! - safer serialization/deserialization using `deku`

/// RP MCU memory address for the start of ROM storage
pub const ROM_START: u32 = 0x00000000;
/// RP2040 memory address for the end of ROM storage
pub const ROM_END_RP2040: u32 = 0x00004000;
/// RP2350 memory address for the end of ROM storage
pub const ROM_END_RP2350: u32 = 0x00008000;

/// RP MCU memory address for the start of flash storage
pub const FLASH_START: u32 = 0x10000000;
/// RP2040 memory address for the end of flash storage
pub const FLASH_END_RP2040: u32 = 0x11000000;
/// RP2350 memory address for the end of flash storage
pub const FLASH_END_RP2350: u32 = 0x12000000;

/// RP2040 memory address for the start of XIP (execute-in-place) SRAM storage
pub const XIP_SRAM_START_RP2040: u32 = 0x15000000;
/// RP2040 memory address for the end of XIP (execute-in-place) SRAM storage
pub const XIP_SRAM_END_RP2040: u32 = 0x15004000;
/// RP2350 memory address for the start of XIP (execute-in-place) SRAM storage
pub const XIP_SRAM_START_RP2350: u32 = 0x13ffc000;
/// RP2350 memory address for the end of XIP (execute-in-place) SRAM storage
pub const XIP_SRAM_END_RP2350: u32 = 0x14000000;

/// RP MCU memory address for the start of SRAM storage
pub const SRAM_START_RP2040: u32 = 0x20000000;
/// RP2040 memory address for the end of SRAM storage
pub const SRAM_END_RP2040: u32 = 0x20042000;
/// RP2350 memory address for the end of SRAM storage
pub const SRAM_END_RP2350: u32 = 0x20082000;

/// RP MCU flash page size (for writing) - 256 bytes
pub const PAGE_SIZE: u32 = 0x100;
/// RP MCU flash sector size (for erasing) - 4096 bytes
pub const SECTOR_SIZE: u32 = 0x1000;
/// RP2040 memory address for the initial stack pointer
pub const STACK_POINTER_RP2040: u32 = SRAM_END_RP2040;
/// RP2350 memory address for the initial stack pointer
pub const STACK_POINTER_RP2350: u32 = SRAM_END_RP2350;

/// RP USB Vendor ID
pub const PICOBOOT_VID: u16 = 0x2E8A;
/// RP2040 USB Product ID
pub const PICOBOOT_PID_RP2040: u16 = 0x0003;
/// RP2350 USB Product ID
pub const PICOBOOT_PID_RP2350: u16 = 0x000f;

/// RP MCU magic number for USB interfacing
pub const PICOBOOT_MAGIC: u32 = 0x431FD10B;

/// UF2 Family ID for RP2040
pub const UF2_RP2040_FAMILY_ID: u32 = 0xE48BFF56;
pub const UF2_ABSOLUTE_FAMILY_ID: u32 = 0xE48BFF57;
pub const UF2_DATA_FAMILY_ID: u32 = 0xE48BFF58;
/// UF2 Family ID for RP2350 (ARM, Secure TrustZone)
pub const UF2_RP2350_ARM_S_FAMILY_ID: u32 = 0xE48BFF59;
/// UF2 Family ID for RP2350 (RISC-V)
pub const UF2_RP2350_RISCV_FAMILY_ID: u32 = 0xE48BFF5A;
/// UF2 Family ID for RP2350 (ARM, Non-Secure TrustZone)
pub const UF2_RP2350_ARM_NS_FAMILY_ID: u32 = 0xE48BFF5B;
pub const UF2_FAMILY_ID_MAX: u32 = 0xE48BFF5B;

/// USB Module
pub mod usb;
pub use usb::{Picoboot, Connection};

/// Command Module
pub mod cmd;
pub use cmd::{PicobootCmd, PicobootCmdId};

/// Target type for PicobootConnection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// RP2040/Pico target
    Rp2040,
    /// RP2350/Pico 2 target
    Rp2350,
    /// Custom target with specified VID/PID.  Used for an RP2040/RP2350 with
    /// non-standard VID/PIDs written via OTP.
    /// 
    /// Some methods do not support custom targets, as they require knowledge
    /// of the target type:
    /// - [`Connection::reboot()`]
    /// 
    /// You can use target specific methods, such as
    /// [`Connection::reboot_rp2040()`] and [`Connection::reboot_rp2350()`]
    /// instead, but they may silently fail if used with the wrong target.
    Custom {
        vid: u16,
        pid: u16,
    }
}

impl Target {
    /// Returns the USB Product ID for this target
    pub fn pid(&self) -> u16 {
        match self {
            Target::Rp2040 => PICOBOOT_PID_RP2040,
            Target::Rp2350 => PICOBOOT_PID_RP2350,
            Target::Custom { vid: _, pid } => *pid,
        }
    }

    /// Returns the USB Vendor ID for this target
    pub(crate) fn vid(&self) -> u16 {
        match self {
            Target::Rp2040 => PICOBOOT_VID,
            Target::Rp2350 => PICOBOOT_VID,
            Target::Custom { vid, pid: _ } => *vid,
        }
    }

    /// Returns the flash start address for this target
    pub fn flash_start(&self) -> u32 {
        FLASH_START
    }

    /// Returns the flash end address for this target
    pub fn flash_end(&self) -> Option<u32> {
        match self {
            Target::Rp2040 => Some(FLASH_END_RP2040),
            Target::Rp2350 => Some(FLASH_END_RP2350),
            Target::Custom { vid: _, pid: _ } => None,
        }
    }

    /// Returns the flash end address for this target
    pub fn flash_sector_size(&self) -> u32 {
        SECTOR_SIZE
    }

    /// Returns the flash end address for this target
    pub fn flash_page_size(&self) -> u32 {
        PAGE_SIZE
    }

    /// Returns the flash end address for this target
    pub fn default_stack_pointer(&self) -> Option<u32> {
        match self {
            Target::Rp2040 => Some(STACK_POINTER_RP2040),
            Target::Rp2350 => Some(STACK_POINTER_RP2350),
            Target::Custom { vid: _, pid: _ } => None,
        }
    }
}

/// Exclusive access modes for PICOBOOT EXCLUSIVE_ACCESS command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// No restriction on USB Mass Storage operation
    NotExclusive,
    /// Disable USB Mass Storage writes (the host should see them as write
    /// protect failures, but in any case any active UF2 download will be
    /// aborted)
    Exclusive,
    /// Lock the USB Mass Storage Interface out by marking the drive media as
    /// not present (ejecting the drive)
    ExclusiveAndEject,
}

impl From<Access> for u8 {
    fn from(access: Access) -> Self {
        match access {
            Access::NotExclusive => 0,
            Access::Exclusive => 1,
            Access::ExclusiveAndEject => 2,
        }
    }
}

/// Data transfer direction for PICOBOOT commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Data transfer from RP2040/RP2350 to host
    In,
    /// Data transfer from host to RP2040/RP2350
    Out,
}

/// Error type for this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// USB stack error
    #[error("usb stack error: {0}")]
    NusbError(nusb::Error),
    /// USB device not found.
    #[error("usb device not found")]
    UsbDeviceNotFound,
    /// PICBOOT interface not found
    #[error("picoboot interface not found")]
    PicobootInterfaceNotFound,
    /// USB device found, but failed to open.
    #[error("usb device found but can't open: {0}")]
    UsbDeviceFailedToOpen(nusb::Error),
    /// Failed to get active USB configuration.
    #[error("failed to get active usb configuration: {0}")]
    UsbGetActiveConfigurationFailure(nusb::ActiveConfigurationError),
    /// Failed to get the descriptor for the active USB configuration.
    #[error("no descriptor found for active usb configuration")]
    UsbNoActiveInterfaceDescriptor,
    /// Failed to claim endpoint
    #[error("failed to claim usb bulk endpoint: {0}")]
    UsbEndpointClaimFailure(nusb::Error),
    /// Failed to get USB bulk endpoints.
    #[error("failed to get usb bulk endpoints")]
    UsbEndpointsNotFound,
    /// USB bulk endpoints returned unexpected results.
    #[error("usb bulk endpoints are not expected")]
    UsbEndpointsUnexpected,
    /// Failed to detach USB kernel driver.
    #[error("failed to detach usb kernel driver: {0}")]
    UsbDetachKernelDriverFailure(nusb::Error),
    /// Failed to re-attach USB kernel driver.
    #[error("failed to re-attach usb kernel driver: {0}")]
    UsbReattachKernelDriverFailure(nusb::Error),
    /// Command failed as PICOBOOT is not connected
    #[error("picoboot not connected")]
    UsbNotConnected,
    /// Failed to claim USB interface.
    #[error("failed to claim usb interface: {0}")]
    UsbClaimInterfaceFailure(nusb::Error),
    /// Failed to configure alt USB setting.
    #[error("failed to set alt usb setting: {0}")]
    UsbSetAltSettingFailure(nusb::Error),
    /// Failed to read from USB bulk endpoint.
    #[error("failed to read bulk: {0}")]
    UsbReadBulkFailure(nusb::transfer::TransferError),
    /// Read data from USB does not match expected size.
    #[error("read did not match expected size")]
    UsbReadBulkMismatch,
    /// Failed to write to USB bulk endpoint.
    #[error("failed to write bulk: {0}")]
    UsbWriteBulkFailure(nusb::transfer::TransferError),
    /// Written data to USB does not match expected size.
    #[error("write did not match expected size")]
    UsbWriteBulkMismatch,

    /// Failed to clear USB in address halt.
    #[error("failed to clear in addr halt: {0}")]
    UsbClearInAddrHalt(nusb::Error),
    /// Failed to clear USB out address halt.
    #[error("failed to clear out addr halt: {0}")]
    UsbClearOutAddrHalt(nusb::Error),
    /// Failed to reset USB interface.
    #[error("failed to reset interface: {0}")]
    UsbResetInterfaceFailure(nusb::transfer::TransferError),

    /// Failed to get command status from device.
    #[error("failed to get command status: {0}")]
    UsbGetCommandStatusFailure(nusb::transfer::TransferError),

    /// Failed to serialize command for device.
    #[error("cmd failed to binary encode: {0}")]
    CmdSerializeFailure(deku::DekuError),
    /// Failed to deserialize command from device.
    #[error("cmd failed to binary decode: {0}")]
    CmdDeserializeFailure(deku::DekuError),

    /// Command is not allowed for target device.
    #[error("cmd not allowed for target device")]
    CmdNotAllowedForTarget,

    /// Erase command address invalid.
    #[error("erase address invalid")]
    EraseInvalidAddr,
    /// Erase command size invalid.
    #[error("erase size invalid")]
    EraseInvalidSize,

    /// Write command address invalid.
    #[error("write address invalid")]
    WriteInvalidAddr,

    /// Invalid duration
    #[error("invalid duration")]
    InvalidDuration,

    /// Missing buffer for data transfer command.
    #[error("missing buffer for data transfer command")]
    CmdDataTransferBufferMissing,
}
