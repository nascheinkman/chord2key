use chord2key::input::{device::InputDevice, events::InputEvent};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

pub const SLEEP_TIME: u64 = 300;
pub const TIMEOUT_TIME: u64 = 1000;

/// Helper struct for reading input from a background thread. Accumulates recieved input for later
/// use.
pub struct BackgroundInputDevice {
    event_rx: Receiver<InputEvent>,
}

impl BackgroundInputDevice {
    pub fn from_name(name: &str) -> Result<Self, std::io::Error> {
        let mut input_device = InputDevice::from_name(name).ok_or(std::io::ErrorKind::NotFound)?;
        let (tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

        let _handle = thread::spawn(move || loop {
            let mut failed = false;
            let res = input_device.poll(|ev| {
                let event_res = tx.send(ev.clone());
                if event_res.is_err() {
                    failed = true;
                }
            });
            if failed || res.is_err() {
                break;
            }
        });
        Ok(Self { event_rx: rx })
    }

    pub fn collect_timeout(&self, timeout: Duration) -> Result<Vec<InputEvent>, RecvTimeoutError> {
        let mut returned = vec![];
        let recv_event = self.event_rx.recv_timeout(timeout)?;
        let mut recv_event = Ok(recv_event);
        while recv_event.is_ok() {
            returned.push(recv_event.unwrap());
            recv_event = self.event_rx.try_recv();
        }
        Ok(returned)
    }
}
