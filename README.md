# PdfCraft, SMS build

Internal Windows build of PdfCraft for SMS group clients. It is based on
[PdfCraft](https://github.com/storytold/pdfcraft) by the ArtCraft team. It is not an ArtCraft
product and is not made, endorsed or supported by the ArtCraft team.

Branch `sms-build` holds the SMS changes on top of an upstream release tag. Branch `main` mirrors
upstream unchanged.

## Changes against upstream

- ArtCraft name, logos and community links removed (upstream [`docs/brand/LICENSE-brand.txt`](https://github.com/storytold/pdfcraft/blob/main/docs/brand/LICENSE-brand.txt) requires this for
  modified versions): no Discord button, no community card on the home screen, plain-text credit in
  the About dialog.
- Help ▸ Check for updates and the MSI support links point at this fork.
- MSI manufacturer is "SMS group". UpgradeCode, ProgId `PdfCraft.Document` and install folder are
  unchanged, so an SMS build and an upstream build replace each other.
- `packaging/windows/sign.ps1` can sign with a certificate from the CurrentUser\My store
  (`WINDOWS_CERTIFICATE_THUMBPRINT`), so the private key never leaves the build host.

Code and app icon stay under MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE, NOTICE,
ATTRIBUTION.md).

## Build and sign (Windows x64)

Needs: Rust (MSVC), the Windows SDK (signtool.exe, rc.exe), WiX v5 (`dotnet tool install --global wix --version 5.0.2`),
network access for the OCR models, and the SMS code-signing certificate in CurrentUser\My.

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
cargo test -p pdfcraft-ui-egui --test it links::
```

On a README.md conflict keep this file. Pin the tested version in the NinjaOne script
`PdfCraft.ps1` (source, SHA256, signer thumbprint, version).
