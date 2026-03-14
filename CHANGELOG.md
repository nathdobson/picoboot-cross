# Changelog

## [0.2.0]

- Added picobootx support which extends picoboot support with additional command using a different (non RP2040/RP2350) magic.  Works with [picobootx](https://github.com/piersfinlayson/picobootx).
- More sophisticated picoboot interface matching, supporting new One ROM Fire USB stack/picobootx.
- Add an airfrog-rpc::io::Reader implementation for live reading flash and RAM via Picoboot.
- Add Picotboot::reboot convenience function to reboot the device.
- Added read/write functions to support RAM.  The old flash_read and flash_write supported RAM, but added unnecessary alignment and size checks for RAM.
- Allow reboot2 on custom targets.

## [0.1.1]

Added:
- `Picoboot::info` to output VID/PID of device
- Additional `Picoboot` error variants for better error handling

Fixed:
- USB control request types from Class to Vendor

## [0.1.0] - 2025-11-03

Initial release