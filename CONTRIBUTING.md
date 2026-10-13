# Contributing to SlopXLR

Use English for issues, pull requests and maintained project documentation. The application supports English and Portuguese.

## Report a bug

Use the bug-report template and include release version, Windows version, device model, reproducible steps, expected/actual result and whether the previous interface behaves differently. State whether a check used real hardware or simulated data. Review diagnostic exports for personal identifiers and paths before attaching them.

## Propose a change

Open an issue describing the concrete behavior before a large feature change. Keep a pull request scoped to one problem. Preserve upstream license notices and attribute copied third-party work. Explain any device/profile/data migration and how existing users retain their settings.

Run the relevant checks in [DEVELOPMENT.md](docs/DEVELOPMENT.md), and record which checks passed. Hardware behavior requires separate physical validation; include the device model and results when available. Avoid bundling live configurations, sample libraries, access tokens, signing keys or generated build folders.

## Translations

The authored UI catalogs live in daemon/web-content/redesign/locales/en.json and pt-PT.json. Keep their source-key sets synchronized. Translate static authored text and template fragments; do not translate IPC enums, user names, executable paths or filenames. Run the i18n and rendering tests and visually inspect long labels in both languages.

## Documentation

Update the user guide and comparison when behavior changes. Update CHANGELOG.md for release changes. Historical notes describe earlier test states and should not be cited as proof that a new feature has been physically validated.

The inherited upstream policy belongs to the upstream project. Contributions to upstream must follow that project's own requirements; SlopXLR changes are maintained independently.
