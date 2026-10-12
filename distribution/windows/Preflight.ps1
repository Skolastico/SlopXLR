param([switch]$Uninstall)
$ErrorActionPreference='Stop'
if(@(Get-Process -Name goxlr-daemon,'GoXLR App','GoXLR Beta App' -ErrorAction SilentlyContinue).Count){exit 3}
if($Uninstall){exit 0}
$driver='C:\Program Files\TC-HELICON\GoXLR_Audio_Driver\W10_x64\goxlr_audioapi_x64.dll'
try{$registered=(Get-Item 'Registry::HKEY_CLASSES_ROOT\CLSID\{024D0372-641F-4B7B-8140-F4DFE458C982}\InprocServer32').GetValue('')}catch{$registered=$null}
if(!(Test-Path -LiteralPath $driver) -and (!$registered -or !(Test-Path -LiteralPath $registered))){exit 2}
exit 0
