param([switch]$SkipBuild,[string]$UiBinary,[string]$SigningKeyPath,[string]$NsisPath,[string]$WebViewBootstrap)
$ErrorActionPreference='Stop'
$repo=Split-Path $PSScriptRoot
$release=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'release.json') -Raw|ConvertFrom-Json
if(!$SkipBuild){
 Push-Location $repo
 try{
  & cargo build --release -p goxlr-daemon -p goxlr-defaults --locked
  if($LASTEXITCODE){throw 'Service build failed'}
  & cargo build --manifest-path ui-wrapper/src-tauri/Cargo.toml --target-dir target --release --locked
  if($LASTEXITCODE){throw 'Window build failed'}
 }finally{Pop-Location}
}
if(!$UiBinary){$UiBinary=Join-Path $repo 'target\release\goxlr-utility-ui.exe'}
if(!$NsisPath){$NsisPath=(Get-Command makensis.exe -ErrorAction Stop).Source}
if(!$SigningKeyPath -or !(Test-Path -LiteralPath $SigningKeyPath)){throw 'A private update signing key is required. Keep it outside Git.'}
$output=Join-Path $repo 'dist'
$stage=Join-Path $output ('package-'+$release.version)
New-Item -ItemType Directory -Path $stage -Force|Out-Null
foreach($file in @('goxlr-daemon.exe','goxlr-defaults.exe','SAAPI64.dll','nvdaControllerClient64.dll')){Copy-Item -LiteralPath (Join-Path $repo ('target\release\'+$file)) -Destination $stage}
Copy-Item -LiteralPath $UiBinary -Destination (Join-Path $stage 'goxlr-utility-ui.exe')
foreach($file in @('Start.ps1','Open.cmd','Update.ps1','Preflight.ps1')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot ('windows\'+$file)) -Destination $stage}
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'release.json') -Destination $stage
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'update-public.xml') -Destination $stage
Copy-Item -LiteralPath (Join-Path $repo 'LICENSE') -Destination $stage
Copy-Item -LiteralPath (Join-Path $repo 'LICENSE-3RD-PARTY') -Destination $stage
Copy-Item -LiteralPath (Join-Path $repo 'ui-wrapper\LICENSE') -Destination (Join-Path $stage 'LICENSE-UI')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'README.txt') -Destination $stage
if(!$WebViewBootstrap){
 $WebViewBootstrap=Join-Path $output 'WebView2Setup.exe'
 Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $WebViewBootstrap -UseBasicParsing
}
$signature=Get-AuthenticodeSignature -LiteralPath $WebViewBootstrap
if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation'){throw 'WebView2 bootstrap signature is not trusted'}
Copy-Item -LiteralPath $WebViewBootstrap -Destination (Join-Path $stage 'WebView2Setup.exe')
$setupName='SlopXLR-'+$release.version+'-setup.exe'
$zipName='SlopXLR-'+$release.version+'-portable.zip'
$setup=Join-Path $output $setupName
& $NsisPath '/INPUTCHARSET' 'UTF8' ('/DVERSION='+$release.version) ('/DPACKAGE_DIR='+$stage) ('/DOUTPUT='+$setup) (Join-Path $PSScriptRoot 'windows\installer.nsi')
if($LASTEXITCODE){throw 'Installer compilation failed'}
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath (Join-Path $output $zipName) -Force
$base='https://github.com/'+$release.repository+'/releases/download/v'+$release.version+'/'
$manifest=[ordered]@{version=$release.version;repository=$release.repository;setup=@{url=$base+$setupName;sha256=(Get-FileHash $setup).Hash.ToLower()};portable=@{url=$base+$zipName;sha256=(Get-FileHash (Join-Path $output $zipName)).Hash.ToLower()}}
$encoding=[Text.UTF8Encoding]::new($false)
$metadata=Join-Path $output 'update.json'
[IO.File]::WriteAllText($metadata,($manifest|ConvertTo-Json -Depth 8),$encoding)
$rsa=[Security.Cryptography.RSA]::Create()
try{
 $rsa.FromXmlString((Get-Content -LiteralPath $SigningKeyPath -Raw))
 $expected=(Get-Content -LiteralPath (Join-Path $PSScriptRoot 'update-public.xml') -Raw).Trim()
 if($rsa.ToXmlString($false) -ne $expected){throw 'The signing key does not match the pinned public key'}
 $signature=[Convert]::ToBase64String($rsa.SignData([IO.File]::ReadAllBytes($metadata),[Security.Cryptography.HashAlgorithmName]::SHA256,[Security.Cryptography.RSASignaturePadding]::Pkcs1))
 [IO.File]::WriteAllText((Join-Path $output 'update.json.sig'),$signature,$encoding)
}finally{$rsa.Dispose()}
@($setupName,$zipName,'update.json','update.json.sig')|ForEach-Object {Get-FileHash -LiteralPath (Join-Path $output $_)}|ForEach-Object {$_.Hash.ToLower()+'  '+(Split-Path $_.Path -Leaf)}|Set-Content (Join-Path $output 'SHA256SUMS.txt') -Encoding ascii
Write-Output ('Release package created: '+$output)
