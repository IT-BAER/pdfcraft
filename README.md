# PdfCraft, SMS build

Internal Windows build of PdfCraft for SMS group clients: a plain PDF reader that replaces
Adobe Acrobat Reader. It is based on
[PdfCraft](https://github.com/storytold/pdfcraft) by the ArtCraft team. It is not an ArtCraft
product and is not made, endorsed or supported by the ArtCraft team.

Branch `sms-build` holds the SMS commits on top of an upstream release tag (currently v0.14.0).
Branch `main` mirrors upstream unchanged.

## Changes against upstream

- ArtCraft name, logos and community links removed (upstream [`docs/brand/LICENSE-brand.txt`](https://github.com/storytold/pdfcraft/blob/main/docs/brand/LICENSE-brand.txt) requires this for
  modified versions): no Discord button, no community card on the home screen, plain-text credit in
  the About dialog.
- Reader profile (`crates/engine/src/reader.rs`, switched on by the desktop app): menus, palette,
  All tools, Home, tool panels, context menus and shortcuts offer only reader features: view,
  search, print, save, comments and stamps, Fill & Sign, form filling, measure, digital signing and
  validation, and the page tools rotate, delete, extract, insert and combine. Everything else
  (create, export, edit, OCR, redact, protect, optimize, prepare form, accessibility, JavaScript,
  AI...) stays in the code but is hidden and refused, so upstream rebases stay small. Unit and
  kittest tests keep the profile off by default, so the upstream test suite runs unchanged.
- No external links or online info: no Help web links, no update check (the HTTP client is gone),
  no Contributors/Models tabs in About, no tagline or privacy card on Home, no URLs in the CLI
  version text, the MCP server info, the code signature or Add/Remove Programs. Links inside PDF
  documents still work, after the "Open this web page?" prompt.
- MSI installs only `pdfcraft.exe` and the licence files: no `pdfcraft-cli.exe`, no OCR models, no
  "Create PDF with PdfCraft…" menu on images. The PDF right-click entry reads "Mit PDF Reader öffnen".
- Display name "PDF Reader" (`pdfcraft_ui_egui::DISPLAY_NAME`, `apps/pdfcraft/build.rs`, the MSI):
  window title, About, Start Menu and desktop shortcut, Default apps, Add/Remove Programs, and the
  German catalog. Internal names stay PdfCraft (exe, registry keys, ProgId, install and settings
  folders), so updates and existing settings keep working. The German catalog (`de.tsv`) is
  complete (`cargo xtask i18n-report`: de 0 missing).
- MSI manufacturer is "SMS group". UpgradeCode, ProgId `PdfCraft.Document` and install folder are
  unchanged, so an SMS build and an upstream build replace each other.
- `packaging/windows/sign.ps1` can sign with a certificate from the CurrentUser\My store
  (`WINDOWS_CERTIFICATE_THUMBPRINT`), so the private key never leaves the build host.

Code and app icon stay under MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE, NOTICE,
ATTRIBUTION.md).

## Build and sign (Windows x64)

Needs: Rust 1.99 or newer (MSVC; 1.98 cannot read the craft-libs manifests), the Windows SDK
(signtool.exe, rc.exe), WiX v5 (`dotnet tool install --global wix --version 5.0.2`), and the SMS
code-signing certificate in CurrentUser\My.

```powershell
$env:WINDOWS_CERTIFICATE_THUMBPRINT = '<thumbprint of the SMS code-signing certificate>'
pwsh packaging/windows/package.ps1 -Arch x64
pwsh packaging/windows/test-msi.ps1 dist/release/pdfcraft-<ver>-windows-x64.msi
Get-FileHash dist/release/pdfcraft-<ver>-windows-x64.msi -Algorithm SHA256
```

The GitHub release workflow is not used: it calls `storytold/craft-common-actions` at `@main`,
and the SMS signing key must not be handed to third-party workflow code.

## Update to a new upstream release

```powershell
gh repo sync IT-BAER/pdfcraft --source storytold/pdfcraft --branch main
git fetch upstream --tags
git rebase --onto v<new> v<old> sms-build
cargo test -p pdfcraft-engine reader::
cargo test -p pdfcraft-ui-egui --test it -- links:: reader::
```

`reader::` in the engine checks that every allowlisted command and tool group still exists; a
failure after a rebase means upstream renamed one. New upstream commands are hidden until they are
added to the allowlist.

On a README.md conflict keep this file. Pin the tested version in the NinjaOne script
`PdfCraft.ps1` (source, SHA256, signer thumbprint, version).
