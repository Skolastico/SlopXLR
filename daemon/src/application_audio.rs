use anyhow::Result;

use crate::button_actions::RunningApplication;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApplicationRouteStatus {
    pub executable: String,
    pub channel: goxlr_types::ChannelName,
    pub state: String,
    pub message: String,
}

#[cfg(windows)]
mod windows_audio {
    use super::*;
    use anyhow::Context;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::{Path, PathBuf};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Media::Audio::{
        AudioSessionStateExpired, DEVICE_STATE_ACTIVE, IAudioSessionControl2,
        IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
        eRender,
    };
    use windows::Win32::System::Com::{
        CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::core::{Interface, PWSTR};

    struct ComApartment;

    impl ComApartment {
        fn initialize() -> Result<Self> {
            unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
                .ok()
                .context("Não foi possível inicializar o controlo de áudio do Windows")?;
            Ok(Self)
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }

    struct Session {
        pid: u32,
        endpoint: String,
        executable: PathBuf,
        control: ISimpleAudioVolume,
        volume: u8,
        muted: bool,
    }

    fn process_path(pid: u32) -> Option<PathBuf> {
        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        let result = unsafe {
            QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut length,
            )
        };
        let _ = unsafe { CloseHandle(process) };
        result.ok()?;
        Some(PathBuf::from(OsString::from_wide(
            &buffer[..length as usize],
        )))
    }

    fn sessions() -> Result<Vec<Session>> {
        let enumerator: IMMDeviceEnumerator = unsafe {
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .context("Não foi possível localizar os dispositivos de áudio do Windows")?
        };
        let devices = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
        let mut processes = HashMap::new();
        let mut result = Vec::new();

        for device_index in 0..unsafe { devices.GetCount()? } {
            let device = unsafe { devices.Item(device_index)? };
            let id = unsafe { device.GetId()? };
            let endpoint = unsafe { id.to_string() };
            unsafe { windows::Win32::System::Com::CoTaskMemFree(Some(id.0.cast())) };
            let endpoint = endpoint?;
            let manager: IAudioSessionManager2 = match unsafe { device.Activate(CLSCTX_ALL, None) }
            {
                Ok(manager) => manager,
                Err(_) => continue,
            };
            let enumerated = unsafe { manager.GetSessionEnumerator()? };
            for session_index in 0..unsafe { enumerated.GetCount()? } {
                let session = unsafe { enumerated.GetSession(session_index)? };
                let metadata: IAudioSessionControl2 = match session.cast() {
                    Ok(control) => control,
                    Err(_) => continue,
                };
                match unsafe { metadata.GetState() } {
                    Ok(state) if state != AudioSessionStateExpired => {}
                    _ => continue,
                }
                let pid = match unsafe { metadata.GetProcessId() } {
                    Ok(pid) if pid != 0 => pid,
                    _ => continue,
                };
                let executable = processes
                    .entry(pid)
                    .or_insert_with(|| process_path(pid))
                    .clone();
                let Some(executable) = executable else {
                    continue;
                };
                let control: ISimpleAudioVolume = match session.cast() {
                    Ok(control) => control,
                    Err(_) => continue,
                };
                let volume = unsafe { control.GetMasterVolume()? };
                let muted = unsafe { control.GetMute()?.as_bool() };
                result.push(Session {
                    pid,
                    endpoint: endpoint.clone(),
                    executable,
                    control,
                    volume: (volume.clamp(0.0, 1.0) * 100.0).round() as u8,
                    muted,
                });
            }
        }
        Ok(result)
    }

    fn matching_sessions(path: &Path) -> Result<Vec<Session>> {
        let expected = path.to_string_lossy();
        Ok(sessions()?
            .into_iter()
            .filter(|session| {
                session
                    .executable
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&expected)
            })
            .collect())
    }

    pub fn list() -> Result<Vec<RunningApplication>> {
        let _apartment = ComApartment::initialize()?;
        let mut grouped: HashMap<String, (PathBuf, u32, usize, bool)> = HashMap::new();
        for session in sessions()? {
            let key = session.executable.to_string_lossy().to_lowercase();
            let entry = grouped
                .entry(key)
                .or_insert_with(|| (session.executable.clone(), 0, 0, false));
            entry.1 += session.volume as u32;
            entry.2 += 1;
            entry.3 |= session.muted;
        }
        let mut applications = grouped
            .into_values()
            .map(|(path, volume, count, muted)| RunningApplication {
                name: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                executable: path.to_string_lossy().into_owned(),
                volume: (volume / count.max(1) as u32) as u8,
                muted,
            })
            .collect::<Vec<_>>();
        applications
            .sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
        Ok(applications)
    }

