use anyhow::{Result, bail};
use enumset::EnumSet;
use goxlr_types::{ChannelName, FaderName, SampleBank, SampleButtons};
use goxlr_usb::buttonstate::Buttons;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "StoredLayer")]
pub struct ButtonLayer {
    pub enabled: bool,
    pub pages: Vec<ActionPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPage {
    pub name: String,
    pub actions: [ButtonAction; 5],
    #[serde(default = "default_colours")]
    pub colours: [String; 6],
    #[serde(default = "default_pressed_colours")]
    pub pressed_colours: [String; 6],
}

pub fn default_colours() -> [String; 6] {
    std::array::from_fn(|_| "#b06cff".into())
}

pub fn default_pressed_colours() -> [String; 6] {
    std::array::from_fn(|_| "#ffffff".into())
}

pub const LAYER_BUTTONS: [Buttons; 6] = [
    Buttons::Fader1Mute,
    Buttons::Fader2Mute,
    Buttons::Fader3Mute,
    Buttons::Fader4Mute,
    Buttons::Bleep,
    Buttons::MicrophoneMute,
];

impl ActionPage {
    pub fn lighting(&self, pressed: EnumSet<Buttons>) -> Result<[[u8; 4]; 6]> {
        let mut colours = [[0; 4]; 6];
        for (index, button) in LAYER_BUTTONS.iter().enumerate() {
            let rgb = if pressed.contains(*button) {
                &self.pressed_colours[index]
            } else {
                &self.colours[index]
            };
            if rgb.len() != 7
                || !rgb.starts_with('#')
                || !rgb.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
            {
                bail!("Escolhe uma cor válida no formato #RRGGBB.");
            }
            colours[index] = goxlr_profile_loader::components::colours::Colour::fromrgb(&rgb[1..])?
                .to_reverse_bytes();
        }
        Ok(colours)
    }
}

pub fn overlay_lighting(map: &mut [u8; 520], new_format: bool, colours: &[[u8; 4]; 6]) {
    use goxlr_usb::colouring::ColourTargets;
    let targets = [
        ColourTargets::Fader1Mute,
        ColourTargets::Fader2Mute,
        ColourTargets::Fader3Mute,
        ColourTargets::Fader4Mute,
        ColourTargets::Bleep,
        ColourTargets::MicrophoneMute,
    ];
    for (target, colour) in targets.iter().zip(colours) {
        for slot in 0..2 {
            let offset = target.position(slot, new_format);
            map[offset..offset + 4].copy_from_slice(colour);
        }
    }
}

impl Default for ButtonLayer {
    fn default() -> Self {
        Self {
            enabled: false,
            pages: vec![ActionPage {
                name: "Página 1".into(),
                actions: Default::default(),
                colours: default_colours(),
                pressed_colours: default_pressed_colours(),
            }],
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct StoredLayer {
    enabled: bool,
    pages: Option<Vec<ActionPage>>,
    actions: Option<[ButtonAction; 5]>,
}

impl From<StoredLayer> for ButtonLayer {
    fn from(stored: StoredLayer) -> Self {
        Self {
            enabled: stored.enabled,
            pages: stored.pages.unwrap_or_else(|| {
                vec![ActionPage {
                    name: "Página 1".into(),
                    actions: stored.actions.unwrap_or_default(),
                    colours: default_colours(),
                    pressed_colours: default_pressed_colours(),
                }]
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct LayerRuntime {
    pub active: bool,
    pub page: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ButtonAction {
    #[default]
    None,
    LoadProfile {
        profile: String,
    },
    ToggleMute {
        fader: FaderName,
    },
    MediaKey {
        key: u16,
    },
    Shortcut {
        keys: Vec<u16>,
    },
    OpenProgram {
        path: String,
    },
    PlaySample { bank: SampleBank, button: SampleButtons },
    StopSample { bank: SampleBank, button: SampleButtons },
}

impl ButtonLayer {
    pub fn validate(&self) -> Result<()> {
        if self.pages.is_empty()
            || self
                .pages
                .iter()
                .any(|page| page.name.trim().is_empty() || page.name.len() > 200)
        {
            bail!("Cria pelo menos uma página e dá um nome a cada página.");
        }
        for page in &self.pages {
            page.lighting(EnumSet::empty())?;
            page.lighting(LAYER_BUTTONS.into_iter().collect())?;
        }
        for action in self.pages.iter().flat_map(|page| &page.actions) {
            action.validate()?;
        }
        Ok(())
    }
}

impl ButtonAction {
    pub fn validate(&self) -> Result<()> {
        let action = self;
        match action {
            ButtonAction::None | ButtonAction::ToggleMute { .. } | ButtonAction::PlaySample { .. } | ButtonAction::StopSample { .. } => {}
            ButtonAction::LoadProfile { profile } => {
                if profile.is_empty()
                    || profile.len() > 200
                    || profile.contains(['/', '\\', ':'])
                    || profile == "."
                    || profile == ".."
                {
                    bail!("Escolhe um nome de perfil válido.");
                }
            }
            ButtonAction::MediaKey { key } => {
                if !matches!(key, 0xAD..=0xB3) {
                    bail!("Tecla multimédia inválida.");
                }
            }
            ButtonAction::Shortcut { keys } => {
                let primary = keys.last().copied().unwrap_or(0);
                let valid_primary = matches!(primary, 0x30..=0x39 | 0x41..=0x5A | 0x70..=0x87);
                let valid_modifiers = keys
                    .iter()
                    .take(keys.len().saturating_sub(1))
                    .all(|key| matches!(key, 0x10..=0x12 | 0x5B));
                if keys.is_empty()
                    || keys.len() > 5
                    || !valid_primary
                    || !valid_modifiers
                    || (keys.len() == 1 && primary < 0x70)
                {
                    bail!("Atalho inválido. Usa modificadores, letras, números ou F1–F24.");
                }
                let mut distinct = keys.clone();
                distinct.sort_unstable();
                distinct.dedup();
                if distinct.len() != keys.len() {
                    bail!("O atalho contém teclas repetidas.");
                }
            }
            ButtonAction::OpenProgram { path } => {
                let path = std::path::Path::new(path);
                if !path.is_absolute()
                    || !path.is_file()
                    || !path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
                {
                    bail!("Indica o caminho completo de uma aplicação .exe existente.");
                }
            }
        }
        #[cfg(not(windows))]
        if matches!(
            action,
            ButtonAction::MediaKey { .. }
                | ButtonAction::Shortcut { .. }
                | ButtonAction::OpenProgram { .. }
        ) {
            bail!("Esta ação está disponível apenas no Windows.");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionReport {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationBinding {
    pub name: String,
    pub executable: String,
    pub channel: ChannelName,
    pub colour: String,
    pub reset_volume: u8,
    /// Zero-based position: page * 4 + fader. Missing in older configurations.
    #[serde(default)]
    pub slot: Option<usize>,
}

pub fn assign_application_slots(bindings: &mut [ApplicationBinding]) {
    for index in 0..bindings.len() {
        if bindings[index].slot.is_some() {
            continue;
        }
        let channel = bindings[index].channel;
        let slot = (0..)
            .find(|slot| {
                !bindings
                    .iter()
                    .any(|app| app.channel == channel && app.slot == Some(*slot))
            })
            .unwrap();
        bindings[index].slot = Some(slot);
    }
}

pub fn application_page_count(bindings: &[ApplicationBinding], channel: ChannelName) -> usize {
    bindings
        .iter()
        .filter(|app| app.channel == channel)
        .filter_map(|app| app.slot.map(|slot| slot / 4 + 1))
        .max()
        .unwrap_or(0)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ApplicationModeStatus {
    pub active: bool,
    pub channel: Option<ChannelName>,
    pub page: usize,
    pub page_count: usize,
    pub exit_fader: Option<FaderName>,
    pub pickup: [bool; 4],
    pub volume: [Option<u8>; 4],
    pub muted: [Option<bool>; 4],
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunningApplication {
    pub executable: String,
    pub name: String,
    pub volume: u8,
    pub muted: bool,
}

pub fn validate_application_bindings(bindings: &[ApplicationBinding]) -> Result<()> {
    let mut positioned = bindings.to_vec();
    assign_application_slots(&mut positioned);
    let mut positions = Vec::new();
    let mut executables: Vec<String> = Vec::new();
    for binding in &positioned {
        let position = (binding.channel, binding.slot.unwrap());
        if position.1 >= 400 || positions.contains(&position) {
            bail!("Escolhe um slider livre numa página de 1 a 100 para cada aplicação do canal.");
        }
        positions.push(position);
        if binding.reset_volume > 100 {
            bail!("O volume de reposição deve estar entre 0 e 100%.");
        }
        let path = std::path::Path::new(&binding.executable);
        if binding.name.trim().is_empty() || binding.name.len() > 80 {
            bail!("Dá um nome com até 80 caracteres a cada aplicação.");
        }
        if !path.is_absolute()
            || !path.is_file()
            || !path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
        {
            bail!(
                "A aplicação '{}' já não existe neste caminho.",
                binding.name
            );
        }
        if !matches!(
            binding.channel,
            ChannelName::System
                | ChannelName::Game
                | ChannelName::Chat
                | ChannelName::Music
                | ChannelName::Console
                | ChannelName::LineIn
                | ChannelName::Sample
        ) {
            bail!("Escolhe um canal de reprodução do GoXLR.");
        }
        if binding.colour.len() != 7
            || !binding.colour.starts_with('#')
            || !binding.colour.as_bytes()[1..]
                .iter()
                .all(u8::is_ascii_hexdigit)
        {
            bail!("Escolhe uma cor válida no formato #RRGGBB.");
        }
        if executables
            .iter()
            .any(|path| path.eq_ignore_ascii_case(&binding.executable))
        {
            bail!("Cada aplicação só pode pertencer a um canal.");
        }
        executables.push(binding.executable.clone());
    }
    Ok(())
}

pub fn action_index(button: Buttons) -> Option<usize> {
    match button {
        Buttons::Fader1Mute => Some(0),
        Buttons::Fader2Mute => Some(1),
        Buttons::Fader3Mute => Some(2),
        Buttons::Fader4Mute => Some(3),
        Buttons::Bleep => Some(4),
        _ => None,
    }
}

#[derive(Default)]
pub struct LayerTracker {
    pub runtime: LayerRuntime,
    consumed: EnumSet<Buttons>,
    cough_reserved: bool,
    cough_started: Option<std::time::Instant>,
    normal_cough: bool,
    long_handled: bool,
    clicks: u8,
    released_at: Option<std::time::Instant>,
}

#[derive(Default)]
pub struct LayerEvents {
    pub actions: Vec<(Buttons, usize)>,
    pub suppressed: EnumSet<Buttons>,
    pub normal_down: bool,
    pub normal_up: bool,
    pub cancel_normal: bool,
}

impl LayerTracker {
    pub fn tick(
        &mut self,
        enabled: bool,
        page_count: usize,
        now: std::time::Instant,
    ) -> LayerEvents {
        let mut events = LayerEvents::default();
        self.runtime.page %= page_count.max(1);
        if !enabled {
            self.runtime.active = false;
            self.clicks = 0;
            self.released_at = None;
            return events;
        }
        if !self.long_handled
            && self.cough_started.is_some_and(|start| {
                now.duration_since(start) >= std::time::Duration::from_millis(700)
            })
        {
            self.long_handled = true;
            self.runtime.active = !self.runtime.active;
            self.clicks = 0;
            self.released_at = None;
            events.cancel_normal = self.normal_cough;
            self.normal_cough = false;
        }
        // Resolve double click only after the third-click window has closed.
        if self.cough_started.is_none()
            && self.released_at.is_some_and(|release| {
                now.duration_since(release) >= std::time::Duration::from_millis(350)
            })
        {
            if self.runtime.active && page_count > 0 {
                if self.clicks == 2 {
                    self.runtime.page = (self.runtime.page + 1) % page_count;
                }
                if self.clicks == 3 {
                    self.runtime.page = (self.runtime.page + page_count - 1) % page_count;
                }
            }
            self.clicks = 0;
            self.released_at = None;
        }
        events
    }

    // Button ownership stays latched through release, including a mode change.
    pub fn update(
        &mut self,
        previous: EnumSet<Buttons>,
        current: EnumSet<Buttons>,
        enabled: bool,
        page_count: usize,
        now: std::time::Instant,
    ) -> LayerEvents {
        let mut events = self.tick(enabled, page_count, now);
        let modifier = Buttons::MicrophoneMute;
        if current.contains(modifier) && !previous.contains(modifier) {
            self.cough_reserved = enabled;
            self.long_handled = false;
            if enabled {
                self.cough_started = Some(now);
                self.normal_cough = !self.runtime.active;
                events.normal_down = self.normal_cough;
            }
        }
        events.suppressed = self.consumed;
        if self.cough_reserved {
            events.suppressed.insert(modifier);
        }
        if self.runtime.active {
            for button in current.difference(previous) {
                if action_index(button).is_some() {
                    self.consumed.insert(button);
                    events.suppressed.insert(button);
                    events.actions.push((button, self.runtime.page));
                }
            }
        }
        self.consumed = self.consumed.intersection(current);
        if previous.contains(modifier) && !current.contains(modifier) {
            if self.cough_reserved {
                events.normal_up = self.normal_cough;
                if !self.long_handled && self.runtime.active && enabled {
                    self.clicks = self.clicks.saturating_add(1);
                    self.released_at = Some(now);
                }
            }
            self.normal_cough = false;
            self.cough_reserved = false;
            self.cough_started = None;
        }
        events
    }
}

pub fn execute_external(action: ButtonAction) -> Result<()> {
    #[cfg(windows)]
    {
        match action {
            ButtonAction::MediaKey { key } => send_keys(&[key]),
            ButtonAction::Shortcut { keys } => send_keys(&keys),
            ButtonAction::OpenProgram { path } => {
                use std::os::windows::process::CommandExt;
                // Launch the executable directly; never interpret a shell command.
                std::process::Command::new(path)
                    .creation_flags(0x08000000)
                    .spawn()?;
                Ok(())
            }
            _ => bail!("Ação externa inválida."),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = action;
        bail!("Esta ação está disponível apenas no Windows.");
    }
}

#[cfg(windows)]
fn send_keys(keys: &[u16]) -> Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
    };
    if [0x10, 0x11, 0x12, 0x5B, 0x5C]
        .iter()
        .any(|key| unsafe { GetAsyncKeyState(*key) } < 0)
    {
        bail!("Larga Ctrl, Alt, Shift e Windows antes de executar o atalho.");
    }
    let make_input = |key, flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(key),
                dwFlags: if matches!(key, 0x5B | 0xAD..=0xB3) {
                    flags | KEYEVENTF_EXTENDEDKEY
                } else {
                    flags
                },
                ..Default::default()
            },
        },
    };
    let mut inputs: Vec<INPUT> = keys
        .iter()
        .map(|key| make_input(*key, Default::default()))
        .collect();
    inputs.extend(
        keys.iter()
            .rev()
            .map(|key| make_input(*key, KEYEVENTF_KEYUP)),
    );
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        // Best effort release if Windows only accepted part of the sequence.
        let releases: Vec<INPUT> = keys
            .iter()
            .rev()
            .map(|key| make_input(*key, KEYEVENTF_KEYUP))
            .collect();
        unsafe {
            SendInput(&releases, std::mem::size_of::<INPUT>() as i32);
        }
        bail!("O Windows não aceitou o atalho. Aplicações elevadas podem bloqueá-lo.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_slots_preserve_legacy_order_and_explicit_gaps() {
        let make = |name: &str, channel: ChannelName, slot| ApplicationBinding {
            name: name.into(),
            executable: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            channel,
            colour: "#30dfa7".into(),
            reset_volume: 100,
            slot,
        };
        let mut bindings = vec![
            make("one", ChannelName::Music, None),
            make("other", ChannelName::Game, None),
            make("two", ChannelName::Music, None),
        ];
        assign_application_slots(&mut bindings);
        assert_eq!(
            bindings.iter().map(|app| app.slot).collect::<Vec<_>>(),
            vec![Some(0), Some(0), Some(1)]
        );
        bindings[0].slot = Some(6);
        assert_eq!(application_page_count(&bindings, ChannelName::Music), 2);
        assert_eq!(application_page_count(&bindings, ChannelName::Chat), 0);
        let mut explicit = vec![
            make("chosen", ChannelName::Music, Some(2)),
            make("legacy", ChannelName::Music, None),
        ];
        assign_application_slots(&mut explicit);
        assert_eq!(explicit[1].slot, Some(0));
        let valid = vec![make("chosen", ChannelName::Music, Some(399))];
        assert!(validate_application_bindings(&valid).is_ok());
        let mut invalid = valid.clone();
        invalid[0].slot = Some(400);
        assert!(validate_application_bindings(&invalid).is_err());
        invalid[0].slot = Some(0);
        invalid[0].reset_volume = 101;
        assert!(validate_application_bindings(&invalid).is_err());
        let duplicate = vec![
            make("one", ChannelName::Music, Some(2)),
            make("two", ChannelName::Music, Some(2)),
        ];
        assert!(
            validate_application_bindings(&duplicate)
                .unwrap_err()
                .to_string()
                .contains("slider livre")
        );
        let legacy: ApplicationBinding = serde_json::from_value(serde_json::json!({"name":"old", "executable":"old.exe", "channel":"Music", "colour":"#ffffff", "reset_volume":100})).unwrap();
        assert_eq!(legacy.slot, None);
    }

    #[test]
    fn invalid_program_in_another_binding_does_not_block_a_valid_action() {
        let mut layer = ButtonLayer::default();
        layer.pages[0].actions[0] = ButtonAction::OpenProgram {
            path: "deleted.exe".into(),
        };
        layer.pages[0].actions[1] = ButtonAction::ToggleMute {
            fader: FaderName::C,
        };
        assert!(layer.validate().is_err());
        assert!(layer.pages[0].actions[1].validate().is_ok());
        for key in 0xAD..=0xB3 {
            assert_eq!(
                ButtonAction::MediaKey { key }.validate().is_ok(),
                cfg!(windows)
            );
        }
        for key in [0x41, 0x30, 0x70, 0x87] {
            assert_eq!(
                ButtonAction::Shortcut {
                    keys: vec![0x11, key]
                }
                .validate()
                .is_ok(),
                cfg!(windows)
            );
        }
    }

    #[test]
    fn page_colours_follow_presses_and_touch_only_button_slots_in_both_formats() {
        use goxlr_usb::colouring::ColourTargets;
        let mut layer = ButtonLayer::default();
        layer.pages[0].colours[0] = "#123456".into();
        layer.pages[0].pressed_colours[0] = "#abcdef".into();
        assert_eq!(
            layer.pages[0].lighting(EnumSet::empty()).unwrap()[0],
            [0x56, 0x34, 0x12, 0xff]
        );
        let colours = layer.pages[0]
            .lighting(EnumSet::only(Buttons::Fader1Mute))
            .unwrap();
        assert_eq!(colours[0], [0xef, 0xcd, 0xab, 0xff]);
        for format in [false, true] {
            let mut map = [0x77; 520];
            overlay_lighting(&mut map, format, &colours);
            let targets = [
                ColourTargets::Fader1Mute,
                ColourTargets::Fader2Mute,
                ColourTargets::Fader3Mute,
                ColourTargets::Fader4Mute,
                ColourTargets::Bleep,
                ColourTargets::MicrophoneMute,
            ];
            let mut modified = [false; 520];
            for (index, target) in targets.iter().enumerate() {
                for slot in 0..2 {
                    let offset = target.position(slot, format);
                    assert_eq!(&map[offset..offset + 4], &colours[index]);
                    modified[offset..offset + 4].fill(true);
                }
            }
            for (index, modified) in modified.into_iter().enumerate() {
                if !modified {
                    assert_eq!(map[index], 0x77);
                }
            }
        }
        layer.pages[0].pressed_colours[0] = "#zzzzzz".into();
        assert!(layer.validate().is_err());
    }

    fn update(
        tracker: &mut LayerTracker,
        previous: EnumSet<Buttons>,
        current: EnumSet<Buttons>,
        now: std::time::Instant,
    ) -> LayerEvents {
        tracker.update(previous, current, true, 4, now)
    }

    #[test]
    fn long_press_enters_and_exits_once_without_toggling_normal_mute() {
        let mut tracker = LayerTracker::default();
        let now = std::time::Instant::now();
        let empty = EnumSet::empty();
        let cough = EnumSet::only(Buttons::MicrophoneMute);
        assert!(update(&mut tracker, empty, cough, now).normal_down);
        assert!(
            !tracker
                .tick(true, 4, now + std::time::Duration::from_millis(699))
                .cancel_normal
        );
        assert!(
            tracker
                .tick(true, 4, now + std::time::Duration::from_millis(700))
                .cancel_normal
        );
        assert!(tracker.runtime.active);
        tracker.tick(true, 4, now + std::time::Duration::from_secs(2));
        assert!(tracker.runtime.active);
        assert!(
            !update(
                &mut tracker,
                cough,
                empty,
                now + std::time::Duration::from_secs(2)
            )
            .normal_up
        );
        let later = now + std::time::Duration::from_secs(3);
        assert!(!update(&mut tracker, empty, cough, later).normal_down);
        tracker.tick(true, 4, later + std::time::Duration::from_millis(700));
        assert!(!tracker.runtime.active);
    }

    #[test]
    fn double_and_triple_click_resolve_once_and_wrap_pages() {
        let mut tracker = LayerTracker::default();
        tracker.runtime.active = true;
        let now = std::time::Instant::now();
        let empty = EnumSet::empty();
        let cough = EnumSet::only(Buttons::MicrophoneMute);
        for index in 0..2 {
            update(
                &mut tracker,
                empty,
                cough,
                now + std::time::Duration::from_millis(index * 100),
            );
            update(
                &mut tracker,
                cough,
                empty,
                now + std::time::Duration::from_millis(index * 100 + 30),
            );
        }
        tracker.tick(true, 4, now + std::time::Duration::from_millis(479));
        assert_eq!(tracker.runtime.page, 0);
        tracker.tick(true, 4, now + std::time::Duration::from_millis(480));
        assert_eq!(tracker.runtime.page, 1);
        let later = now + std::time::Duration::from_secs(1);
        for index in 0..3 {
            update(
                &mut tracker,
                empty,
                cough,
                later + std::time::Duration::from_millis(index * 100),
            );
            update(
                &mut tracker,
                cough,
                empty,
                later + std::time::Duration::from_millis(index * 100 + 30),
            );
        }
        tracker.tick(true, 4, later + std::time::Duration::from_millis(580));
        assert_eq!(tracker.runtime.page, 0);
        tracker.clicks = 3;
        tracker.released_at = Some(later);
        tracker.tick(true, 4, later + std::time::Duration::from_secs(2));
        assert_eq!(tracker.runtime.page, 3);
    }

    #[test]
    fn held_buttons_keep_ownership_across_mode_changes_and_disabled_is_original() {
        let mut tracker = LayerTracker::default();
        let now = std::time::Instant::now();
        let empty = EnumSet::empty();
        let button = EnumSet::only(Buttons::Fader1Mute);
        assert!(
            update(&mut tracker, empty, button, now)
                .suppressed
                .is_empty()
        );
        tracker.runtime.active = true;
        assert!(update(&mut tracker, button, button, now).actions.is_empty());
        assert!(
            update(&mut tracker, button, empty, now)
                .suppressed
                .is_empty()
        );
        assert_eq!(
            update(&mut tracker, empty, button, now).actions,
            vec![(Buttons::Fader1Mute, 0)]
        );
        tracker.tick(false, 4, now);
        assert!(!tracker.runtime.active);
        assert!(
            tracker
                .update(button, empty, false, 4, now)
                .suppressed
                .contains(Buttons::Fader1Mute)
        );
        assert!(
            tracker
                .update(empty, button, false, 4, now)
                .suppressed
                .is_empty()
        );
    }

    #[test]
    fn short_click_preserves_normal_and_existing_bindings_migrate() {
        let mut tracker = LayerTracker::default();
        let now = std::time::Instant::now();
        let cough = EnumSet::only(Buttons::MicrophoneMute);
        assert!(update(&mut tracker, EnumSet::empty(), cough, now).normal_down);
        assert!(
            update(
                &mut tracker,
                cough,
                EnumSet::empty(),
                now + std::time::Duration::from_millis(150)
            )
            .normal_up
        );
        assert!(!tracker.runtime.active);
        let old: ButtonLayer = serde_json::from_str(r#"{"enabled":true,"actions":[{"kind":"toggle_mute","fader":"A"},{"kind":"none"},{"kind":"none"},{"kind":"none"},{"kind":"none"}]}"#).unwrap();
        assert_eq!(old.pages.len(), 1);
        assert!(matches!(
            old.pages[0].actions[0],
            ButtonAction::ToggleMute {
                fader: FaderName::A
            }
        ));
        assert!(old.validate().is_ok());
    }

    #[test]
    fn old_configuration_defaults_and_invalid_actions_are_rejected() {
        let config: ButtonLayer = serde_json::from_str("{}").unwrap();
        assert!(!config.enabled);
        assert!(config.validate().is_ok());
        let mut config = config;
        config.pages[0].actions[0] = ButtonAction::LoadProfile {
            profile: "../outside".into(),
        };
        assert!(config.validate().is_err());
        config.pages[0].actions[0] = ButtonAction::Shortcut {
            keys: vec![0x11, 0x11],
        };
        assert!(config.validate().is_err());
        config.pages[0].actions[0] = ButtonAction::MediaKey { key: 0 };
        assert!(config.validate().is_err());
        config.pages[0].actions[0] = ButtonAction::Shortcut {
            keys: vec![0x11, 0x10, 0x4D],
        };
        assert_eq!(config.validate().is_ok(), cfg!(windows));
        config.pages[0].actions[0] = ButtonAction::Shortcut { keys: vec![0x11] };
        assert!(config.validate().is_err());
        config.pages[0].actions[0] = ButtonAction::OpenProgram {
            path: "relative.exe".into(),
        };
        assert!(config.validate().is_err());
    }
}

#[cfg(test)]
mod soundboard_tests {
    use super::*;
    #[test]
    fn sampler_actions_round_trip_and_reject_unknown_banks() {
        for kind in ["play_sample", "stop_sample"] {
            let json = format!(r#"{{"kind":"{kind}","bank":"B","button":"TopRight"}}"#);
            let action: ButtonAction = serde_json::from_str(&json).unwrap();
            assert!(action.validate().is_ok());
            assert_eq!(serde_json::from_str::<serde_json::Value>(&json).unwrap(), serde_json::to_value(action).unwrap());
        }
        assert!(serde_json::from_str::<ButtonAction>(r#"{"kind":"play_sample","bank":"X","button":"TopRight"}"#).is_err());
    }
}
