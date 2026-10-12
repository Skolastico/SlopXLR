use super::*;
use goxlr_usb::commands::Command;
use goxlr_usb::device::base::{AttachGoXLR, ExecutableGoXLR, GoXLRCommands, GoXLRDevice, UsbData};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct SimulatedUsb {
    connected: bool,
    fail: Option<Command>,
    failures_left: usize,
    disconnect_on_failure: bool,
    music_volume: u8,
    channel_volumes: EnumMap<ChannelName, u8>,
    hardware_faders: EnumMap<FaderName, ChannelName>,
    physical_volumes: EnumMap<FaderName, u8>,
    latched: EnumMap<FaderName, bool>,
    volume_writes: usize,
    display_modes: EnumMap<FaderName, [u8; 2]>,
    requests: Vec<(Command, Vec<u8>)>,
}

impl SimulatedUsb {
    fn move_fader(&mut self, fader: FaderName, value: u8) -> [u8; 4] {
        let channel = self.hardware_faders[fader];
        let target = self.channel_volumes[channel];
        let previous = self.physical_volumes[fader];
        if (previous <= target && value >= target) || (previous >= target && value <= target) {
            self.latched[fader] = true;
        }
        self.physical_volumes[fader] = value;
        if self.latched[fader] {
            self.channel_volumes[channel] = value;
        }
        self.read_volumes()
    }
    fn read_volumes(&self) -> [u8; 4] {
        std::array::from_fn(|index| self.channel_volumes[self.hardware_faders[FaderName::iter().nth(index).unwrap()]])
    }
}

struct SimulatedDevice(Arc<Mutex<SimulatedUsb>>);
impl FullGoXLRDevice for SimulatedDevice {}
impl GoXLRCommands for SimulatedDevice {}
impl AttachGoXLR for SimulatedDevice {
    fn from_device(
        _: GoXLRDevice,
        _: Sender<String>,
        _: Sender<String>,
        _: bool,
    ) -> Result<Box<dyn FullGoXLRDevice>> {
        bail!("Tests create the simulated device directly")
    }
    fn set_unique_identifier(&mut self, _: String) {}
    fn set_is_polling(&mut self, _: bool) {}
    fn is_connected(&mut self) -> bool {
        self.0.lock().unwrap().connected
    }
}
impl ExecutableGoXLR for SimulatedDevice {
    fn get_descriptor(&self) -> Result<UsbData> {
        bail!("No real USB descriptor in tests")
    }
    fn perform_request(&mut self, command: Command, body: &[u8], _: bool) -> Result<Vec<u8>> {
        let mut state = self.0.lock().unwrap();
        if !state.connected {
            bail!("USB disconnected");
        }
        if state.fail == Some(command) && state.failures_left > 0 {
            state.failures_left -= 1;
            if state.disconnect_on_failure {
                state.connected = false;
            }
            bail!("Simulated USB failure after partial application");
        }
        state.requests.push((command, body.to_vec()));
        if command == Command::SetChannelVolume(ChannelName::Music) {
            state.music_volume = body[0];
        }
        if let Command::SetChannelVolume(channel) = command {
            state.channel_volumes[channel] = body[0];
            state.volume_writes += 1;
            for fader in FaderName::iter() {
                if state.hardware_faders[fader] == channel {
                    state.latched[fader] = false;
                }
            }
        }
        if let Command::SetFader(fader) = command {
            state.hardware_faders[fader] = ChannelName::iter().find(|channel| *channel as u8 == body[0]).unwrap();
            state.latched[fader] = false;
        }
        if let Command::SetFaderDisplayMode(fader) = command {
            state.display_modes[fader] = [body[0], body[1]];
        }
        Ok(vec![])
    }
}

