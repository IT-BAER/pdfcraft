<#
.SYNOPSIS
  SMS build: packs the staged, signed pdfcraft.exe of package.ps1 into an MSIX and signs it.
.DESCRIPTION
  Run after package.ps1, which stages and signs pdfcraft.exe in
  $CARGO_TARGET_DIR\windows-package\<Arch>. Writes pdfcraft-<version>-windows-<arch>.msix to
  $env:DIST (default dist/release). The package's Publisher must equal the signing certificate's
  subject, so it is read from WINDOWS_CERTIFICATE_THUMBPRINT's certificate in CurrentUser\My;
  without one the package uses "CN=SMS group" and stays unsigned (it then can't be installed).
  Needs makeappx.exe and signtool.exe from the Windows SDK.
  Installed side by side with the MSI, both register .pdf and keep separate settings (MSIX
  redirects %APPDATA% writes into the package), so deploy one or the other to a PC.
.EXAMPLE
  pwsh packaging/windows/package.ps1; pwsh packaging/windows/msix.ps1
#>
param(
  [ValidateSet('x64', 'arm64')] [string] $Arch = 'x64'
)
$ErrorActionPreference = 'Stop'
$Root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
$Stage = Join-Path $TargetDir "windows-package\$Arch"
if (-not (Test-Path (Join-Path $Stage 'pdfcraft.exe'))) { throw "no staged pdfcraft.exe in ${Stage}: run package.ps1 first" }

$cargo = Get-Content (Join-Path $Root 'Cargo.toml') -Raw
if ($cargo -notmatch '(?ms)^\[workspace\.package\].*?^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"') { throw 'no [workspace.package] version in Cargo.toml' }
# MSIX versions have four parts, each at most 65535.
$Version = "$($Matches[1]).$($Matches[2]).$($Matches[3])"
$PackageVersion = "$Version.0"

$Publisher = 'CN=SMS group'
if ($env:WINDOWS_CERTIFICATE_THUMBPRINT) {
  $cert = Get-Item -LiteralPath "Cert:\CurrentUser\My\$($env:WINDOWS_CERTIFICATE_THUMBPRINT)" -ErrorAction Stop
  $Publisher = $cert.Subject
}

function Find-SdkTool([string] $Name) {
  $onPath = Get-Command $Name -ErrorAction SilentlyContinue
  if ($onPath) { return $onPath.Source }
  $found = Get-ChildItem (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin') -Recurse -Filter $Name -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match '\\x64\\' } | Sort-Object FullName -Descending | Select-Object -First 1
  if (-not $found) { throw "$Name not found (install the Windows SDK)" }
  $found.FullName
}
$MakeAppx = Find-SdkTool 'makeappx.exe'

$Pack = Join-Path $TargetDir "windows-package\msix-$Arch"
Remove-Item -Recurse -Force $Pack -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path (Join-Path $Pack 'Assets') | Out-Null
foreach ($f in 'pdfcraft.exe', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'LICENSE-document.txt') {
  Copy-Item (Join-Path $Stage $f) $Pack
}
$fontLicences = Join-Path $Stage 'font-licences'
if (Get-ChildItem $fontLicences -File -ErrorAction SilentlyContinue) { Copy-Item -Recurse $fontLicences $Pack }

# Tile and list logos from the app icon (assets/app-icon, ATTRIBUTION.toml), at the sizes the manifest names.
Add-Type -AssemblyName System.Drawing
$icon = Join-Path $Root 'assets\app-icon\hicolor\256x256\apps\ai.storyteller.pdfcraft.png'
$source = [System.Drawing.Image]::FromFile($icon)
try {
  foreach ($logo in @(@('Square44x44Logo.png', 44), @('Square150x150Logo.png', 150), @('StoreLogo.png', 50))) {
    $bitmap = [System.Drawing.Bitmap]::new($logo[1], $logo[1])
    try {
      $g = [System.Drawing.Graphics]::FromImage($bitmap)
      $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
      $g.DrawImage($source, 0, 0, $logo[1], $logo[1])
      $g.Dispose()
      $bitmap.Save((Join-Path $Pack "Assets\$($logo[0])"), [System.Drawing.Imaging.ImageFormat]::Png)
    } finally { $bitmap.Dispose() }
  }
} finally { $source.Dispose() }

$xmlPublisher = [System.Security.SecurityElement]::Escape($Publisher)
@"
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
         IgnorableNamespaces="uap rescap">
  <Identity Name="SMSgroup.PDFReader" Publisher="$xmlPublisher" Version="$PackageVersion" ProcessorArchitecture="$Arch" />
  <Properties>
    <DisplayName>PDF Reader</DisplayName>
    <PublisherDisplayName>SMS group</PublisherDisplayName>
    <Logo>Assets\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Resources>
    <Resource Language="de-DE" />
  </Resources>
  <Applications>
    <Application Id="PDFReader" Executable="pdfcraft.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="PDF Reader" Description="PDF Reader" BackgroundColor="transparent"
                          Square150x150Logo="Assets\Square150x150Logo.png" Square44x44Logo="Assets\Square44x44Logo.png" />
      <Extensions>
        <uap:Extension Category="windows.fileTypeAssociation">
          <uap:FileTypeAssociation Name="pdf">
            <uap:DisplayName>PDF-Dokument</uap:DisplayName>
            <uap:Logo>Assets\Square44x44Logo.png</uap:Logo>
            <uap:SupportedFileTypes>
              <uap:FileType>.pdf</uap:FileType>
            </uap:SupportedFileTypes>
          </uap:FileTypeAssociation>
        </uap:Extension>
      </Extensions>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
"@ | Set-Content -Encoding utf8NoBOM (Join-Path $Pack 'AppxManifest.xml')

New-Item -ItemType Directory -Force -Path $Dist | Out-Null
$Msix = Join-Path $Dist "pdfcraft-$Version-windows-$Arch.msix"
& $MakeAppx pack /o /h SHA256 /d $Pack /p $Msix
if ($LASTEXITCODE -ne 0) { throw "makeappx failed with exit code $LASTEXITCODE" }
& (Join-Path $PSScriptRoot 'sign.ps1') $Msix
Write-Output "ok MSIX: $Msix ($PackageVersion, publisher $Publisher)"
