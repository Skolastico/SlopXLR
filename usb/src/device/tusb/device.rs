use crate::commands::Command;
use crate::device::base::{
    AttachGoXLR, ExecutableGoXLR, FullGoXLRDevice, GoXLRCommands, GoXLRDevice, UsbData,
};
use crate::device::tusb::tusbaudio::{
    DeviceHandle, EventChannelReceiver, EventChannelSender, TUSB_INTERFACE, get_devices,
    get_version,
};
use anyhow::{Result, bail};
use byteorder::{ByteOrder, LittleEndian};
use goxlr_types::{DriverInterface, VersionNumber};
use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;

pub struct TUSBAudioGoXLR {
    // Basic Device Information..
    handle: DeviceHandle,
    identifier: Option<String>,
    command_count: u16,

    // Event Handlers..
    event_receivers: EventChannelReceiver,
    disconnect_sender: Sender<String>,
    event_sender: Sender<String>,

    // Identifier for Daemon..
    daemon_identifier: Arc<Mutex<Option<String>>>,

    // Thread states
    stopped: Arc<AtomicBool>,
    event_thread: Option<thread::JoinHandle<()>>,
}

impl TUSBAudioGoXLR {
    fn write_control(&self, request: u8, value: u16, index: u16, data: &[u8]) -> Result<()> {
        self.handle.send_request(request, value, index, data)
    }

    fn read_control(
        &mut self,
        request: u8,
        value: u16,
        index: u16,
        length: usize,
    ) -> Result<Vec<u8>> {
        self.handle.read_response(request, value, index, length)
    }

    fn trigger_disconnect(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        // A failed profile may issue several commands before the daemon removes the device.
        // Close and announce this session only once.
        if !self.handle.is_open() {
            return;
        }
        let _ = self.handle.close_handle();

        if let Some(daemon_identifier) = &*self.daemon_identifier.lock().unwrap() {
            let _ = self.disconnect_sender.try_send(daemon_identifier.clone());
        }
    }

    fn await_data(receiver: &Mutex<mpsc::Receiver<bool>>, stopped: &AtomicBool) -> bool {
        if stopped.load(Ordering::Relaxed) {
            return false;
        }
        // A native channel wakes as soon as the event thread responds, without spinning
        // or imposing a polling delay on every configuration command.
        match receiver
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(1))
        {
            Ok(result) => result && !stopped.load(Ordering::Relaxed),
            Err(RecvTimeoutError::Timeout) => {
                warn!("Timeout Awaiting Response..");
                false
            }
            Err(RecvTimeoutError::Disconnected) => {
                warn!("USB response channel has closed");
                false
            }
        }
    }

    pub fn await_ready(mut receiver: tokio::sync::oneshot::Receiver<bool>) -> bool {
        let timeout = Instant::now() + Duration::from_secs(1);
        loop {
            thread::sleep(Duration::from_millis(5));
            if Instant::now() > timeout {
                // We've hit a timeout, don't infinite loop, instead throw as error.
                return false;
            }

            let result = receiver.try_recv();
            match result {
                Ok(result) => break result,
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => break false,
                Err(_) => continue,
            }
        }
    }
}

impl Drop for TUSBAudioGoXLR {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        let _ = self.handle.close_handle();
        if let Some(event_thread) = self.event_thread.take() {
            let _ = event_thread.join();
        }
        TUSB_INTERFACE.request_device_refresh();
    }
}

// Only probe vendor-interface readiness again; never replay a reset or profile command.
fn activate_vendor_interface(
    mut read_packet: impl FnMut() -> Result<Vec<u8>>,
    retry_window: Duration,
) -> Result<()> {
    let deadline = Instant::now() + retry_window;
    loop {
        match read_packet() {
            Ok(_) => return Ok(()),
            Err(error) => {
                if error.to_string() != "TSTATUS_USBD_STATUS_STALL_PID"
                    || Instant::now() >= deadline
                {
                    return Err(error);
                }
                sleep(
                    Duration::from_millis(50)
                        .min(deadline.saturating_duration_since(Instant::now())),
                );
            }
        }
    }
}

