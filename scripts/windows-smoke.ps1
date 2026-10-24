$ErrorActionPreference = 'Stop'
# This script is restricted to an ephemeral GitHub-hosted Windows runner.
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows') { throw 'Run only on isolated Windows CI' }
$root = Join-Path $env:RUNNER_TEMP ('pile-acceptance-' + [guid]::NewGuid())
$db = Join-Path $env:LOCALAPPDATA 'software.feedthepile.app\pile.sqlite3'
if (Test-Path $db) { throw 'Refusing to replace an existing journal' }
cargo run --locked --manifest-path core/Cargo.toml --example acceptance_fixture -- $root $db
if ($LASTEXITCODE -ne 0) { throw 'Fixture setup failed' }
$installer = Get-ChildItem 'src-tauri/target/release/bundle/nsis/*.exe' | Select-Object -First 1
if (!$installer) { throw 'Missing NSIS installer' }
$install = Start-Process $installer.FullName -ArgumentList '/S' -Wait -PassThru
if ($install.ExitCode -ne 0) { throw "Installer failed: $($install.ExitCode)" }
$candidates = @((Join-Path $env:LOCALAPPDATA 'Feed the Pile'), (Join-Path $env:LOCALAPPDATA 'Programs\Feed the Pile'))
$exe = $candidates | Where-Object { Test-Path $_ } | ForEach-Object { Get-ChildItem $_ -Filter 'feed-the-pile.exe' -Recurse } | Select-Object -First 1
if (!$exe) { throw 'Installed application was not found' }
node scripts/windows-smoke.mjs $exe.FullName $root
if ($LASTEXITCODE -ne 0) { throw 'Installed application acceptance failed' }
