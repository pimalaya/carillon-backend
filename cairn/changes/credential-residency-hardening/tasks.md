---
cairn: tasks
change: credential-residency-hardening
---

- [x] Change `Credential::Password(String)` to hold the encrypted form; give the watcher task an `Arc<Crypto>` and decrypt just-in-time at each `connect` (`supervisor.rs`)
- [x] Decrypt into `secrecy::SecretString` (zeroize-on-drop), expose only for the `ImapLogin::new` / WebDAV Basic call, drop immediately after auth; re-decrypt on reconnect (IMAP + CardDAV paths)
- [x] Apply the same zeroizing-secret discipline to the OAuth refresh/access-token plaintext in `mint_access_token`
- [x] Audit the credential types' `Debug`/`Display` and every log site so no decrypted secret is ever logged, formatted, or stored
- [x] Disable coredumps (`LimitCORE=0`) and keep secrets out of swap (swapless or `LimitMEMLOCK` + page lock) in the NixOS module and the host baseline
- [x] Verify: per-connect decrypt (not once-at-spawn); plaintext absent from a mid-watch core dump; reconnect works across a dropped connection
- [x] Fold the delta into the spec ([[hardening]], [[architecture]], [[nixos]]) and add the log entry
