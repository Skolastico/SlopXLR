# Installation and migration

[Documentation index](../README.md#documentation)

## Requirements

- Windows x64; other platforms inherited from upstream are not packaged or validated for SlopXLR.
- A GoXLR Mini for the custom hardware modes. Full-size GoXLR support is experimental in this fork.
- The official GoXLR audio driver, installed before running SlopXLR. The application does not replace that driver.
- Microsoft Edge WebView2 Runtime for the native window. The installer includes Microsoft's official bootstrapper, which may need Internet access.
- A writable destination for the portable package.

Close the original Utility and any other SlopXLR instance before installing or starting. Both services use the same local device connection and service port.

## Installer

1. Download the Windows setup executable from the chosen release.
2. Run it under the Windows account that will use SlopXLR.
3. Choose the installation folder. The default remains %LOCALAPPDATA%/Programs/GoXLR-Personalizado for compatibility with the first beta.
4. Optionally select Windows startup and a desktop shortcut.
5. Open SlopXLR from the Start menu or shortcut.

The installer uses per-user installation and registration. The official audio driver's installation requirements are separate. It checks for a running Utility and for the driver. It installs WebView2 if the runtime is missing.

Installed settings and libraries are stored in %APPDATA%/Skolastico/GoXLR-Personalizado. The legacy folder name is retained across the SlopXLR rename so existing data is found.

## Portable package

1. Download and extract the entire portable ZIP into a writable folder.
2. Open Open.cmd. Keep the helper scripts, DLLs and executables together.
3. User data is created in the user-data folder next to Open.cmd.

An existing customized portable folder with config/settings.json continues to use that configuration and its configured libraries. Copying only the daemon executable is not a complete portable installation.

## Import existing settings

On the first interactive launch, SlopXLR offers to import the original GoXLR Utility configuration if it is found in its standard Windows location. Choose Yes to copy it, No to create a fresh configuration, or Cancel to stop.

Import copies settings, device profiles, microphone profiles, effect presets, icons and samples. It rewrites the copied library paths to the destination and keeps an imported settings backup. It leaves the source configuration and libraries intact. Logs and old backups are not imported.

To import a customized folder explicitly, open PowerShell in the SlopXLR program folder and run:

~~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\Start.ps1 -ImportFrom "C:\path\to\config\settings.json"
~~~~

Import only initializes a destination that does not already contain config/settings.json. An existing destination is kept as-is. Back up any existing destination before choosing a fresh location.

## Updates

### Current private repository

Sign in to GitHub with an account that has access, download the newer release, close the service through Settings or its tray menu, and run the newer installer in the same program folder.

For portable copies, extract the newer package separately, then replace program files in the existing folder while the service is stopped. Keep user-data, config, data and backup folders. Do not replace settings or libraries with files from another user's installation.

The in-app update checker does not contain a GitHub token and cannot download private releases. Its GitHub link opens the release page for manual access.

### When public releases are available

Settings → Application updates can check public releases. Installing requires confirmation and saved drafts. The helper verifies the pinned RSA signature, signed release metadata and package SHA-256 before stopping the service. It backs up program files, configuration and profiles, starts the replacement service and attempts rollback on failure. Samples stay in their existing library.

A real update between two published versions has not yet been validated end to end. Review [validation](VALIDATION.md) before relying on this path.

## Backups and recovery

Back up config and the library folders shown under Profiles and files. The default data layout contains data/profiles, data/mic-profiles, data/presets, data/icons, data/samples and data/backups. Library locations can also be customized in the service settings.

The update helper creates update-backups beneath the data root. Keep the old program package as well when making a manual upgrade. For a failed upgrade, stop the service, restore the old program files and matching backed-up settings/profiles, and open Open.cmd.

## Uninstall

Stop the service, then use Windows' installed-apps list or Uninstall.exe in the program folder. Uninstall removes managed program files and shortcuts. It preserves settings, libraries and samples, including installed data under AppData. Delete personal data separately only if you intend to discard it.

Reinstalling to the same location uses the retained configuration. Changing the program location does not move custom library paths automatically.

## English / Portuguese

Use Settings → Language. The selection is saved in the frontend's local storage, so different browser origins or profiles can have different selections. The initial language follows the browser/WebView locale. Save or reset edited actions, application bindings and lifecycle lists before changing language.

Profile names, executable paths, filenames and raw diagnostic messages are preserved. The service message language is a separate setting.