impl AttachGoXLR for TUSBAudioGoXLR {
    fn from_device(
        device: GoXLRDevice,
        disconnect_sender: Sender<String>,
        event_sender: Sender<String>,
        skip_pause: bool,
    ) -> Result<Box<dyn FullGoXLRDevice>>
    where
        Self: Sized,
    {
        let mut device_identifier = None;
        if let Some(identifier) = &device.identifier {
            device_identifier = Some(identifier.clone());
        }

        let handle = DeviceHandle::from_device(device).inspect_err(|_| {
            TUSB_INTERFACE.request_device_refresh();
        })?;

        // Spawn the Event handler thread..
        let (data_sender, data_receiver) = mpsc::channel();

        // In this case, we spawn a thread to manage windows events..
        let event_receivers = EventChannelReceiver {
            data_read: Mutex::new(data_receiver),
        };

        let mut goxlr = Box::new(Self {
            handle,
            identifier: device_identifier,

            command_count: 0,

            event_receivers,
            disconnect_sender,
            event_sender,

            daemon_identifier: Arc::new(Mutex::new(None)),

            stopped: Arc::new(AtomicBool::new(false)),
            event_thread: None,
        });

        let (ready_sender, ready_recv) = tokio::sync::oneshot::channel();

        // Spawn an event loop for this handle..
        let thread_event_sender = goxlr.event_sender.clone();
        let thread_daemon_identifier = goxlr.daemon_identifier.clone();
        let thread_stopped = goxlr.stopped.clone();
        if let Some(ref thread_device_identifier) = goxlr.identifier {
            // Clone it so we can move it into the thread..
            let thread_device_identifier = thread_device_identifier.clone();

            goxlr.event_thread = Some(thread::spawn(move || {
                let sender = EventChannelSender {
                    ready_notifier: ready_sender,
                    data_read: data_sender,
                    input_changed: thread_event_sender,
                };

                // Spawn the Event Loop..
                let result = TUSB_INTERFACE.event_loop(
                    thread_device_identifier.clone(),
                    thread_daemon_identifier,
                    sender,
                    thread_stopped.clone(),
                );
                thread_stopped.store(true, Ordering::Relaxed);
                if let Err(error) = result {
                    warn!("USB event monitor failed: {}", error);
                }
            }));
        } else {
            bail!("Unable to Create Event Loop, Device Identifier not set!");
        }

        // Wait for the event loop to be ready and registered..
        if !TUSBAudioGoXLR::await_ready(ready_recv) {
            goxlr.stopped.store(true, Ordering::Relaxed);
            bail!("Unable to establish Event Loop..");
        }

        // Activate the Vendor interface, also initialises audio on Windows!
        let readiness_window = if skip_pause {
            Duration::ZERO
        } else {
            Duration::from_secs(2)
        };
        if let Err(error) = activate_vendor_interface(
            || {
                if goxlr.stopped.load(Ordering::Relaxed) {
                    bail!("USB session ended during audio-interface activation");
                }
                goxlr.handle.read_response(0, 0, 0, 24)
            },
            readiness_window,
        ) {
            goxlr.stopped.store(true, Ordering::Relaxed);
            bail!("Error Reading Initial Packet: {}", error);
        }

        // Perform soft reset.
        if let Err(error) = goxlr.handle.send_request(1, 0, 0, &[]) {
            goxlr.stopped.store(true, Ordering::Relaxed);
            bail!("Error Sending initial Reset Packet: {}", error);
        }

        // Wait for the response event, then read..
        if !Self::await_data(&goxlr.event_receivers.data_read, &goxlr.stopped) {
            bail!("Error received from Event Handler..");
        }

        if let Err(error) = goxlr.handle.read_response(3, 0, 0, 1040) {
            goxlr.stopped.store(true, Ordering::Relaxed);
            bail!("Error Reading Response to Initial Reset: {}", error);
        }
        Ok(goxlr)
    }

