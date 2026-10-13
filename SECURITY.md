# Security and release verification

SlopXLR is a local device-control application. The custom button/application/update endpoints require a local peer and loopback host, and validate Origin when present. Keep local-network access disabled unless you understand and need the inherited service setting.

Program actions accept existing absolute executable paths and are launched without a shell. Keyboard actions still depend on Windows permissions and the active target. User-entered frontend text is escaped; sample access is constrained to the configured library.

The update helper verifies RSA/SHA-256 signed metadata with a pinned public key and checks package SHA-256 before replacement. It accepts this repository's HTTPS GitHub release URLs. A signature/hash failure must not be bypassed. This is separate from Windows Authenticode signing, which is not currently provided.

Private releases require authenticated manual downloads. No GitHub access token is distributed in the program. Keep UPDATE_SIGNING_KEY_XML in repository Actions secrets and private key files outside Git. Losing the matching signing key requires a deliberate trust migration for existing installations.

Review diagnostic exports before sharing; they can contain local paths, device identifiers and profile names. Report a suspected vulnerability to the repository owner using GitHub private vulnerability reporting if available, or ask for a private reporting channel without publishing exploit details or credentials in an issue.
