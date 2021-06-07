//use super::types::*;
//use crate::events::*;
use super::events::*;
use std::convert::TryFrom;
use std::io::prelude::*;

/// A wrapper around input devices to simplify use for this crate.
pub struct InputDevice {
    device: evdev::Device,
}

impl From<evdev::Device> for InputDevice {
    fn from(evd: evdev::Device) -> Self {
        Self { device: evd }
    }
}

impl InputDevice {
    /// Creates a new Device by searching for the given name.
    ///
    /// Returns Some(Device) if the name was found, otherwise None.
    ///
    /// # Example:
    /// ```
    /// use chord2key::input::device::*;
    /// let none = InputDevice::from_name("No device");
    /// assert!(none.is_none());
    /// ```
    pub fn from_name(name: &str) -> Option<Self> {
        evdev::enumerate()
            .find(|d| d.name() == Some(name))
            .map(Self::from)
    }

    /// Creates a new Device by querying the user through the CLI.
    ///
    /// Returns the Device chosen by the user.
    pub fn from_cli() -> Self {
        let devices = evdev::enumerate().collect::<Vec<_>>();
        for (i, d) in devices.iter().enumerate() {
            println!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
        }
        print!("Select the device [0-{}]: ", devices.len() - 1);
        let _ = std::io::stdout().flush();
        let mut chosen = String::new();
        std::io::stdin().read_line(&mut chosen).unwrap();
        let n = chosen.trim().parse::<usize>().unwrap();
        let device = devices.into_iter().nth(n).unwrap();

        Self { device }
    }

    /// Polls the device for events, sending valid events to a closure.
    ///
    /// This will block until an event -- valid or invalid -- occurs.
    pub fn poll<F>(&mut self, mut callback: F) -> Result<(), std::io::Error>
    where
        F: FnMut(&InputEvent),
    {
        let events = self.device.fetch_events()?;

        //let start = std::time::Instant::now();
        for event in events {
            if let Ok(input) = InputEvent::try_from(event) {
                callback(&input);
            }
        }
        //let duration = start.elapsed();
        //println!("Time elapsed to handle event is: {:?}", duration);
        Ok(())
    }
}
