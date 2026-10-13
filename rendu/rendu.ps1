# L'appli de rendu de Diorama (Windows) : elle dessine les bandeaux en vrais pixels,
# par-dessus le terminal (rendu/src, en Rust).
#
#   rendu.ps1 -Lancer [-Cadence <ms>]
#       ce que fait le plugin au début d'une session : refait l'appli si sa source a changé,
#       puis la lance, cachée et hors de la session, si elle ne tourne pas déjà ;
#       une appli d'une source plus ancienne laisse sa place
#   rendu.ps1 -Refaire
#       refait l'appli, sans la lancer
#
# L'appli est compilée par cargo dans %LOCALAPPDATA%\Diorama (rendu-cible, puis rendu.exe), et de
# nouveau quand sa source change. Elle et ce lanceur notent ce qu'ils font dans
# %TEMP%\diorama-rendu.log.

param([switch]$Lancer, [switch]$Refaire, [int]$Cadence = 0)

$ErrorActionPreference = 'Stop'
$cache = Join-Path $env:LOCALAPPDATA 'Diorama'
$exe = Join-Path $cache 'rendu.exe'
$print = Join-Path $cache 'rendu.empreinte' # l'empreinte de la source dont rendu.exe est fait
$log = Join-Path $env:TEMP 'diorama-rendu.log'
$port = 41917 # ecoute.rs : PORT

function Note([string]$line) {
  Add-Content -Path $log -Value "$(Get-Date -Format 'HH:mm:ss') rendu.ps1 : $line" -Encoding UTF8
}

# l'empreinte de la source : Cargo.toml, Cargo.lock, les .rs de src (sous-dossiers compris) et
# les images que l'appli embarque (pet/)
function Print {
  $files = @('Cargo.toml', 'Cargo.lock' | ForEach-Object { Join-Path $PSScriptRoot $_ }) +
    @(Get-ChildItem (Join-Path $PSScriptRoot 'src') -Filter *.rs -Recurse | Sort-Object FullName | ForEach-Object FullName) +
    @(Get-ChildItem (Join-Path $PSScriptRoot 'pet') -File -ErrorAction SilentlyContinue | Sort-Object FullName | ForEach-Object FullName)
  $all = ($files | Where-Object { Test-Path $_ } | ForEach-Object { (Get-FileHash $_ -Algorithm SHA256).Hash }) -join ''
  $sha = [System.Security.Cryptography.SHA256]::Create()
  ($sha.ComputeHash([System.Text.Encoding]::ASCII.GetBytes($all)) | ForEach-Object { $_.ToString('x2') }) -join ''
}

function Running {
  $m = $null
  if ([System.Threading.Mutex]::TryOpenExisting('Local\DioramaRendu', [ref]$m)) { $m.Dispose(); return $true }
  return $false
}

# rendu.exe à jour de la source : $true, ou $false s'il ne se refait pas (noté au journal)
function Build {
  $hash = Print
  if ((Test-Path $exe) -and (Test-Path $print) -and ((Get-Content $print -Raw).Trim() -eq $hash)) { return $true }
  if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Note 'pas de cargo : l''appli ne se fait pas'; return $false }
  New-Item -ItemType Directory -Force $cache | Out-Null
  Note 'la source a changé : cargo build'
  $target = Join-Path $cache 'rendu-cible'
  $out = & cargo build --release --quiet --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml') --target-dir $target 2>&1
  if ($LASTEXITCODE -ne 0) { Note "cargo a échoué : $(($out | Select-Object -Last 5) -join ' / ')"; return $false }
  # une appli d'avant tient encore son exécutable : elle laisse sa place
  if (Running) {
    try { Invoke-WebRequest -UseBasicParsing -Method Post -Uri "http://127.0.0.1:$port/" -Body 'quitter' -TimeoutSec 2 | Out-Null } catch { }
    for ($i = 0; $i -lt 30 -and (Running); $i++) { Start-Sleep -Milliseconds 100 }
  }
  Copy-Item (Join-Path $target 'release\rendu.exe') $exe -Force
  Set-Content -Path $print -Value $hash -Encoding ascii
  Note 'appli refaite'
  return $true
}

# un seul lanceur à la fois : deux sessions qui démarrent ensemble ne compilent pas deux fois
$guard = New-Object System.Threading.Mutex($false, 'Local\DioramaRenduLanceur')
[void]$guard.WaitOne()
try {
  if (-not (Build)) { exit 0 }
  if ($Refaire -or -not $Lancer -or (Running)) { exit 0 }
  # par WMI : l'appli ne fait pas partie de la session qui l'a lancée, et lui survit
  $line = "`"$exe`""
  if ($Cadence -gt 0) { $line += " --cadence $Cadence" }
  $startup = New-CimInstance -ClassName Win32_ProcessStartup -ClientOnly -Property @{ ShowWindow = [uint16]0 }
  Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{
    CommandLine = $line; ProcessStartupInformation = $startup
  } | Out-Null
} finally {
  $guard.ReleaseMutex()
  $guard.Dispose()
}
