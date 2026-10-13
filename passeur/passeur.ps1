# Le passeur de Diorama : quand un personnage rend visite à une autre session,
# il le fait voler d'une fenêtre de terminal à l'autre (Windows seulement).
#
#   passeur.ps1 -Lancer     ce que fait le hook SessionStart : lance le passeur,
#                           caché et hors de la session, s'il ne tourne pas déjà
#   passeur.ps1 [-Racine <dossier>]
#                           le passeur, et attend qu'il s'arrête ; -Racine : sur un autre
#                           dossier .claude que le vrai (un banc d'essai)
#   passeur.ps1 -Essai <sprite.json> -De <session|pid> -Vers <session|pid> [-Colonnes 144] [-Lignes 16]
#                           un vol d'essai (tools/envol.mts le prépare)
#
# Le passeur est un petit programme à lui (passeur.cs), compilé une fois dans
# %LOCALAPPDATA%\Diorama\passeur.exe, et de nouveau quand passeur.cs change :
# au repos, il ne garde pas PowerShell en mémoire. Il note ce qu'il fait dans
# %TEMP%\diorama-passeur.log.

param(
  [switch]$Lancer,
  [string]$Essai,
  [string]$De,
  [string]$Vers,
  [int]$Colonnes = 144,
  [int]$Lignes = 16,
  [string]$Racine
)

$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'passeur.cs'
$cache = Join-Path $env:LOCALAPPDATA 'Diorama'
$exe = Join-Path $cache 'passeur.exe'
$print = Join-Path $cache 'passeur.empreinte' # l'empreinte du passeur.cs dont passeur.exe est fait

# passeur.exe à jour de passeur.cs : $null, ou pourquoi il ne l'est pas (noté aussi au journal)
function Build {
  $hash = (Get-FileHash $source -Algorithm SHA256).Hash
  if ((Test-Path $exe) -and (Test-Path $print) -and ((Get-Content $print -Raw).Trim() -eq $hash)) { return $null }
  New-Item -ItemType Directory -Force $cache | Out-Null
  try {
    Add-Type -ReferencedAssemblies System.Windows.Forms, System.Drawing, System.Web.Extensions `
      -TypeDefinition (Get-Content -Raw -Encoding UTF8 $source) -OutputAssembly $exe -OutputType WindowsApplication
  } catch {
    $why = "passeur.exe ne se refait pas : $($_.Exception.Message)"
    Add-Content -Path (Join-Path $env:TEMP 'diorama-passeur.log') -Value "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') $why" -Encoding UTF8
    return $why
  }
  Set-Content -Path $print -Value $hash -Encoding ascii
  return $null
}

if ($Lancer) {
  $running = $null
  if ([System.Threading.Mutex]::TryOpenExisting('Local\DioramaPasseur', [ref]$running)) { $running.Dispose(); exit 0 }
  if (Build) { exit 0 }
  # par WMI : le passeur ne fait pas partie de la session qui l'a lancé, et lui survit
  $startup = New-CimInstance -ClassName Win32_ProcessStartup -ClientOnly -Property @{ ShowWindow = [uint16]0 }
  Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{
    CommandLine = "`"$exe`" `"$source`""; ProcessStartupInformation = $startup
  } | Out-Null
  exit 0
}

$why = Build
if ($why) { Write-Error $why }

if ($Essai) {
  [void][System.Reflection.Assembly]::LoadFrom($exe)
  [Diorama.Passeur]::Try((Resolve-Path $Essai).Path, $De, $Vers, $Colonnes, $Lignes)
} else {
  $arguments = @("`"$source`"")
  if ($Racine) { $arguments += "`"$Racine`"" }
  Start-Process -FilePath $exe -ArgumentList $arguments -Wait
}
