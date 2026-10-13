param([switch]$Background,[string]$ImportFrom,[switch]$PrepareOnly,[string]$DataRoot)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName PresentationFramework
$script:English=([Globalization.CultureInfo]::CurrentUICulture.TwoLetterISOLanguageName -ne 'pt')
function Msg($pt,$en){if($script:English){$en}else{$pt}}
function Initialize-UserData {
 param([string]$Root,[string]$Source)
 $target=Join-Path $Root 'config\settings.json'
 if(Test-Path -LiteralPath $target){return $target}
 $settings=@{show_tray_icon=$true;open_ui_on_launch=$false;allow_network_access=$false;tts_enabled=$false;button_layers=@{};application_bindings=@{}}
 if($Source){
  if(!(Test-Path -LiteralPath $Source -PathType Leaf)){throw (Msg 'Não encontrei o ficheiro de configurações para importar.' 'Import settings file was not found.')}
  $settings=Get-Content -LiteralPath $Source -Raw | ConvertFrom-Json
 }
 New-Item -ItemType Directory -Path (Split-Path $target) -Force | Out-Null
 $folders=@{profile_directory='profiles';mic_profile_directory='mic-profiles';presets_directory='presets';icons_directory='icons';samples_directory='samples';backup_directory='backups';logs_directory='logs'}
 foreach($entry in $folders.GetEnumerator()){
  $destination=Join-Path $Root ('data\'+$entry.Value)
  New-Item -ItemType Directory -Path $destination -Force | Out-Null
  if($Source){
   $sourcePath=$settings.($entry.Key)
   if(!$sourcePath){$sourcePath=Join-Path (Split-Path (Split-Path $Source)) ('data\'+$entry.Value)}
   if($entry.Value -notin @('logs','backups') -and (Test-Path -LiteralPath $sourcePath)){
    Get-ChildItem -LiteralPath $sourcePath -Force | Copy-Item -Destination $destination -Recurse -Force
   }
   $settings | Add-Member -NotePropertyName $entry.Key -NotePropertyValue $destination -Force
  }else{$settings[$entry.Key]=$destination}
 }
 if($Source){Copy-Item -LiteralPath $Source -Destination (Join-Path $Root 'config\imported-settings-backup.json')}
 $settings | ConvertTo-Json -Depth 50 | Set-Content -LiteralPath ($target+'.tmp') -Encoding utf8
 Move-Item -LiteralPath ($target+'.tmp') -Destination $target
 return $target
}
if($PrepareOnly){Initialize-UserData -Root $DataRoot -Source $ImportFrom;exit}
try{
 $appDir=$PSScriptRoot
 $daemon=Join-Path $appDir 'goxlr-daemon.exe'
 $native=Join-Path $appDir 'goxlr-utility-ui.exe'
 $running=@(Get-Process -Name goxlr-daemon -ErrorAction SilentlyContinue)
 if($running.Count -and ($running.Count -ne 1 -or $running[0].Path -ne $daemon)){throw (Msg 'Outra versão do GoXLR Utility está aberta. Encerra-a pelo ícone junto ao relógio e volta a tentar.' 'Another GoXLR Utility is running. Quit it from the tray icon and try again.')}
 if(!$DataRoot){
  if(Test-Path (Join-Path $appDir 'config\settings.json')){$DataRoot=$appDir}
  elseif(Test-Path (Join-Path $appDir 'installed.marker')){$DataRoot=Join-Path $env:APPDATA 'Skolastico\GoXLR-Personalizado'}
  else{$DataRoot=Join-Path $appDir 'user-data'}
 }
 if(!$running.Count){
  if(!(Test-Path -LiteralPath (Join-Path $DataRoot 'config\settings.json')) -and !$ImportFrom -and !$Background){
   $original=Join-Path $env:APPDATA 'GoXLR-on-Linux\GoXLR-Utility\config\settings.json'
   if(Test-Path -LiteralPath $original){
    $choice=[System.Windows.MessageBox]::Show((Msg 'Importar perfis, ações e samples do GoXLR Utility existente? Escolhe Não para começar de raiz.' 'Import profiles, actions and samples from the existing GoXLR Utility? Choose No to start fresh.'),'SlopXLR','YesNoCancel')
    if($choice -eq 'Cancel'){exit}
    if($choice -eq 'Yes'){$ImportFrom=$original}
   }
  }
  $config=Initialize-UserData -Root $DataRoot -Source $ImportFrom
  Start-Process -FilePath $daemon -ArgumentList @('--config',('"'+$config+'"')) -WorkingDirectory $appDir -WindowStyle Hidden | Out-Null
 }
 $ready=$false
 for($i=0;$i -lt 40;$i++){try{$state=Invoke-RestMethod 'http://localhost:14564/api/get-devices' -TimeoutSec 2;if($state.config.daemon_version){$ready=$true;break}}catch{};Start-Sleep -Milliseconds 250}
 if(!$ready){throw (Msg 'O serviço não respondeu. Verifica o driver GoXLR e consulta os registos na pasta de dados.' 'The service did not respond. Check the GoXLR driver and the logs in the data folder.')}
 $body=@{Daemon=@{SetActivatorPath=('"'+$native+'"')}}|ConvertTo-Json -Compress
 Invoke-RestMethod 'http://localhost:14564/api/command' -Method Post -ContentType application/json -Body ([Text.Encoding]::UTF8.GetBytes($body)) -TimeoutSec 5 | Out-Null
 if(!$Background){Invoke-RestMethod 'http://localhost:14564/api/command' -Method Post -ContentType application/json -Body '{"Daemon":"Activate"}' -TimeoutSec 5 | Out-Null}
}catch{[System.Windows.MessageBox]::Show($_.Exception.Message,'SlopXLR')|Out-Null;exit 1}
