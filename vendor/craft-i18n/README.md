# Vendored craft-i18n catalogs (SMS build)

The SMS build embeds its interface translations from this folder instead of a
[storytold/craft-i18n](https://github.com/storytold/craft-i18n) checkout. `.cargo/config.toml` sets
`CRAFT_I18N_DIR` to it, and `crates/ui-egui/build.rs` reads `locales/<code>/pdfcraft.tsv` and
`locales/<code>/common.tsv` as it would from a checkout.

- Source: craft-i18n at `866aff71dd63d8369eb100b9c432b5af1400c22a` (the rev in `craft-i18n.rev`),
  exported byte for byte with `git show` (no line-ending conversion).
- Locales: every language in `LANGUAGES` (`crates/ui-egui/src/i18n.rs`), the set the upstream
  i18n tests expect to be embedded.
- Only `de` differs from upstream: the SMS edits (display name "PDF Reader", informal "du"
  throughout, rows for SMS-only features).
- Every file has an `[[asset]]` entry with its SHA-256 in `ATTRIBUTION.toml`, between the
  `BEGIN/END SMS vendored craft-i18n catalogs` markers. Update the `sha256` after each edit.
- Licence: MIT OR Apache-2.0 ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).
  Translators are listed in craft-i18n's
  [ATTRIBUTION.md](https://github.com/storytold/craft-i18n/blob/866aff71dd63d8369eb100b9c432b5af1400c22a/ATTRIBUTION.md).