    pub fn toggle_mute(path: &str) -> Result<Option<bool>> {
        let _apartment = ComApartment::initialize()?;
        let sessions = matching_sessions(Path::new(path))?;
        if sessions.is_empty() {
            return Ok(None);
        }
        let mute = !sessions.iter().all(|session| session.muted);
        let context = windows::core::GUID::zeroed();
        for session in sessions {
            unsafe { session.control.SetMute(mute, &context)? };
        }
        Ok(Some(mute))
    }

    pub fn set_volumes(targets: Vec<(String, u8)>) -> Result<Vec<String>> {
        let _apartment = ComApartment::initialize()?;
        let targets = targets
            .into_iter()
            .map(|(path, volume)| (path.to_lowercase(), volume.min(100)))
            .collect::<HashMap<_, _>>();
        let sessions = sessions()?;
        let context = windows::core::GUID::zeroed();
        let mut applied = HashMap::new();
        for session in sessions {
            let path = session.executable.to_string_lossy().to_lowercase();
            if let Some(volume) = targets.get(&path) {
                unsafe {
                    session
                        .control
                        .SetMasterVolume(*volume as f32 / 100.0, &context)?
                };
                applied.insert(path, ());
            }
        }
        Ok(applied.into_keys().collect())
    }

    // ABI and endpoint encoding verified against EarTrumpet (MIT):
    // https://github.com/File-New-Project/EarTrumpet/tree/master/EarTrumpet/Interop/MMDeviceAPI
    // Windows exposes this policy through an internal interface; try both known versions.
    #[repr(C)]
    struct PolicyVtable {
        base: windows::core::IInspectable_Vtbl,
        unused: [usize; 19],
        set: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            u32,
            i32,
            i32,
            *mut std::ffi::c_void,
        ) -> windows::core::HRESULT,
        get: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            u32,
            i32,
            i32,
            *mut *mut std::ffi::c_void,
        ) -> windows::core::HRESULT,
    }

    windows::core::link!("api-ms-win-core-winrt-l1-1-0.dll" "system" fn RoGetActivationFactory(class: *mut std::ffi::c_void, iid: *const windows::core::GUID, factory: *mut *mut std::ffi::c_void) -> windows::core::HRESULT);

    struct Policy(windows::core::IUnknown);

    impl Policy {
        fn new() -> Result<Self> {
            use windows::core::{GUID, HSTRING, IUnknown};
            let class = HSTRING::from("Windows.Media.Internal.AudioPolicyConfig");
            let mut last = None;
            for iid in [
                GUID::from_u128(0xab3d4648_e242_459f_b02f_541c70306324),
                GUID::from_u128(0x2a59116d_6c4f_45e0_a74f_707e3fef9258),
            ] {
                let mut raw = std::ptr::null_mut();
                let result = unsafe {
                    RoGetActivationFactory(std::mem::transmute_copy(&class), &iid, &mut raw)
                }
                .ok();
                match result {
                    Ok(()) => return Ok(Self(unsafe { IUnknown::from_raw(raw) })),
                    Err(error) => last = Some(error),
                }
            }
            Err(last.unwrap().into())
        }

        fn vtable(&self) -> &PolicyVtable {
            // Both factory IIDs have this identical IInspectable-based vtable layout.
            unsafe { &**(self.0.as_raw() as *const *const PolicyVtable) }
        }

        fn route(&self, pid: u32, endpoint: &str) -> Result<()> {
            let desired = windows::core::HSTRING::from(format!(
                r"\\?\SWD#MMDEVAPI#{endpoint}#{{e6327cad-dcec-4949-ae8a-991e976a79d2}}"
            ));
            // Console, multimedia and communications must all select the same output.
            for role in 0..3 {
                let mut raw = std::ptr::null_mut();
                unsafe { (self.vtable().get)(self.0.as_raw(), pid, 0, role, &mut raw) }.ok()?;
                let current: windows::core::HSTRING = unsafe { std::mem::transmute(raw) };
                if current != desired {
                    unsafe {
                        (self.vtable().set)(
                            self.0.as_raw(),
                            pid,
                            0,
                            role,
                            std::mem::transmute_copy(&desired),
                        )
                    }
                    .ok()?;
                    let mut verified = std::ptr::null_mut();
                    unsafe { (self.vtable().get)(self.0.as_raw(), pid, 0, role, &mut verified) }
                        .ok()?;
                    let verified: windows::core::HSTRING = unsafe { std::mem::transmute(verified) };
                    anyhow::ensure!(
                        verified == desired,
                        "O Windows não confirmou a saída escolhida."
                    );
                }
            }
            Ok(())
        }
    }

    fn playback_endpoints() -> Result<Vec<(String, String)>> {
        use windows::Win32::Devices::FunctionDiscovery::{
            PKEY_Device_DeviceDesc, PKEY_DeviceInterface_FriendlyName,
        };
        use windows::Win32::System::Com::StructuredStorage::{
            PropVariantClear, PropVariantToStringAlloc,
        };
        use windows::Win32::System::Com::{CoTaskMemFree, STGM_READ};
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        let devices = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
        let mut endpoints = Vec::new();
        for index in 0..unsafe { devices.GetCount()? } {
            let device = unsafe { devices.Item(index)? };
            let store = unsafe { device.OpenPropertyStore(STGM_READ)? };
            let read_property = |key| -> Result<String> {
                let mut value = unsafe { store.GetValue(key)? };
                let text = unsafe { PropVariantToStringAlloc(&value) };
                unsafe { PropVariantClear(&mut value)? };
                let text = text?;
                let result = unsafe { text.to_string() };
                unsafe { CoTaskMemFree(Some(text.0.cast())) };
                Ok(result?)
            };
            let product = read_property(&PKEY_DeviceInterface_FriendlyName).unwrap_or_default();
            if !product.to_lowercase().contains("goxlr mini") {
                continue;
            }
            let channel = read_property(&PKEY_Device_DeviceDesc)?;
            let name = format!("{channel} ({product})");
            let id = unsafe { device.GetId()? };
            let text = unsafe { id.to_string() };
            unsafe { CoTaskMemFree(Some(id.0.cast())) };
            endpoints.push((text?, name));
        }
        Ok(endpoints)
    }

    fn choose_endpoint<'a>(
        endpoints: &'a [(String, String)],
        channel: &str,
    ) -> Result<&'a (String, String)> {
        let mut candidates = endpoints.iter().filter(|(_, name)| {
            name.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .next()
                == Some(channel)
        });
        let selected = candidates
            .next()
            .context("A saída deste canal do GoXLR Mini não está disponível.")?;
        anyhow::ensure!(
            candidates.next().is_none(),
            "Existem várias saídas GoXLR para este canal. Não foi possível escolher o dispositivo com segurança."
        );
        Ok(selected)
    }

    #[cfg(test)]
    #[test]
    fn routing_contract() {
        // Prevent moving policy calls onto a different COM slot, or choosing the wrong channel.
        assert_eq!(
            std::mem::offset_of!(PolicyVtable, set),
            25 * std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::offset_of!(PolicyVtable, get),
            26 * std::mem::size_of::<usize>()
        );
        let mut endpoints = vec![
            (
                "music-id".to_owned(),
                "Music (TC-HELICON GoXLR Mini)".to_owned(),
            ),
            (
                "system-id".to_owned(),
                "System (TC-HELICON GoXLR Mini)".to_owned(),
            ),
        ];
        assert_eq!(choose_endpoint(&endpoints, "music").unwrap().0, "music-id");
        assert!(choose_endpoint(&endpoints, "chat").is_err());
        endpoints.push((
            "other-music".to_owned(),
            "Music (another GoXLR Mini)".to_owned(),
        ));
        assert!(choose_endpoint(&endpoints, "music").is_err());
    }

    pub fn route_applications(
        bindings: Vec<crate::button_actions::ApplicationBinding>,
    ) -> Result<Vec<ApplicationRouteStatus>> {
        use goxlr_types::ChannelName;
        let _apartment = ComApartment::initialize()?;
        let endpoints = playback_endpoints()?;
        let sessions = sessions()?;
        let policy = Policy::new();
        let mut result = Vec::new();
        for binding in bindings {
            let route = (|| -> Result<(&str, String)> {
                let channel = match binding.channel {
                    ChannelName::System => "system",
                    ChannelName::Game => "game",
                    ChannelName::Chat => "chat",
                    ChannelName::Music => "music",
                    ChannelName::Sample => "sample",
                    _ => anyhow::bail!(
                        "Este canal é uma entrada física e não recebe áudio de aplicações do Windows."
                    ),
                };
                let (endpoint, name) = choose_endpoint(&endpoints, channel)?;
                let matching = sessions
                    .iter()
                    .filter(|s| {
                        s.executable
                            .to_string_lossy()
                            .eq_ignore_ascii_case(&binding.executable)
                    })
                    .collect::<Vec<_>>();
                if matching.is_empty() {
                    return Ok((
                        "waiting",
                        "À espera de a aplicação reproduzir som.".to_owned(),
                    ));
                }
                let policy = policy.as_ref().map_err(|e| {
                    anyhow::anyhow!(
                        "O Windows não disponibilizou o encaminhamento por aplicação: {e}"
                    )
                })?;
                let pids = matching
                    .iter()
                    .map(|s| s.pid)
                    .collect::<std::collections::HashSet<_>>();
                for pid in pids {
                    policy.route(pid, endpoint)?;
                }
                if matching.iter().all(|s| &s.endpoint == endpoint) {
                    Ok(("applied", format!("Áudio em {name}.")))
                } else {
                    Ok((
                        "pending",
                        format!(
                            "Saída definida: {name}. Se o áudio não mudar, volta a abrir a aplicação."
                        ),
                    ))
                }
            })();
            let (state, message) = match route {
                Ok((state, message)) => (state.to_owned(), message),
                Err(error) => ("error".to_owned(), error.to_string()),
            };
            result.push(ApplicationRouteStatus {
                executable: binding.executable,
                channel: binding.channel,
                state,
                message,
            });
        }
        Ok(result)
    }
}