#[tokio::test]
async fn profile_changes_commit_or_recover_after_partial_usb_application() -> Result<()> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "goxlr-profile-test-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory)?;
    let path = directory.join("settings.json");
    std::fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({
            "profile_directory": directory, "mic_profile_directory": directory,
            "backup_directory": directory, "samples_directory": directory,
            "devices": {"TEST": {"profile": "Previous", "mic_profile": "PreviousMic"}}
        }))?,
    )?;
    let settings = SettingsHandle::load(path.clone()).await?;
    let mut previous = ProfileAdapter::default();
    previous.set_channel_volume(ChannelName::Music, 42)?;
    previous.save_as("Previous".into(), &directory, false)?;
    let mut next = ProfileAdapter::default();
    next.set_channel_volume(ChannelName::Music, 7)?;
    next.save_as("Next".into(), &directory, false)?;
    let mut mic = MicProfileAdapter::default();
    mic.save_as("PreviousMic".into(), &directory, false)?;
    mic.save_as("NextMic".into(), &directory, false)?;
    assert!(previous.save_as("Next".into(), &directory, false).is_err());
    assert_eq!(previous.name(), "Previous");
    let mut previous_mic = MicProfileAdapter::from_named("PreviousMic".into(), &directory)?;
    assert!(
        previous_mic
            .save_as("NextMic".into(), &directory, false)
            .is_err()
    );
    assert_eq!(previous_mic.name(), "PreviousMic");

    // Both kinds of profile use their real command paths, with faults injected
    // after some hardware settings have already been sent.
    for microphone in [false, true] {
        for (failures, disconnect) in [(0, false), (1, false), (1, true), (2, false)] {
            settings.set_device_profile_name("TEST", "Previous").await;
            settings
                .set_device_mic_profile_name("TEST", "PreviousMic")
                .await;
            settings.save().await;
            let usb = Arc::new(Mutex::new(SimulatedUsb {
                connected: true,
                fail: None,
                failures_left: 0,
                disconnect_on_failure: false,
                music_volume: 0,
                channel_volumes: Default::default(),
                ..Default::default()
            }));
            let hardware = HardwareStatus {
                versions: goxlr_types::FirmwareVersions {
                    firmware: VersionNumber::default(),
                    fpga_count: 0,
                    dice: VersionNumber::default(),
                },
                serial_number: "TEST".into(),
                manufactured_date: String::new(),
                device_type: DeviceType::Mini,
                colour_way: goxlr_ipc::ColourWay::Black,
                usb_device: goxlr_ipc::UsbProductInformation {
                    manufacturer_name: String::new(),
                    product_name: String::new(),
                    version: (0, 0, 0),
                    bus_number: 0,
                    address: 0,
                    identifier: None,
                },
            };
            let (events, _receiver) = tokio::sync::mpsc::channel(16);
            let mut device = Device::new(
                Box::new(SimulatedDevice(usb.clone())),
                hardware,
                &settings,
                events,
            )
            .await?;
            assert!(
                device
                    .perform_command(GoXLRCommand::LoadProfile("Missing".into(), false))
                    .await
                    .is_err()
            );
            assert_eq!(device.profile.name(), "Previous");
            assert_eq!(usb.lock().unwrap().music_volume, 42);
            {
                let mut state = usb.lock().unwrap();
                state.fail = Some(if microphone {
                    Command::SetEffectParameters
                } else {
                    Command::SetColourMap()
                });
                state.failures_left = failures;
                state.disconnect_on_failure = disconnect;
            }
            let command = if microphone {
                GoXLRCommand::LoadMicProfile("NextMic".into(), true)
            } else {
                GoXLRCommand::LoadProfile("Next".into(), true)
            };
            let result = device.perform_command(command).await;
            let success = failures == 0;
            assert_eq!(
                result.is_ok(),
                success,
                "microphone={microphone}, failures={failures}, disconnect={disconnect}"
            );
            assert_eq!(
                device.profile.name(),
                if success && !microphone {
                    "Next"
                } else {
                    "Previous"
                }
            );
            assert_eq!(
                device.mic_profile.name(),
                if success && microphone {
                    "NextMic"
                } else {
                    "PreviousMic"
                }
            );
            assert_eq!(device.needs_profile_recovery(), disconnect || failures == 2);
            let saved = SettingsHandle::load(path.clone()).await?;
            assert_eq!(
                saved.get_device_profile_name("TEST").await.as_deref(),
                Some(if success && !microphone {
                    "Next"
                } else {
                    "Previous"
                })
            );
            assert_eq!(
                saved.get_device_mic_profile_name("TEST").await.as_deref(),
                Some(if success && microphone {
                    "NextMic"
                } else {
                    "PreviousMic"
                })
            );
            if !microphone && !device.needs_profile_recovery() {
                assert_eq!(
                    usb.lock().unwrap().music_volume,
                    if success { 7 } else { 42 }
                );
            }
            if device.needs_profile_recovery() {
                assert!(
                    device
                        .perform_command(GoXLRCommand::SaveProfile())
                        .await
                        .is_err()
                );
            }
        }
    }
    // No recursive deletion: this fixture contains only the files created above.
    for entry in std::fs::read_dir(&directory)? {
        std::fs::remove_file(entry?.path())?;
    }
    std::fs::remove_dir(directory)?;
    Ok(())
}

