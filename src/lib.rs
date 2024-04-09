//! # plato-hook-helper
//! `plato-hook-helper` is a set of utility functions to assist with writing fetch hooks for the
//! [Plato](https://github.com/baskerville/plato) e-reader document system.

use std::io::{BufRead, BufReader, Read, Stdin, Stdout, Write};

use serde::{Deserialize, Serialize};

/// The status of the e-reader's Wi-Fi.
pub enum WifiStatus {
    /// The Wi-Fi is turned on, allowing network connections to be made.
    Enabled,
    /// The Wi-Fi is turned off, no network connections can be made.
    Disabled,
}

impl From<WifiStatus> for bool {
    fn from(f: WifiStatus) -> bool {
        match f {
            WifiStatus::Enabled => true,
            WifiStatus::Disabled => false,
        }
    }
}

/// The structure of a notification event. Used to display a message on the device.
#[derive(Serialize, Deserialize)]
struct NotificationEvent {
    r#type: String,
    message: String,
}

/// The structure of a Wi-Fi event. Used to enable or disable the device's Wi-Fi.
#[derive(Serialize, Deserialize)]
struct WifiEvent {
    r#type: String,
    enable: bool,
}

/// The structure of a network event. Used to signal when the device's network status changes.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NetworkEvent {
    r#type: String,
    status: String,
}

/// A helper struct for interacting with the Plato e-reader software. Holds a writer to output JSON
/// to, by default `stdout`.
pub struct PlatoHelper<W: Write, R: Read> {
    writer: W,
    reader: R,
}

impl Default for PlatoHelper<Stdout, Stdin> {
    fn default() -> Self {
        PlatoHelper {
            writer: std::io::stdout(),
            reader: std::io::stdin(),
        }
    }
}

impl<W: Write, R: Read> PlatoHelper<W, R> {
    pub fn new(writer: W, reader: R) -> Self {
        PlatoHelper { writer, reader }
    }

    /// Take's a serializable struct and writes it to the internal writer as a JSON string.
    fn write_json<T: Serialize>(&mut self, value: &T) -> std::io::Result<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.writer.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Reads JSON from the internal reader, blocking until a valid JSON string is received matching
    /// the given type `J`.
    fn read_json_blocking<J>(&mut self) -> Result<J, std::io::Error>
    where
        J: for<'de> Deserialize<'de>,
    {
        let mut reader = BufReader::new(&mut self.reader);
        let mut input = String::new();
        loop {
            input.clear();
            reader.read_line(&mut input)?;
            if let Ok(json) = serde_json::from_str(&input) {
                return Ok(json);
            }
        }
    }

    /// Displays a notification on the device with the given `message`.
    pub fn display_notification(&mut self, message: &str) -> std::io::Result<()> {
        let event = NotificationEvent {
            r#type: "notify".to_string(),
            message: message.to_string(),
        };
        self.write_json(&event)
    }

    /// Sets the device's Wi-Fi state to `status`.
    pub fn set_wifi(&mut self, status: WifiStatus) -> std::io::Result<()> {
        let event = WifiEvent {
            r#type: "setWifi".to_string(),
            enable: status.into(),
        };
        self.write_json(&event)
    }

    /// Waits until a network event is received from the internal reader.
    /// This function will block indefinitely until a valid event is received or an IO error occurs.
    pub fn wait_for_network_blocking(&mut self) -> Result<NetworkEvent, std::io::Error> {
        self.read_json_blocking()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Cursor};

    use super::*;

    #[test]
    fn notification_formatting() {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        {
            let mut plato = PlatoHelper::new(writer, std::io::stdin());
            plato.display_notification("Hello, World!").unwrap();
        }

        let notification = NotificationEvent {
            r#type: "notify".to_string(),
            message: "Hello, World!".to_string(),
        };

        let event = serde_json::to_string(&notification).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert_eq!(event, output);
    }

    #[test]
    fn wait_for_network_blocking_deserializes_correctly() {
        let json = r#"{"type": "network", "status": "up"}"#;
        let reader = Cursor::new(json);
        let mut plato = PlatoHelper::new(Vec::new(), reader);
        let result: Result<NetworkEvent, std::io::Error> = plato.wait_for_network_blocking();

        assert_eq!(
            result.unwrap(),
            NetworkEvent {
                r#type: "network".to_string(),
                status: "up".to_string()
            }
        );
    }
}