    fn set_unique_identifier(&mut self, identifier: String) {
        // Spawn Notification Thread..
        let mut local_identifier = self.daemon_identifier.lock().unwrap();
        *local_identifier = Some(identifier);
    }

    fn is_connected(&mut self) -> bool {
        if self.stopped.load(Ordering::Relaxed) || self.handle.get_device_id_string().is_err() {
            self.trigger_disconnect();
            return false;
        }
        true
    }

    fn set_is_polling(&mut self, _polling: bool) {
        // The TUSB implementation is event driven, so there's no polling to stop.
    }
}

impl ExecutableGoXLR for TUSBAudioGoXLR {
    fn perform_request(&mut self, command: Command, body: &[u8], retry: bool) -> Result<Vec<u8>> {
        if self.stopped.load(Ordering::Relaxed) {
            self.trigger_disconnect();
            bail!("USB session has ended; waiting for reinitialization");
        }
        if command == Command::ResetCommandIndex {
            self.command_count = 0;
        } else {
            if self.command_count == u16::MAX {
                let _ = self.request_data(Command::ResetCommandIndex, &[])?;
            }
            self.command_count += 1;
        }

        let command_index = self.command_count;
        let mut full_request = vec![0; 16];
        LittleEndian::write_u32(&mut full_request[0..4], command.command_id());
        LittleEndian::write_u16(&mut full_request[4..6], body.len() as u16);
        LittleEndian::write_u16(&mut full_request[6..8], command_index);
        full_request.extend(body);

        if let Err(error) = self.write_control(2, 0, 0, &full_request) {
            // A button-state request only reads current device state. If its USB write
            // times out but the session is still valid, reset the protocol counter and
            // retry this read once. All other commands retain full-session recovery.
            if command == Command::GetButtonStates
                && !retry
                && error.to_string() == "TSTATUS_TIMEOUT"
                && self.is_connected()
            {
                warn!("GetButtonStates write timed out; attempting one protocol resynchronization");
                self.perform_request(Command::ResetCommandIndex, &[], true)?;
                info!("USB protocol counter resynchronized; retrying GetButtonStates once");
                return self.perform_request(command, body, true);
            }

            // Reopening a handle alone bypasses vendor-interface activation and preserves
            // the command index from the previous USB session. Let the daemon initialize anew.
            self.trigger_disconnect();
            bail!(
                "USB command {:?} failed; reinitializing: {}",
                command,
                error
            );
        }

        // We will sit here, and wait for a response.. this may take a few cycles..
        if !Self::await_data(&self.event_receivers.data_read, &self.stopped) {
            self.trigger_disconnect();
            bail!("No USB response for {:?}; reinitializing", command);
        }

        let mut response_header = match self.read_control(3, 0, 0, 1040) {
            Ok(response) => response,
            Err(error) => {
                self.trigger_disconnect();
                bail!(
                    "USB response for {:?} failed; reinitializing: {}",
                    command,
                    error
                );
            }
        };
        if response_header.len() < 16 {
            error!(
                "Invalid Response received from the GoXLR, Expected: 16, Received: {}",
                response_header.len()
            );
            bail!("Invalid Response");
        }

        let response = response_header.split_off(16);
        let response_length = LittleEndian::read_u16(&response_header[4..6]);
        let response_command_index = LittleEndian::read_u16(&response_header[6..8]);

        if response_command_index != command_index {
            debug!("Mismatched Command Indexes..");
            debug!(
                "Expected {}, received: {}",
                command_index, response_command_index
            );
            debug!("Full Request: {:?}", full_request);
            debug!("Response Header: {:?}", response_header);
            debug!("Response Body: {:?}", response);

            return if !retry {
                debug!("Attempting Resync and Retry");
                self.perform_request(Command::ResetCommandIndex, &[], true)?;

                debug!("Resync complete, retrying Command..");
                self.perform_request(command, body, true)
            } else {
                debug!("Resync Failed, Throwing Error..");
                self.trigger_disconnect();
                bail!("Invalid Response received from GoXLR, disconnecting!");
            };
        }

        debug_assert!(response.len() == response_length as usize);
        Ok(response)
    }

