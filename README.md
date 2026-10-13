# SlopXLR

**The name says it all.** A Windows desktop variant of [GoXLR Utility](https://github.com/GoXLR-on-Linux/goxlr-utility), with a redesigned interface and additional GoXLR Mini controls.

SlopXLR builds on the original Utility's device service, audio configuration and profile system. It adds configurable button pages, per-application volume control, a software soundboard for the Mini, and English / Portuguese UI support. It also includes a per-user installer and a portable package.

This is an independent derivative maintained by [Skolastico](https://github.com/Skolastico). It is not an official TC Helicon product or an upstream GoXLR Utility release. Original authorship and licenses are retained.

> **Beta:** Windows x64 and GoXLR Mini are the current distribution target. Standard full-size GoXLR features remain in the code, but have not been physically validated in this fork. The custom Controls and Applications modes are Mini-only. See [validation and limitations](docs/VALIDATION.md).

## Get started

1. Install the official GoXLR audio driver and close other GoXLR Utility instances.
2. Download the installer or portable ZIP from [Releases](https://github.com/Skolastico/SlopXLR/releases).
3. Run the installer, or extract the portable ZIP to a writable folder and open **Open.cmd**.
4. Import an existing Utility configuration when offered, or start fresh.
5. Select your device and profile. Set the UI language in **Settings → Language**.

This repository is currently private. Downloading releases requires an account with access. Application updates are installed manually while releases remain private.

## What SlopXLR adds

| Feature | What you can do |
| --- | --- |
| New main interface | Use dedicated Mixer, Microphone, Controls, Applications, Routing, Lighting, Sampler, Profiles and Settings pages. |
| Mini button pages | Assign programs, media keys, keyboard shortcuts, profiles, fader mute and soundboard actions to A–D and Bleep. |
| Application mode | Control individual Windows applications with the Mini's four faders, with pages, mute, reference volumes and pickup. |
| Mini soundboard | Use three virtual banks of four slots, multiple clips per slot, sequential/random playback and non-destructive trimming. |
| Microphone workflow | Use a measured input signal, gate history, simple/detailed controls and local EQ A/B snapshots. |
| English and Portuguese | Choose the interface language without renaming your profiles, programs or files. |
| Distribution | Install per user or run a portable copy; import settings; retain user data when reinstalling or uninstalling. |
| Update verification | Verify signed update metadata and package hashes before replacing program files. |

Core channel mixing, microphone processing, routing, profiles, firmware tools and model-specific hardware features are inherited from GoXLR Utility. See the [comparison](docs/COMPARISON.md) for the scope of each addition.

## Documentation

| Guide | Contents |
| --- | --- |
| [Installation and migration](docs/INSTALLATION.md) | Requirements, installer, portable mode, importing, upgrading, data locations and uninstalling. |
| [User guide](docs/USER_GUIDE.md) | Every application area, hardware gestures, worked examples and what to save. |
| [Comparison with GoXLR Utility](docs/COMPARISON.md) | Inherited behavior, additions, interface changes and model differences. |
| [Troubleshooting](docs/TROUBLESHOOTING.md) | Missing device, audio routing, applications, pickup, soundboard, shortcuts and recovery. |
| [Build and release guide](docs/DEVELOPMENT.md) | Source layout, builds, tests, packaging, release signing and GitHub workflow. |
| [Validation and limitations](docs/VALIDATION.md) | Confirmed checks and outstanding physical / clean-machine tests. |
| [Changelog](CHANGELOG.md) | Changes by release. |
| [Roadmap](ROADMAP.md) | Remaining validation and development priorities. |
| [Contributing](CONTRIBUTING.md) | Bug reports, changes and translation contributions. |
| [Security](SECURITY.md) | Local access, release verification and reporting. |

The original interface is available under **Settings → Advanced options → Previous interface**. Button actions and application control are handled by the service and can continue after closing the window; exiting the service stops them.

## Credits and licenses

- Device service and original frontend: [GoXLR-on-Linux / GoXLR Utility](https://github.com/GoXLR-on-Linux/goxlr-utility), initially based on commit 49ab4e4.
- Native window source: [FrostyCoolSlug / goxlr-utility-ui](https://github.com/FrostyCoolSlug/goxlr-utility-ui), initially based on commit e71bb2e, included in ui-wrapper/.
- SlopXLR customization and distribution: Skolastico.

The repository retains the upstream Git history. See [LICENSE](LICENSE), [third-party licenses](LICENSE-3RD-PARTY), [wrapper license](ui-wrapper/LICENSE) and [the original README](README-UPSTREAM.md). The release's signed update manifest is separate from Windows executable code signing.
