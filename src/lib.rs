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
//!     let mut picoboot = Picoboot::from_first(None).await?;
//!     let conn = picoboot.connect().await?;
//!
//!     // Claim exclusive access, ejecting the BOOTSEL mass storage device and
//!     // exit XIP mode (only necessary on RP2040)
//!     conn.set_exclusive_access(Access::ExclusiveAndEject).await?;
//!     conn.exit_xip().await?;
//! 
//!     // Erase first 4096 bytes of flash (1 sector)
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

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Rp2040 => write!(f, "RP2040"),
            Target::Rp2350 => write!(f, "RP2350"),
            Target::Custom { vid, pid } => {
                write!(f, "{:04x}:{:04x}", vid, pid)
            }
        }
    }
}

impl From<&nusb::DeviceInfo> for Target {
    fn from(dev_info: &nusb::DeviceInfo) -> Self {
        match (dev_info.vendor_id(), dev_info.product_id()) {
            (PICOBOOT_VID, PICOBOOT_PID_RP2040) => Target::Rp2040,
            (PICOBOOT_VID, PICOBOOT_PID_RP2350) => Target::Rp2350,
            (vid, pid) => Target::Custom { vid, pid },
        }
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
    pub fn vid(&self) -> u16 {
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

/// `picoboot` error type.
/// 
/// Errors are broken down into:
/// - USB errors - originating from the underlying `nusb` crate
/// - PICOBOOT errors - originating from PICOBOOT device or protocol handling
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Hit error enumerating USB devices
    #[error("Error enumerating USB devices: {0}")]
    UsbEnumerationError(nusb::Error),

    /// Failed to open USB device.
    #[error("Failed to open target {0}: {1}")]
    UsbOpenError(Target, nusb::Error),

    /// Failed to claim USB interface.
    #[error("Failed to claim USB interface on {0}: {1}")]
    UsbClaimInterfaceFailure(Target, nusb::Error),

    /// Failed to get the descriptor for the active USB configuration.
    #[error("Failed to get descriptor for active usb configuration on {0}")]
    UsbNoActiveInterfaceDescriptor(Target),

    /// Failed to get USB bulk endpoints.
    #[error("Failed to find USB bulk endpoints on {0}")]
    UsbEndpointsNotFound(Target),

    /// Failed to claim endpoint
    #[error("Failed to claim USB bulk IN endpoint on {0}: {1}")]
    UsbInEndpointClaimFailure(Target, nusb::Error),

    /// Failed to claim USB bulk OUT endpoint.
    #[error("Failed to claim USB bulk OUT endpoint on {0}: {1}")]
    UsbOutEndpointClaimFailure(Target, nusb::Error),

    /// Failed to clear USB IN address halt.
    #[error("Failed to clear IN endoint halt on {0}: {1}")]
    UsbClearInEndpointHaltFailure(Target, nusb::Error),

    /// Failed to clear USB out address halt.
    #[error("Failed to clear OUT endpoint halt on {0}: {1}")]
    UsbClearOutEndpointHaltFailure(Target, nusb::Error),

    /// Failed to detach USB kernel driver (linux only).
    #[cfg(target_os = "linux")]
    #[error("Failed to detach USB kernel driver for {0}: {1}")]
    UsbDetachKernelDriverFailure(Target, nusb::Error),

    /// Failed to re-attach USB kernel driver (linux only).
    #[cfg(target_os = "linux")]
    #[error("Failed to re-attach USB kernel driver for {0}: {1}")]
    UsbReattachKernelDriverFailure(Target, nusb::Error),

    /// Failed to read from USB bulk endpoint.
    #[error("Bulk read failed on {0}: {1}")]
    UsbReadBulkFailure(Target, nusb::transfer::TransferError),

    /// Read data from USB does not match expected size.
    #[error("Bulk read did not match expected size on {0}: {1} >= {2}")]
    UsbReadBulkMismatch(Target, usize, usize),

    /// Failed to write to USB bulk endpoint.
    #[error("Bulk write failed on {0}: {1}")]
    UsbWriteBulkFailure(Target, nusb::transfer::TransferError),

    /// Written data to USB does not match expected size.
    #[error("Bulk write did not match expected size on {0}: {1} != {2}")]
    UsbWriteBulkMismatch(Target, usize, usize),

    /// No PICOBOOT devices found.
    #[error("No PICOBOOT devices found")]
    PicobootNoDevicesFound,

    /// PICOBOOT get command status from device failed.
    #[error("PICOBOOT get command status from {0} failed: {1}")]
    PicobootGetCommandStatusFailure(Target, nusb::transfer::TransferError),

    /// Failed to reset PICOBOOT interface.
    #[error("Failed to reset PICOBOOT interface: {0}")]
    PicobootResetInterfaceFailure(Target, nusb::transfer::TransferError),

    /// PICOBOOT interface not found on a device purporting to be an RP2040 or
    /// RP2350 in BOOTSEL mode.
    #[error("PICOBOOT interface not found on {0}")]
    PicobootInterfaceNotFound(Target),

    /// Command is not allowed for this PICOBOOT target.
    #[error("PICOBOOT command not allowed for {0}: {1}")]
    PicobootCmdNotAllowedForTarget(Target, PicobootCmdId),

    /// Erase command address invalid.
    #[error("Erase address invalid on {0}: {1:#X}")]
    PicobootEraseInvalidAddr(Target, u32),

    /// Erase command size invalid.
    #[error("Erase size invalid on {0}: {1:#X}")]
    PicobootEraseInvalidSize(Target, u32),

    /// Write command address invalid.
    #[error("Write address invalid on {0}: {1:#X}")]
    PicobootWriteInvalidAddr(Target, u32),

    /// Failed to serialize PICOBOOT command for device.  Most likely an
    /// internal error.
    #[error("PICOBOOT command failed to binary encode for {0}: {1}")]
    PicobootCmdSerializeFailure(Target, deku::DekuError),

    /// Failed to deserialize PICOBOOT command from device.  Suggests the
    /// device sent malformed data.
    #[error("PICOBOOT command failed to binary decode    {0}: {1}")]
    PicobootCmdDeserializeFailure(Target, deku::DekuError),

    /// Invalid duration
    #[error("Invalid duration on {0}: {1:?}")]
    PicobootInvalidDuration(Target, std::time::Duration),

    /// Missing data for a command that transmits data.
    #[error("Missing buffer for data transfer command on {0}: {1}")]
    PicobootCmdDataMissing(Target, PicobootCmdId),
}
