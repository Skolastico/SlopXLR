# Validation and current limitations

[Documentation index](../README.md#documentation)

## Baseline verification

The first distributed beta passed 33 frontend tests and 16 service tests. These checks include command contracts, model gating, action recognition, held-button ownership, application positions, escaping, transport updates, profile loading recovery and Mini startup-volume preservation. Update self-tests rejected tampered metadata and lookalike download hosts.

A workspace-only installer test verified installation, reinstallation with retained settings/samples, copied import with an unchanged source, and uninstallation with retained user data. That test uses the installer's SANDBOX option: it does not validate actual Start-menu/startup registration, driver preflight or fresh WebView2 installation on a clean PC.

A real connected Mini was queried and the application service was restarted with before/after comparisons of device profiles, microphone settings, volumes, mutes, routing, lighting and sampler state. Those values were retained. Temporary WAV playback/stop, trim and sample-path behavior were checked in earlier connected-Mini work. Testing did not replace the user's saved button assignments.

## Still pending

- A clean second Windows machine, including driver/WebView2 prerequisites and normal installer/uninstaller shortcuts and registry behavior.
- A complete Windows reboot and verification of the chosen startup configuration.
- A full-size GoXLR: effects, physical recording/sampler, scribble displays and lighting.
- User-operated physical gestures for custom modes, page navigation, immediate mute and fader pickup, with audible confirmation.
- A genuine automatic update between two public signed releases, including installation failure and rollback.
- A production GitHub Actions run with the repository signing secret configured.
- Windows executable code signing.

## How to run an acceptance check

Back up your profiles/configuration and note the device/microphone profile names first. Use low listening levels for the soundboard check.

1. Launch the chosen installer or portable package and confirm device detection.
2. Select English and Portuguese, reopen the frontend, and confirm names/paths remain unchanged.
3. Save an action page and enter/leave Actions mode with Cough. Check next/previous page gestures.
4. Configure Play/Stop for one temporary soundboard clip, save both profile and action bindings, and listen for the intended routed output.
5. Associate two running applications, enter Applications mode, prepare/pick up the faders and check separate volume/mute.
6. Leave custom modes and confirm normal mute behavior.
7. Close only the window; check that service-driven controls remain available. Stop and reopen the service; verify saved profiles/settings and volumes.
8. On a spare/clean machine, check normal installation, startup, upgrade and uninstall while confirming retained data.

Record what was actually exercised, whether hardware or simulated, and any original configuration you restored. Do not describe interface exposure or unit tests as physical validation.
