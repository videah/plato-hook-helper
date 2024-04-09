# plato-hook-helper

a set of utility functions to assist with writing fetch hooks for the [Plato](https://github.com/baskerville/plato)
e-reader document system.

Implements events from the documented [Plato Hook API](https://github.com/baskerville/plato/blob/master/doc/HOOKS.md).

## Usage

```rust
let mut plato = PlatoHelper::default ();

// Display a notification on the device.
plato.display_notification("Turning on Wi-Fi...").unwrap();

// Enable the device's Wi-Fi.
plato.set_wifi(WifiStatus::Enabled).unwrap();

// Block and wait for the network to come up.
plato.wait_for_network_blocking().unwrap();
```