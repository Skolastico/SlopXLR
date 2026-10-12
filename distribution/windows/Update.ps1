param([ValidateSet('Check','Install','SelfTest')][string]$Mode='Check',[string]$ExpectedVersion)
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$OutputEncoding=[Console]::OutputEncoding
[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12
$script:AppDir=$PSScriptRoot
function Compare-Version($left,$right){
 $pattern='^(\d+)\.(\d+)\.(\d+)(?:-beta\.(\d+))?$'
 if($left -notmatch $pattern){throw 'Invalid release version'}
 $a=@($Matches[1],$Matches[2],$Matches[3],$(if($Matches[4]){$Matches[4]}else{[int]::MaxValue}))
 if($right -notmatch $pattern){throw 'Invalid installed version'}
 $b=@($Matches[1],$Matches[2],$Matches[3],$(if($Matches[4]){$Matches[4]}else{[int]::MaxValue}))
 for($i=0;$i -lt 4;$i++){if([int]$a[$i] -gt [int]$b[$i]){return 1};if([int]$a[$i] -lt [int]$b[$i]){return -1}}
 return 0
}
function Verify-SignedManifest([byte[]]$Bytes,[string]$Signature,[string]$PublicKey){
 $rsa=[Security.Cryptography.RSA]::Create()
 try{$rsa.FromXmlString($PublicKey);if(!$rsa.VerifyData($Bytes,[Convert]::FromBase64String($Signature),[Security.Cryptography.HashAlgorithmName]::SHA256,[Security.Cryptography.RSASignaturePadding]::Pkcs1)){throw 'Update signature is invalid'}}finally{$rsa.Dispose()}
}
function Assert-DownloadUrl($Url,$Repository){
 $uri=[Uri]$Url
 if($uri.Scheme -ne 'https' -or $uri.Host -ne 'github.com' -or !$uri.AbsolutePath.StartsWith('/'+$Repository+'/releases/download/',[StringComparison]::OrdinalIgnoreCase) -or $uri.UserInfo -or $uri.Query){throw 'Untrusted release URL'}
}
function Get-VerifiedRelease {
 $local=Get-Content -LiteralPath (Join-Path $script:AppDir 'release.json') -Raw | ConvertFrom-Json
 if($local.repository -ne 'Skolastico/SlopXLR'){throw 'Unexpected update repository'}
 try{$releases=Invoke-RestMethod ('https://api.github.com/repos/'+$local.repository+'/releases?per_page=20') -Headers @{'User-Agent'='GoXLR-Personalizado';Accept='application/vnd.github+json'} -TimeoutSec 15}catch{if($_.Exception.Response.StatusCode.value__ -eq 404){return @{available=$false;message='Ainda não há versões públicas disponíveis. Se o repositório for privado, consulta as versões no GitHub.'}};throw}
 $release=$releases | Where-Object {!$_.draft -and ($local.channel -eq 'beta' -or !$_.prerelease) -and $_.tag_name -match '^v\d+\.\d+\.\d+(?:-beta\.\d+)?$'} | Select-Object -First 1
 if(!$release){return @{available=$false;message='Ainda não existem versões publicadas.'}}
 $version=$release.tag_name.TrimStart('v')
 if((Compare-Version $version $local.version) -le 0){return @{available=$false;version=$local.version;message='A versão instalada está atualizada.'}}
 $asset=$release.assets | Where-Object name -EQ 'update.json' | Select-Object -First 1
 $sig=$release.assets | Where-Object name -EQ 'update.json.sig' | Select-Object -First 1
 if(!$asset -or !$sig){throw 'Release is missing signed update metadata'}
 Assert-DownloadUrl $asset.browser_download_url $local.repository
 Assert-DownloadUrl $sig.browser_download_url $local.repository
 $client=New-Object Net.WebClient
 try{$bytes=$client.DownloadData($asset.browser_download_url);$signature=$client.DownloadString($sig.browser_download_url).Trim()}finally{$client.Dispose()}
 Verify-SignedManifest $bytes $signature (Get-Content -LiteralPath (Join-Path $script:AppDir 'update-public.xml') -Raw)
 $manifest=[Text.Encoding]::UTF8.GetString($bytes)|ConvertFrom-Json
 if($manifest.version -ne $version -or $manifest.repository -ne $local.repository){throw 'Release metadata mismatch'}
 $kind=if(Test-Path (Join-Path $script:AppDir 'installed.marker')){'setup'}else{'portable'}
 $package=$manifest.$kind
 if(!$package -or $package.sha256 -notmatch '^[a-fA-F0-9]{64}$'){throw 'Invalid package metadata'}
 Assert-DownloadUrl $package.url $local.repository
 return @{available=$true;version=$version;url=$package.url;sha256=$package.sha256;kind=$kind}
}
if($Mode -eq 'SelfTest'){
 if((Compare-Version '0.1.0-beta.2' '0.1.0-beta.1') -ne 1){throw 'Beta ordering failed'}
 if((Compare-Version '0.1.0' '0.1.0-beta.9') -ne 1){throw 'Stable ordering failed'}
 if((Compare-Version '0.2.0-beta.1' '0.1.9') -ne 1){throw 'Core ordering failed'}
 if((Compare-Version '0.1.0' '0.1.0') -ne 0){throw 'Equality failed'}
 $rsa=[Security.Cryptography.RSA]::Create(2048)
 try{$bytes=[Text.Encoding]::UTF8.GetBytes('trusted');$sig=[Convert]::ToBase64String($rsa.SignData($bytes,[Security.Cryptography.HashAlgorithmName]::SHA256,[Security.Cryptography.RSASignaturePadding]::Pkcs1));Verify-SignedManifest $bytes $sig $rsa.ToXmlString($false);$rejected=$false;try{Verify-SignedManifest ([Text.Encoding]::UTF8.GetBytes('tampered')) $sig $rsa.ToXmlString($false)}catch{$rejected=$true};if(!$rejected){throw 'Tampering accepted'}}finally{$rsa.Dispose()}
 $rejected=$false;try{Assert-DownloadUrl 'https://github.com.evil.test/Skolastico/SlopXLR/releases/download/v1/a.exe' 'Skolastico/SlopXLR'}catch{$rejected=$true};if(!$rejected){throw 'Lookalike host accepted'}
 'Update verification tests passed';exit
}
$stopped=$false
$stateFile=Join-Path $script:AppDir 'update-status.json'
try{
 $release=Get-VerifiedRelease
 if($Mode -eq 'Check'){if(Test-Path $stateFile){$release.status=Get-Content $stateFile -Raw|ConvertFrom-Json};$release|ConvertTo-Json -Compress -Depth 5;exit}
 if(!$release.available -or $release.version -ne $ExpectedVersion){throw 'The requested release is no longer available'}
 @{state='downloading';version=$release.version}|ConvertTo-Json|Set-Content $stateFile -Encoding utf8
 $stage=Join-Path $env:TEMP ('GoXLR-Personalizado-'+[Guid]::NewGuid().ToString('N'))
 New-Item -ItemType Directory -Path $stage|Out-Null
 $package=Join-Path $stage $(if($release.kind -eq 'setup'){'setup.exe'}else{'portable.zip'})
 Invoke-WebRequest -Uri $release.url -OutFile $package -UseBasicParsing -TimeoutSec 120
 if((Get-FileHash -LiteralPath $package -Algorithm SHA256).Hash -ne $release.sha256){throw 'Package hash mismatch'}
 if($release.kind -eq 'portable'){
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $zip=[IO.Compression.ZipFile]::OpenRead($package)
  try{foreach($entry in $zip.Entries){$destination=[IO.Path]::GetFullPath((Join-Path $stage ('new\'+$entry.FullName)));$newRoot=[IO.Path]::GetFullPath((Join-Path $stage 'new'))+[IO.Path]::DirectorySeparatorChar;if(!$destination.StartsWith($newRoot,[StringComparison]::OrdinalIgnoreCase)){throw 'Unsafe ZIP entry'}}}finally{$zip.Dispose()}
  Expand-Archive -LiteralPath $package -DestinationPath (Join-Path $stage 'new')
 }
 $owned=@(Get-Process -Name goxlr-daemon -ErrorAction SilentlyContinue|Where-Object Path -EQ (Join-Path $script:AppDir 'goxlr-daemon.exe'))
 if($owned.Count -ne 1){throw 'Cannot identify the running application'}
 $root=if(Test-Path (Join-Path $script:AppDir 'config\settings.json')){$script:AppDir}elseif($release.kind -eq 'setup'){Join-Path $env:APPDATA 'Skolastico\GoXLR-Personalizado'}else{Join-Path $script:AppDir 'user-data'}
 $backup=Join-Path $root ('update-backups\'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
 New-Item -ItemType Directory -Path $backup -Force|Out-Null
 Invoke-RestMethod 'http://localhost:14564/api/command' -Method Post -ContentType application/json -Body '{"Daemon":"StopDaemon"}' -TimeoutSec 10|Out-Null
 if(!$owned[0].WaitForExit(20000)){throw 'The service did not stop. No files were replaced.'}
 $stopped=$true
 foreach($process in @(Get-Process -Name goxlr-utility-ui -ErrorAction SilentlyContinue|Where-Object Path -EQ (Join-Path $script:AppDir 'goxlr-utility-ui.exe'))){if(!$process.WaitForExit(5000)){throw 'The window did not close. No files were replaced.'}}
 if(Test-Path (Join-Path $root 'config')){Copy-Item -LiteralPath (Join-Path $root 'config') -Destination $backup -Recurse}
 foreach($folder in @('profiles','mic-profiles')){if(Test-Path (Join-Path $root ('data\'+$folder))){Copy-Item -LiteralPath (Join-Path $root ('data\'+$folder)) -Destination $backup -Recurse}}
 $managed=@('goxlr-daemon.exe','goxlr-utility-ui.exe','goxlr-defaults.exe','SAAPI64.dll','nvdaControllerClient64.dll','release.json','update-public.xml','Start.ps1','Open.cmd','Update.ps1','Preflight.ps1','README.txt','LICENSE','LICENSE-3RD-PARTY','LICENSE-UI','WebView2Setup.exe')
 foreach($file in $managed){if(Test-Path (Join-Path $script:AppDir $file)){Copy-Item -LiteralPath (Join-Path $script:AppDir $file) -Destination $backup}}
 @{state='installing';version=$release.version;backup=$backup}|ConvertTo-Json|Set-Content $stateFile -Encoding utf8
 try{
  if($release.kind -eq 'setup'){$installer=Start-Process -FilePath $package -ArgumentList @('/S',('/D='+$script:AppDir)) -Wait -PassThru -WindowStyle Hidden;if($installer.ExitCode -ne 0){throw 'Installer failed'}}
  else{foreach($file in $managed){Copy-Item -LiteralPath (Join-Path $stage ('new\'+$file)) -Destination (Join-Path $script:AppDir $file) -Force}}
  $current=Get-Content -LiteralPath (Join-Path $script:AppDir 'release.json') -Raw|ConvertFrom-Json
  if($current.version -ne $release.version){throw 'Installed version mismatch'}
  $startup=Start-Process powershell.exe -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-WindowStyle','Hidden','-File',('"'+(Join-Path $script:AppDir 'Start.ps1')+'"'),'-Background') -PassThru -WindowStyle Hidden
  if(!$startup.WaitForExit(25000) -or $startup.ExitCode -ne 0){throw 'The updated service did not start'}
  Invoke-RestMethod 'http://localhost:14564/api/get-devices' -TimeoutSec 5|Out-Null
  $stopped=$false
  @{state='complete';version=$release.version;backup=$backup}|ConvertTo-Json|Set-Content $stateFile -Encoding utf8
 }catch{
  $failure=$_.Exception.Message
  $failed=@(Get-Process -Name goxlr-daemon -ErrorAction SilentlyContinue|Where-Object Path -EQ (Join-Path $script:AppDir 'goxlr-daemon.exe'))
  if($failed.Count){try{Invoke-RestMethod 'http://localhost:14564/api/command' -Method Post -ContentType application/json -Body '{"Daemon":"StopDaemon"}' -TimeoutSec 5|Out-Null}catch{};if(!$failed[0].WaitForExit(15000)){throw 'The failed updated service did not stop. Restore the backup manually.'}}
  if(Test-Path (Join-Path $backup 'config')){Get-ChildItem -LiteralPath (Join-Path $backup 'config') -File|Copy-Item -Destination (Join-Path $root 'config') -Force}
  foreach($folder in @('profiles','mic-profiles')){if(Test-Path (Join-Path $backup $folder)){Get-ChildItem -LiteralPath (Join-Path $backup $folder) -File|Copy-Item -Destination (Join-Path $root ('data\\'+$folder)) -Force}}
  foreach($file in $managed){if(Test-Path (Join-Path $backup $file)){Copy-Item -LiteralPath (Join-Path $backup $file) -Destination (Join-Path $script:AppDir $file) -Force}};throw $failure}
 Start-Process powershell.exe -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-WindowStyle','Hidden','-File',('"'+(Join-Path $script:AppDir 'Start.ps1')+'"')) -WindowStyle Hidden
}catch{
 @{state='error';message=$_.Exception.Message}|ConvertTo-Json|Set-Content $stateFile -Encoding utf8
 if($Mode -eq 'Check'){@{Error=$_.Exception.Message}|ConvertTo-Json -Compress}
 else{if($stopped){Start-Process powershell.exe -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-WindowStyle','Hidden','-File',('"'+(Join-Path $script:AppDir 'Start.ps1')+'"')) -WindowStyle Hidden};Add-Type -AssemblyName PresentationFramework;[Windows.MessageBox]::Show($_.Exception.Message,'GoXLR Personalizado update')|Out-Null}
 exit 1
}