pub async fn routing_worker(
    settings: crate::settings::SettingsHandle,
    usb: crate::primary_worker::DeviceSender,
    mut shutdown: crate::shutdown::Shutdown,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = shutdown.recv() => break,
            _ = interval.tick() => {
                let work = async {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    usb.send(crate::primary_worker::DeviceCommand::SendDaemonStatus(tx)).await.ok()?;
                    let status = tokio::time::timeout(std::time::Duration::from_secs(2), rx).await.ok()?.ok()?;
                    for (serial, mixer) in status.mixers {
                        if mixer.hardware.device_type != goxlr_types::DeviceType::Mini { continue; }
                        let bindings = settings.get_application_bindings(&serial).await;
                        let routes = if bindings.is_empty() { Vec::new() } else { route_applications(bindings.clone()).await.unwrap_or_else(|error| bindings.into_iter().map(|binding| ApplicationRouteStatus { executable: binding.executable, channel: binding.channel, state: "error".to_owned(), message: error.to_string() }).collect()) };
                        settings.set_application_routes(&serial, routes).await;
                    }
                    Some(())
                };
                tokio::select! { _ = shutdown.recv() => break, _ = work => {} }
            }
        }
    }
}

#[cfg(windows)]
async fn route_applications(
    bindings: Vec<crate::button_actions::ApplicationBinding>,
) -> Result<Vec<ApplicationRouteStatus>> {
    tokio::task::spawn_blocking(move || windows_audio::route_applications(bindings)).await?
}

