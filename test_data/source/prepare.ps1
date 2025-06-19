# Get the directory where this script resides
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Iterate through each subdirectory and compress it
Get-ChildItem -Path $scriptDir -Directory | ForEach-Object {
    $dir = $_
    $zipName = "$($dir.Name).zip"
    $zipPath = Join-Path $scriptDir $zipName

    Write-Host "Compressing '$($dir.FullName)' to '$zipPath'..."
    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }

    Compress-Archive -Path (Join-Path $dir.FullName '*') -DestinationPath $zipPath -Force
    Write-Host "Created $zipName"
}