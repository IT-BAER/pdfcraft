# Built-in trusted root certificates (SMS build)

The SMS build of PdfCraft trusts these CA root certificates without the user adding them, so
signatures from these trust service providers validate as "valid" like in Acrobat Reader. The
user's own trusted certificates (Preferences) are added on top; they cannot remove these.

These files are public CA certificates: factual public-key data published by the CA. They carry no
open licence, so they are an SMS-build exception to AGENTS.md §1.2
(`LicenseRef-SMS-Trust-Anchor`, enforced by `cargo xtask assets` for `.cer` files in this folder).
They are not part of upstream PdfCraft.

| File | Certificate | Source | SHA-1 (checked against the source) |
|---|---|---|---|
| `d-trust-root-ca-5-2022.cer` | D-TRUST Root CA 5 2022 (RSA 4096), D-Trust GmbH | https://www.d-trust.net/en/support/repository | `643211332169B483B55F7046E56CBFC6C11DC5F8` |

Used by sproof sign (SMS e-signature exports, issuing CA "D-TRUST CA 5-22-2 2022").

To add a root: verify its fingerprint against the CA's own repository, add the DER file here, an
`ATTRIBUTION.toml` entry (`kind = "certificate"`, `licence = "LicenseRef-SMS-Trust-Anchor"`), and
an entry in `BUILT_IN_ROOTS` (`crates/sign/src/pdf.rs`).
