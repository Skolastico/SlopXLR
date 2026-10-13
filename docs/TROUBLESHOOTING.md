# Troubleshooting

[Documentation index](../README.md#documentation)

## The application cannot start

Close other GoXLR Utility instances through their tray menu, then open SlopXLR again. Closing a window may leave its service running. Confirm that the official GoXLR driver is installed. Keep all package executables, DLLs and helper files together.

A missing native window can indicate a missing WebView2 runtime. Use the included Microsoft bootstrapper or Microsoft's official runtime installer. The service must be running before opening the native wrapper.

## Service connected, device missing

Check the GoXLR's power/USB connection and the driver. Open Diagnostics and inspect USB failures and recovery events. Reconnect or restart the service after checking the connection. Do not run competing GoXLR applications at the same time.

A reachable local service is not proof that a physical device has been detected. Hardware-specific controls require a connected device.

## An application does not appear in Applications

Start it and play audio, then refresh the running-application list. Applications with no current Windows audio session may not appear. Bind the executable that actually produces audio, rather than a launcher. A program update that changes its executable path can invalidate a saved association.

## An association is saved, but there is no control

Check its channel and route status, whether the program is running with an audio session, and whether the changes were saved. Enter Applications mode using the physical fader assigned to that channel. Lower all four faders for preparation, then move each through the displayed volume to complete pickup.

Windows session volume and routing can differ for programs using a fixed output or exclusive audio path. An executable binding does not provide independent control of every browser tab or window.

## Faders seem inactive after entering/changing pages

Follow the preparation and pickup messages. Dim LEDs indicate waiting; the service deliberately avoids overwriting an application's volume with an unrelated physical fader position. Once acquired, movement controls volume. Reference volumes apply when you invoke reset.

## Button actions do nothing

Confirm that custom navigation is enabled, the changes are saved and the header shows Actions mode. A–D and Bleep use the current action page; Cough is navigation. Applications mode and Actions mode cannot be active together.

Try Test action for the selected binding. Check executable paths, existing profile names and the selected soundboard bank/slot. Test action executes immediately and does not save a draft. A physical-gesture check is still required after a successful UI test.

## A shortcut or media key has no visible result

Focus the intended program and try again. A standard-permission service may be unable to control an elevated application. Media commands follow Windows' selected media session. Avoid holding modifiers while testing a recorded shortcut. Check the action result in the UI.

## Soundboard preview works, but the stream hears nothing

Preview uses the browser/WebView's default output. Actual Mini soundboard playback uses System on the Mini. Check System's level/mute and its route to the intended output in Routing. Confirm that the clip is assigned to the bank/slot used by the Play action. Save the device profile and Controls changes separately.

## A setting disappears after reopening

Consult the [save table](USER_GUIDE.md#1-device-profiles-and-saving). Device and microphone profiles, Controls, Applications and lifecycle lists are saved separately. Frontend-local EQ snapshots and language preferences belong to that browser origin/profile.

## Update check reports no public releases

That is expected while SlopXLR is private. Open the GitHub release link, sign in with an authorized account, and install manually. The application does not bundle private-repository credentials.

For a signature/hash error, keep the existing installation and use the release's files/checksums to investigate. A missing or mismatched signing key is not a condition to bypass.

## Recovering after an upgrade

Stop the service. Keep a copy of the failed installation and its data. Restore the old program package with the matching configuration/profile backup, then reopen Open.cmd. The updater's backup directory is shown in its result and is normally beneath update-backups in the data root.

Missing default profiles/icons/presets can be recovered from Settings → Recover default files; this preserves existing files. It does not restore your own deleted profiles or samples.

## Reporting a problem

Include the release version, Windows version, GoXLR model, steps, expected/actual result and whether the original interface behaves differently. Review diagnostic exports before attaching them. Remove personal paths, device identifiers and unrelated application/profile names. Never attach signing keys or access tokens.

Use the [bug-report template](../.github/ISSUE_TEMPLATE/bug_report.md). Known validation gaps are listed in [VALIDATION.md](VALIDATION.md).
