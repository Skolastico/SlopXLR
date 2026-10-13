# Changelog

## 0.1.0-beta.2 — SlopXLR identity and documentation

- Name the application, native window, installer and newly built packages SlopXLR.
- Compile the native window from its included source instead of reusing the first beta's window binary.
- Make English the primary GitHub documentation language while retaining English/Portuguese UI choices.
- Add installation/migration, complete user, upstream comparison, troubleshooting, build/release and validation guides.
- Add contribution and security guidance, a relevant roadmap and issue/pull-request templates.
- Preserve upstream credits/licenses and move dated Portuguese development audits into a clearly marked history folder.
- Retain first-beta data/installation identifiers and migrate managed shortcut names during installer upgrades.

## 0.1.0-beta.1 — First packaged customized beta

Originally distributed as GoXLR Personalizado.

- New main interface with standard device controls, profiles, diagnostics and recovery of missing defaults.
- Configurable GoXLR Mini action pages and Windows application audio modes.
- Mini virtual soundboard with clip order, waveform preview, trimming and Play/Stop actions.
- Microphone signal/history views, simple/detailed controls and local EQ A/B comparison.
- English/Portuguese UI catalog and Previous interface under Settings → Advanced options.
- Mini first-fader-read preservation and profile/device recovery work.
- Windows per-user installer, portable launcher, copied settings import and retained user data on uninstall.
- RSA-signed update manifest and package hash verification for future public updates.
- Private GitHub source and prerelease distribution. Online automatic updates remain unavailable for private releases.

Validation: 33 frontend tests, 16 service tests, updater self-tests and isolated install/reinstall/import/uninstall preservation checks passed. Full-size hardware, a clean second PC and version-to-version automatic updates remain unvalidated.