#[cfg(not(windows))]
async fn route_applications(
    _bindings: Vec<crate::button_actions::ApplicationBinding>,
) -> Result<Vec<ApplicationRouteStatus>> {
    anyhow::bail!("O encaminhamento por aplicação está disponível apenas no Windows.")
}

#[cfg(windows)]
pub async fn running_applications() -> Result<Vec<RunningApplication>> {
    tokio::task::spawn_blocking(windows_audio::list).await?
}

#[cfg(not(windows))]
pub async fn running_applications() -> Result<Vec<RunningApplication>> {
    Ok(Vec::new())
}

#[cfg(windows)]
pub async fn toggle_application_mute(executable: String) -> Result<Option<bool>> {
    tokio::task::spawn_blocking(move || windows_audio::toggle_mute(&executable)).await?
}

#[cfg(not(windows))]
pub async fn toggle_application_mute(_executable: String) -> Result<Option<bool>> {
    anyhow::bail!("O controlo de aplicações está disponível apenas no Windows.")
}

#[cfg(windows)]
pub async fn set_application_volumes(targets: Vec<(String, u8)>) -> Result<Vec<String>> {
    tokio::task::spawn_blocking(move || windows_audio::set_volumes(targets)).await?
}

#[cfg(not(windows))]
pub async fn set_application_volumes(_targets: Vec<(String, u8)>) -> Result<Vec<String>> {
    anyhow::bail!("O controlo de aplicações está disponível apenas no Windows.")
}
