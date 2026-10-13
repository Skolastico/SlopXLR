# Build and release guide

[Documentation index](../README.md#documentation) · [Contributing](../CONTRIBUTING.md)

## Source layout

| Path | Purpose |
| --- | --- |
| daemon/src | Device service, HTTP server, profiles, button tracking, Windows application audio and update entry points. |
| daemon/web-content/redesign | Main frontend modules and authored English/Portuguese catalogs. |
| daemon/web-content | Original bundled frontend, diagnostics and standalone compatibility editors. |
| ipc | Shared command/status types. |
| usb | Device protocol, driver integration and USB notification/fader handling. |
| profile | Device/microphone profile formats and supported values. |
| defaults | Default profiles, microphone profiles, presets and icons. |
| ui-wrapper/src-tauri | Native Tauri window and IPC integration. |
| distribution | Version metadata, packaging script, launcher, installer and signed update helper. |
| tests | Frontend contracts, rendering, draft handling, transport and translation checks. |

The service controls the hardware and custom modes. The frontend edits state and sends existing IPC commands. The native window hosts the embedded frontend. Closing the window does not automatically stop the service.

## Windows build prerequisites

Install Rust with the x86_64 MSVC toolchain, Visual Studio C++ Build Tools / Windows SDK, Node.js 22 or later for frontend tests, and NSIS for packaging. Open a terminal configured for the x64 MSVC compiler. The official GoXLR driver is needed for physical-device use, rather than source compilation alone.

The frontend is embedded in the daemon; editing it requires rebuilding that daemon for a packaged release. The inherited original frontend is already bundled. This fork's main frontend does not need a separate npm build step.

~~~~powershell
git clone https://github.com/Skolastico/SlopXLR.git
cd SlopXLR
cargo build --release --locked -p goxlr-daemon -p goxlr-defaults
cargo build --manifest-path ui-wrapper/src-tauri/Cargo.toml --target-dir target --release --locked
~~~~

Each crate retains inherited internal names and versions. The distribution version is defined by distribution/release.json. Binary names such as goxlr-daemon.exe are retained for IPC/process compatibility.

## Tests

~~~~powershell
node --test tests/*.test.mjs
cargo test --locked -p goxlr-daemon
cargo test --locked -p goxlr-ipc --test redesign_contract
powershell.exe -NoProfile -ExecutionPolicy Bypass -File distribution/windows/Update.ps1 -Mode SelfTest
~~~~

If a restricted environment blocks Node child-process test isolation, Node 22 supports --experimental-test-isolation=none. Use the flag supported by your installed Node version. Normal terminals can use the default test command above.

Frontend tests check real IPC serialization, model capabilities, routes, representable parameters, escaping, drafts, page positions, save behavior, transport updates and translations. Service tests cover button timing/ownership, Windows routing contracts, profile recovery, sample paths and Mini behavior. Updater self-tests check version ordering, signature verification/tampering and download-host validation. Passing tests do not replace physical-device testing.

A local frontend-only development server is available in redesign-dev.py. Read its routes before using it: it is a development helper, not the release launcher or a complete update endpoint. Development preview data and a real connected device require different validation claims.

## Package a release

Update distribution/release.json and the frontend's displayed distribution version together. Update CHANGELOG.md and package documentation before building.

Keep the RSA private key outside the repository. It must match distribution/update-public.xml. Do not replace the public key during a routine release: existing installations trust that key.

~~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File distribution/Build.ps1 -SigningKeyPath "C:\secure\update-signing.key.xml" -NsisPath "C:\Program Files (x86)\NSIS\makensis.exe"
~~~~

Build.ps1 compiles the service, defaults and native wrapper using their lockfiles. It stages program files only, verifies Microsoft's WebView2 bootstrapper signature, builds the per-user installer and portable ZIP, and writes signed update metadata and checksums.

Optional parameters:

| Parameter | Use |
| --- | --- |
| -SkipBuild | Package previously compiled binaries. Build them first. |
| -UiBinary | Choose an existing native wrapper binary. Record its origin in release notes if reused. |
| -NsisPath | Provide makensis.exe explicitly. |
| -WebViewBootstrap | Use an already downloaded official Microsoft bootstrapper; its signature is still checked. |
| -SigningKeyPath | Required private RSA XML key matching the committed public key. |

Do not package an active installation's config, profiles, samples, logs, tokens or private signing key. The package staging step intentionally selects managed program files.

## GitHub workflow

The Windows release workflow is manually dispatched. It checks out the selected ref, validates tests/version, installs NSIS, builds both binaries and uploads the five release artifacts. Configure UPDATE_SIGNING_KEY_XML as a repository Actions secret with the matching private XML key before running the packaging job. Without it the workflow fails rather than emitting unsigned updates.

A manual dispatch on a branch produces a build artifact. A dispatch on a version tag can also create its prerelease. The tag must equal v followed by distribution/release.json's version. The workflow has not yet been run with the production signing secret in this repository.

To publish a release manually:

1. Build and validate the package, update docs, and commit the exact release source.
2. Create/push a tag such as v0.1.0-beta.2 on that commit.
3. Create a GitHub prerelease from that tag with English release notes.
4. Upload the setup executable, portable ZIP, update.json, update.json.sig and SHA256SUMS.txt.
5. Compare GitHub's uploaded asset digests with the local checksums.
6. Test the installed/portable package and preserve a rollback package before inviting other users to upgrade.

The repository's current private visibility means automatic public-download updates are unavailable. Do not embed an account token into the app to work around that.

## Signing and compatibility

Update.json binds version, repository, URLs and SHA-256 hashes. Update.json.sig is an RSA/SHA-256 signature verified using the app's pinned public key. The updater allows GitHub release download URLs for this repository and checks package hashes before stopping the service.

This does not provide Windows Authenticode trust for setup or executables. Commercial code signing is a separate future distribution step. Installer/data paths retain first-beta identifiers so upgrades find existing settings. Keep those compatibility identifiers stable unless implementing a tested migration.

The public updater's version-to-version installation and rollback path still require end-to-end testing. See [validation](VALIDATION.md) for the release check scope.