    fn get_descriptor(&self) -> Result<UsbData> {
        let properties = self.handle.get_properties()?;

        Ok(UsbData {
            vendor_id: properties.vendor_id() as u16,
            product_id: properties.product_id() as u16,
            device_version: (2, 0, 0),
            device_manufacturer: properties.manufacturer()?,
            product_name: properties.model()?,
        })
    }
}

impl GoXLRCommands for TUSBAudioGoXLR {}
impl FullGoXLRDevice for TUSBAudioGoXLR {}

pub fn find_devices() -> Vec<GoXLRDevice> {
    get_devices()
}

pub fn get_interface_version() -> (DriverInterface, Option<VersionNumber>) {
    (DriverInterface::TUSB, get_version())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_interface_activates_without_calibration_delay() {
        let mut reads = 0;
        let start = Instant::now();
        activate_vendor_interface(
            || {
                reads += 1;
                Ok(vec![0; 24])
            },
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(reads, 1);
        assert!(start.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn calibrating_interface_can_become_ready() {
        let mut reads = 0;
        activate_vendor_interface(
            || {
                reads += 1;
                if reads < 3 {
                    bail!("TSTATUS_USBD_STATUS_STALL_PID");
                }
                Ok(vec![0; 24])
            },
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(reads, 3);
    }

    #[test]
    fn invalid_handle_and_unknown_errors_are_not_retried() {
        for error in ["TSTATUS_INVALID_HANDLE", "Unexpected driver error"] {
            let mut reads = 0;
            let result = activate_vendor_interface(
                || {
                    reads += 1;
                    bail!("{}", error);
                },
                Duration::from_secs(2),
            );
            assert!(result.is_err());
            assert_eq!(reads, 1);
        }
    }

    #[test]
    fn skip_pause_disables_readiness_retries() {
        let mut reads = 0;
        let result = activate_vendor_interface(
            || {
                reads += 1;
                bail!("TSTATUS_USBD_STATUS_STALL_PID");
            },
            Duration::ZERO,
        );
        assert!(result.is_err());
        assert_eq!(reads, 1);
    }

    #[test]
    fn readiness_retries_stop_at_deadline() {
        let mut reads = 0;
        let result = activate_vendor_interface(
            || {
                reads += 1;
                bail!("TSTATUS_USBD_STATUS_STALL_PID");
            },
            Duration::from_millis(10),
        );
        assert!(result.is_err());
        assert!((1..=2).contains(&reads));
    }

    #[test]
    fn response_from_stopped_session_is_rejected() {
        let (sender, receiver) = mpsc::channel();
        let receiver = Mutex::new(receiver);
        sender.send(true).unwrap();
        let stopped = AtomicBool::new(true);
        assert!(!TUSBAudioGoXLR::await_data(&receiver, &stopped));
        assert_eq!(receiver.lock().unwrap().try_recv(), Ok(true));
    }

    #[test]
    fn response_received_after_wait_is_accepted() {
        let (sender, receiver) = mpsc::channel();
        let receiver = Mutex::new(receiver);
        let sender_thread = thread::spawn(move || {
            sleep(Duration::from_millis(20));
            sender.send(true).unwrap();
        });
        let stopped = AtomicBool::new(false);
        assert!(TUSBAudioGoXLR::await_data(&receiver, &stopped));
        sender_thread.join().unwrap();
    }

    #[test]
    fn closed_response_channel_returns_without_waiting_for_timeout() {
        let (sender, receiver) = mpsc::channel();
        let receiver = Mutex::new(receiver);
        drop(sender);
        let start = Instant::now();
        assert!(!TUSBAudioGoXLR::await_data(
            &receiver,
            &AtomicBool::new(false)
        ));
        assert!(start.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn disconnect_response_is_rejected() {
        let (sender, receiver) = mpsc::channel();
        let receiver = Mutex::new(receiver);
        sender.send(false).unwrap();
        assert!(!TUSBAudioGoXLR::await_data(
            &receiver,
            &AtomicBool::new(false)
        ));
    }
}