#[tokio::test]
async fn physical_channel_mute_is_immediate_and_application_faders_are_isolated() -> Result<()> {
    let directory = std::env::temp_dir().join(format!("goxlr-mute-test-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()));
    std::fs::create_dir(&directory)?;
    let path = directory.join("settings.json");
    std::fs::write(&path, serde_json::to_vec(&serde_json::json!({"profile_directory":directory,"mic_profile_directory":directory,"backup_directory":directory,"samples_directory":directory,"devices":{"TEST":{"profile":"Previous","mic_profile":"PreviousMic"}}}))?)?;
    ProfileAdapter::default().save_as("Previous".into(), &directory, false)?;
    MicProfileAdapter::default().save_as("PreviousMic".into(), &directory, false)?;
    let settings = SettingsHandle::load(path).await?;
    let usb = Arc::new(Mutex::new(SimulatedUsb {connected:true,..Default::default()}));
            let hardware = HardwareStatus {
                versions: goxlr_types::FirmwareVersions {
                    firmware: VersionNumber::default(),
                    fpga_count: 0,
                    dice: VersionNumber::default(),
                },
                serial_number: "TEST".into(),
                manufactured_date: String::new(),
                device_type: DeviceType::Mini,
                colour_way: goxlr_ipc::ColourWay::Black,
                usb_device: goxlr_ipc::UsbProductInformation {
                    manufacturer_name: String::new(),
                    product_name: String::new(),
                    version: (0, 0, 0),
                    bus_number: 0,
                    address: 0,
                    identifier: None,
                },
            };
            let (events, _receiver) = tokio::sync::mpsc::channel(64);
            let mut device = Device::new(
                Box::new(SimulatedDevice(usb.clone())),
                hardware,
                &settings,
                events,
            )
            .await?;

    // A restart restores the saved volume; stationary Mini faders must not
    // replace it until a subsequent hardware reading actually changes.
    device.profile.set_channel_volume(ChannelName::Mic, 130)?;
    assert!(!device.update_volumes_to([37, 0, 0, 0]).await?);
    assert_eq!(device.profile.get_channel_volume(ChannelName::Mic), 130);
    assert!(!device.update_volumes_to([37, 0, 0, 0]).await?);
    assert!(device.update_volumes_to([42, 0, 0, 0]).await?);
    assert_eq!(device.profile.get_channel_volume(ChannelName::Mic), 42);

    let button: EnumSet<Buttons> = Buttons::Fader1Mute.into();
    let empty = EnumSet::new();
    for (fader, channel) in FaderName::iter().zip([ChannelName::Music, ChannelName::Chat, ChannelName::Mic, ChannelName::System]) {
        device.profile.set_fader_assignment(fader, channel);
    }
    for channel in [ChannelName::Game, ChannelName::Console, ChannelName::LineIn, ChannelName::Sample] {
        device.profile.set_channel_volume(channel, 0)?;
    }
    device.process_application_buttons(button, empty, [0;4], true).await?;
    let first = device.profile.get_mute_button_state(FaderName::A);
    assert!(first.0 || first.1, "mute must already apply at release, before the double-click timer");
    assert!(device.application_control.released_at[FaderName::A].is_none(), "unbound channels need no navigation timer");
    device.unmute_fader(FaderName::A).await?;
    settings.set_application_bindings("TEST".into(), vec![ApplicationBinding {name:"Test".into(),executable:std::env::current_exe()?.to_string_lossy().into_owned(),channel:ChannelName::Music,colour:"#009dff".into(),reset_volume:100,slot:Some(0)}]).await?;
    device.process_application_buttons(button, empty, [0;4], true).await?;
    let first = device.profile.get_mute_button_state(FaderName::A);
    assert!(first.0 || first.1, "bound channels also mute immediately");
    device.application_control.released_at[FaderName::A] = Some(Instant::now()-Duration::from_millis(400));
    device.tick_application_control().await?;
    assert_eq!(device.profile.get_mute_button_state(FaderName::A), first, "expiry must not apply mute a second time");
    device.unmute_fader(FaderName::A).await?;
    device.process_application_buttons(button, empty, [0;4], true).await?;
    device.process_application_buttons(button, empty, [0;4], true).await?;
    assert!(device.application_control.status.active, "double click still enters application mode");
    let restored = device.profile.get_mute_button_state(FaderName::A);
    assert!(!restored.0 && !restored.1, "navigation must restore the original channel mute");
    device.exit_application_mode().await?;
    device.mute_fader_to_all(FaderName::A, true).await?;
    let originally_muted = device.profile.get_mute_button_state(FaderName::A);
    device.process_application_buttons(button, empty, [0;4], true).await?;
    device.process_application_buttons(button, empty, [0;4], true).await?;
    assert_eq!(device.profile.get_mute_button_state(FaderName::A), originally_muted, "navigation preserves an already-muted channel too");
    device.exit_application_mode().await?;

    // Reproduce firmware latching: SetVolume changes the reported logical
    // volume and prevents subsequent physical changes until pickup. Faders in
    // application mode must use isolated channels with no recurring SetVolume.
    let app_path = |slot| directory.join(format!("app-{slot}.exe")).to_string_lossy().into_owned();
    for slot in [0, 1, 2, 4] {
        std::fs::write(app_path(slot), b"isolated fixture; never executed")?;
    }
    let bindings = [0, 1, 2, 4].into_iter().map(|slot| ApplicationBinding {
        name: format!("App {slot}"), executable: app_path(slot),
        channel: ChannelName::Music, colour: "#009dff".into(),
        reset_volume: 50, slot: Some(slot),
    }).collect();
    settings.set_application_bindings("TEST".into(), bindings).await?;
    let channels = [ChannelName::Mic, ChannelName::Chat, ChannelName::Music, ChannelName::System];
    for (fader, channel) in FaderName::iter().zip(channels) {
        device.profile.set_fader_assignment(fader, channel);
        device.profile.set_channel_volume(channel, 123)?;
        usb.lock().unwrap().channel_volumes[channel] = 123;
    }
    usb.lock().unwrap().physical_volumes = EnumMap::from_array([40, 60, 70, 90]);
    device.fader_last_seen = EnumMap::from_array([0; 4]);
    device.enter_application_mode(FaderName::C, [0; 4]).await?;
    assert!(device.application_control.status.active);
    let isolated = device.application_control.hardware_faders.unwrap();
    for channel in isolated.values() {
        let (left, right) = InputDevice::from_basic(&device.get_basic_input_from_channel(*channel).unwrap());
        for input in [left, right] {
            let state = usb.lock().unwrap();
            let request = state.requests.iter().rev().find(|(command, _)| *command == Command::SetRouting(input)).unwrap();
            assert!(request.1.iter().all(|value| *value == 0), "isolated channels must reach no output or effect feed");
        }
    }
    assert!(device.application_control.fader_colours.unwrap()[0][0][..3].iter().any(|v| *v > 0));
    let volume_writes = usb.lock().unwrap().volume_writes;
    for (index, fader) in FaderName::iter().enumerate() {
        // First acquire the zero hardware latch, then the application's 50%.
        let volumes = usb.lock().unwrap().move_fader(fader, 0);
        device.update_volumes_to(volumes).await?;
        let volumes = usb.lock().unwrap().move_fader(fader, 153);
        device.update_volumes_to(volumes).await?;
        assert_eq!(usb.lock().unwrap().channel_volumes[channels[index]], 123,
            "audible hardware channel must stay fixed for slot {index}");
        assert_eq!(device.profile.get_channel_volume(channels[index]), 123);
        if index < 3 {
            assert_eq!(device.application_control.pending_volume[FaderName::iter().nth(index).unwrap()],
                Some((app_path(index), 60)));
        } else {
            assert!(device.application_control.pending_volume[FaderName::D].is_none());
        }
    }
    // A second movement must work without returning to the hardware latch.
    let volumes = usb.lock().unwrap().move_fader(FaderName::C, 80);
    device.update_volumes_to(volumes).await?;
    assert_eq!(device.application_control.pending_volume[FaderName::C], Some((app_path(2), 31)));
    assert_eq!(usb.lock().unwrap().volume_writes, volume_writes, "no SetVolume feedback during application movement");
    device.sync_application_fader_lighting().await?;
    assert_eq!(device.application_control.fader_colours.unwrap()[2][0], [0; 4]);
    assert_eq!(device.application_control.fader_colours.unwrap()[3], [[0; 4]; 2], "empty slot strip is dark");
    assert_eq!(usb.lock().unwrap().display_modes[FaderName::C], [0, 0], "LED strip represents volume, not the isolated channel's audio meter");
    device.application_control.status.muted[2] = Some(true);
    device.sync_application_fader_lighting().await?;
    assert_eq!(device.application_control.fader_colours.unwrap()[2], [[0; 4]; 2]);
    device.change_application_page(true, device.fader_last_seen.into_array()).await?;
    // Slot C is empty on page 2, while A has to cross its pickup target.
    for (fader, new_volume) in [(FaderName::A, 140), (FaderName::C, 80), (FaderName::A, 80)] {
        let volumes = usb.lock().unwrap().move_fader(fader, new_volume);
        device.update_volumes_to(volumes).await?;
        assert_eq!(usb.lock().unwrap().channel_volumes[device.profile.get_fader_assignment(fader)], 123);
        if fader == FaderName::A && new_volume == 140 {
            assert!(device.application_control.pending_volume[fader].is_none(), "pickup must not change the application yet");
        }
    }
    assert_eq!(device.application_control.pending_volume[FaderName::A], Some((app_path(4), 31)));
    assert!(device.application_control.pending_volume[FaderName::C].is_none());
    device.exit_application_mode().await?;
    // Normal mode resumes channel volume control without a stale app command.
    assert!(device.application_control.pending_volume.values().all(Option::is_none));
    let volumes = usb.lock().unwrap().read_volumes();
    device.update_volumes_to(volumes).await?;
    let volumes = usb.lock().unwrap().move_fader(FaderName::C, 130);
    device.update_volumes_to(volumes).await?;
    assert_eq!(device.profile.get_channel_volume(ChannelName::Music), 130);
    assert_eq!(usb.lock().unwrap().channel_volumes[ChannelName::Music], 130);
    for (fader, channel) in FaderName::iter().zip(channels) {
        assert_eq!(usb.lock().unwrap().hardware_faders[fader], channel);
        assert_eq!(usb.lock().unwrap().display_modes[fader], [device.profile.is_fader_gradient(fader) as u8, device.profile.is_fader_meter(fader) as u8]);
    }
    for channel in isolated.values() {
        assert_eq!(usb.lock().unwrap().channel_volumes[*channel], device.profile.get_channel_volume(*channel));
    }
    // Refuse entry rather than sacrifice an audible spare channel.
    device.profile.set_channel_volume(ChannelName::Console, 80)?;
    let writes = usb.lock().unwrap().volume_writes;
    device.enter_application_mode(FaderName::C, volumes).await?;
    assert!(!device.application_control.status.active);
    assert!(device.application_control.status.message.is_some());
    assert_eq!(usb.lock().unwrap().volume_writes, writes);
    device.profile.set_channel_volume(ChannelName::Console, 0)?;
    device.enter_application_mode(FaderName::C, volumes).await?;
    device.perform_command(GoXLRCommand::SetVolume(ChannelName::Music, 145)).await?;
    assert!(!device.application_control.status.active, "editing hardware must relinquish temporary channels first");
    assert_eq!(usb.lock().unwrap().channel_volumes[ChannelName::Music], 145);
    // Partial entry restores all assignments instead of leaving silent controls.
    {
        let mut state = usb.lock().unwrap();
        state.fail = Some(Command::SetFader(FaderName::C));
        state.failures_left = 1;
    }
    assert!(device.enter_application_mode(FaderName::C, volumes).await.is_err());
    assert!(device.application_control.hardware_faders.is_none());
    assert!(!device.needs_profile_recovery());
    for (fader, channel) in FaderName::iter().zip(channels) {
        assert_eq!(usb.lock().unwrap().hardware_faders[fader], channel);
    }
    drop(device);
    drop(settings);
    std::fs::remove_dir_all(directory)?;
    Ok(())
}
