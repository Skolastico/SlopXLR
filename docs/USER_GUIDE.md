# SlopXLR user guide

[Documentation index](../README.md#documentation) · [Installation](INSTALLATION.md) · [Troubleshooting](TROUBLESHOOTING.md)

The names below refer to the English interface. Change the language under Settings → Language.

## 1. Device, profiles and saving

Select the connected device and device profile at the top of the window. The header shows whether the Mini is in Normal, Actions or Applications mode. Confirm the active mode before using the hardware controls.

Different controls have different save behavior:

| Change | How to keep it |
| --- | --- |
| Channel volumes, routing, normal lighting and sampler assignments | Save the device profile. |
| Microphone gain, EQ, gate, compressor, processing and saved display preferences | Save the microphone profile. |
| Custom Controls pages and actions | Save changes in Controls. |
| Application bindings, positions, colors and reference volumes | Save changes in Applications. |
| Shutdown, sleep and resume action lists | Save each list in Settings. |
| Service preferences | Saved automatically by the service. |
| UI language and EQ A/B snapshots | Stored locally by the frontend. |

Editing an action or application association creates a draft. Use its Save control to apply it, or Reset changes to discard the draft. A successful action test does not save an edited binding. Profile loading changes real device state; it is not a preview.

## 2. Mixer

Adjust channel and output volumes, toggle mute and configure supported submix controls. Volumes are represented by the device's real range. Available channels and features follow the connected model.

Use Controls → Faders for channel assignments and physical mute behavior. A channel mute is separate from muting an individual Windows application. Save the device profile to retain adjustments.

## 3. Microphone

Choose the microphone type that matches your connection, then set gain while speaking normally. The input meter and history show measured input before processing; the dashed line marks the configured gate threshold.

Use the equalizer's simple controls for broad adjustments, or the detailed controls for individual bands and frequencies. Mini and full-size GoXLR have different bands. The graph illustrates band gains; it is not an acoustic frequency-response measurement.

Gate controls determine when the microphone is attenuated. Compressor controls reduce level differences; makeup gain compensates after compression. Detailed views expose the supported timing and ratio options. Other supported microphone processing is available further down the page.

### EQ A/B comparison

1. Adjust the EQ and choose Save A.
2. Make a second EQ setting and choose Save B.
3. Enable Compare A/B to display the stored curves.
4. Choose Apply A or Apply B to send that EQ to the real device.
5. Save the microphone profile if you want the applied EQ to survive service restart.

Snapshots are local to this frontend and device, rather than portable microphone profiles. Comparing curves alone does not apply them. Reset EQ changes real EQ settings.

## 4. Controls: normal hardware controls

The Faders area sets each physical fader's assigned channel and supported mute options. In Normal mode the Mini's A–D buttons retain channel mute behavior, Bleep retains its normal function, and Cough performs temporary microphone mute.

When custom action navigation is enabled, the long Cough gesture is reserved for entering or leaving Actions mode. Disable custom navigation to retain the original long-press behavior.

## 5. Controls: custom Mini button pages

Custom actions are supported on the GoXLR Mini. Each named page has five assignments: A, B, C, D and Bleep. Cough is used for navigation. Pages can also define resting and pressed colors for the six button positions.

1. Open Controls and the action-page editor.
2. Add or select a page and give it a name.
3. Select a button and choose an action.
4. Fill in its required fields. Optionally use Test action, which executes the action immediately.
5. Save changes and enable custom navigation.

| Action | Setup and behavior |
| --- | --- |
| No action | Leaves that button without a custom action on this page. |
| Open application | Provide an existing absolute .exe path. The service launches the program without a shell command. |
| Media control | Choose play/pause, next, previous, stop or a Windows audio-volume command. The responding application depends on Windows' media handling. |
| Keyboard shortcut | Record the required key combination. The focused application receives it; Windows may restrict interaction with elevated applications. |
| Load profile | Choose an existing device profile. Pressing the button changes real device configuration. |
| Toggle fader mute | Choose A–D. The action uses that fader's current assigned channel. |
| Play soundboard audio | Choose a virtual sampler bank and slot; playback uses the slot's configured clips and playback order. |
| Stop soundboard audio | Choose the bank and slot whose playback should stop. |

### Hardware gestures in Actions mode

| Gesture | Result |
| --- | --- |
| Hold Cough for about 700 ms | Enter or leave Actions mode. |
| Press A–D or Bleep | Run that button's action on the active page. |
| Double-click Cough | Next action page. |
| Triple-click Cough | Previous action page. |
| Single-click Cough in Actions mode | No assigned custom action. |

Page navigation wraps. The service waits about 350 ms after the final click release to distinguish double from triple clicks. Button ownership is retained across a mode transition until release. The underlying profile's lighting returns when leaving the custom mode.

Actions and Applications modes are mutually exclusive. The service runs actions; closing only the window does not disable them.

### Example: a two-button soundboard page

Configure a clip in Sampler bank A, top-left slot. In Controls, assign button A to Play soundboard audio for that bank/slot and button B to Stop soundboard audio for the same destination. Save the device profile and the Controls changes. Hold Cough to enter Actions mode, then press A to play and B to stop.

## 6. Applications: Windows per-application audio

This mode controls Windows audio sessions using the Mini's four faders. It is separate from the hardware channel Mixer.

1. Start the target application and make it play audio so Windows exposes a session.
2. Open Applications, select a supported channel and choose Add application.
3. Select the running executable, a display name, page, position A–D, color and reference volume.
4. Save the association changes.
5. Check the route status shown for each association. Saved channel routing and available sessions affect whether it can be prepared.

Bindings identify executable paths. Several processes/sessions of the same executable may be affected together; this is not a separate slider for every window or browser tab. Application restarts or a changed executable path can require reassociation. Applications that follow their own fixed/exclusive audio path may not respond to session routing as expected.

Each channel has four positions per page, up to 100 pages in the editor. Moving an existing binding into an occupied position swaps the two complete positions in the draft. Save to apply the exchange.

### Entering and using Applications mode

1. In Normal mode, double-click the mute button of the fader assigned to the channel with saved application bindings.
2. Confirm Applications mode in the header.
3. Lower all four physical faders to the bottom to complete preparation.
4. Raise each fader to the displayed application volume until it acquires control. This pickup step avoids a sudden volume jump.
5. Use A–D for application volume and mute on the current page.
6. Bleep moves to the previous page; Cough moves to the next page.
7. Double-click the mute button used to enter the mode to leave it.

Hold Bleep and Cough together to reset the channel's associated applications to their saved reference volumes. Reference volume is a reset target, not a continuous override.

The fader LEDs indicate the application volume; dim lighting indicates waiting/preparation and an unlit muted application is silent. These LEDs do not measure instantaneous audio loudness. On page changes, follow pickup indicators before expecting a fader to change volume.

### Example: Music applications

Associate a music player with Music, page 1, fader A and a browser with Music, page 1, fader B. Save, make both applications produce audio, and check their route status. Double-click the physical mute button currently assigned to Music, lower the faders, then pick up each displayed volume. Adjust or mute the two applications independently and exit using the original entry button.

## 7. Sampler and the Mini soundboard

The Mini has a software soundboard with virtual banks A–C and four slots per bank. This does not add physical recording hardware or full-size GoXLR voice effects to the Mini.

1. Open Profiles and files to locate the Samples library folder; copy your audio files into it.
2. Open Sampler, browse the library and select a bank/slot.
3. Add one or more clips to that slot and choose the supported playback behavior and order.
4. Use playback/stop controls to check the slot.
5. Save the device profile to keep the assignments.
6. Bind Play/Stop actions in Controls to use the Mini's physical buttons.

Multiple clips can use sequential or random order. Use the waveform/trim controls to set start and end percentages for an assigned clip. Trimming does not edit the original file. The start must remain before the end. Save the device profile after changes.

Library preview plays through the browser/WebView's default audio output. Actual Mini soundboard playback uses the service's audio engine and the Mini's System output, following your routing matrix. A working preview therefore does not prove that the stream routing is correct.

On a full-size GoXLR, the existing hardware sampler bank, playback and configuration controls remain. Hardware recording and full-size sampler behavior still require physical validation in SlopXLR.

## 8. Routing

Rows are audio inputs and columns are outputs. A checked cell connects the corresponding input to that output. Changes apply to real audio routing immediately; save the device profile to retain them.

Configure headphones/monitoring and stream or chat destinations according to your setup. For the Mini soundboard, route System to the destination that should hear the clip. Monitor selection and available Stream 2 / recording options are exposed where supported.

## 9. Lighting

Edit the normal profile's colors and supported display/off styles. You can apply a fader's current colors/style to all faders and apply supported colors/off states to button groups. These controls use the last confirmed device state.

Actions and Applications modes can temporarily overlay lighting as feedback without overwriting the normal profile colors. Full-size GoXLR exposes additional lighting and display controls where supported. Save the device profile to keep normal lighting changes.

## 10. Effects: full-size GoXLR

The Effects area appears for supported full-size hardware. Choose a preset and adjust supported voice-effect parameters. Reverb decay uses only values representable by the service: 10 ms steps through 1000 ms, then 100 ms steps through 20000 ms.

These features are inherited hardware capabilities and have not been tested on a physical full-size GoXLR in this fork. They are not available on the Mini.

## 11. Profiles and files

Create a new device or microphone profile, save current settings as a new name, load existing profiles, load profile colors only, or remove a profile. Choose the appropriate device/microphone save control.

Folder controls open the relevant profile, preset, icon and sample libraries for file management. Import/export uses those files and folders; SlopXLR is not a cloud synchronization service. Keep backups before replacing profile files manually.

## 12. Settings

- **Language:** English or Portuguese; save drafts before switching.
- **Application updates:** check public releases or open GitHub. Private releases currently require manual installation.
- **Advanced options:** open the Previous interface.
- **Device behavior:** supported mute duration, stream/recording and related device options.
- **Automatic actions:** ordered shutdown, sleep and resume lists. Add, reorder or remove entries and save each list. Existing custom commands are preserved.
- **Application preferences:** Windows startup, tray icon, opening the window, service log level, service-message language and activator path. Changes are saved automatically.
- **Recover default files:** restore missing default profiles, microphone profiles, effect presets or icons without overwriting existing files or loading a profile.
- **Quit application:** stops the service and runs its configured shutdown actions. Reopen SlopXLR to resume control.
- **Firmware:** inherited online/local-file update tools. Firmware updates interrupt audio and require a continuously connected device. Package updates and firmware updates are separate operations.

## 13. Diagnostics

Review connection status, recent events and USB recovery information. The complete diagnostics page can refresh and export the service's diagnostic information. Raw messages may retain their original language. Review exported content before posting it, since it can contain device identifiers, paths and profile names.

Use the [troubleshooting guide](TROUBLESHOOTING.md) when a saved configuration does not behave as expected. Physical button gestures, audible output and a full Windows reboot still need user verification; automated tests do not substitute for those checks.
