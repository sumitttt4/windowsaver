$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class KnownFolders {
 [DllImport("shell32.dll")] private static extern int SHGetKnownFolderPath([MarshalAs(UnmanagedType.LPStruct)] Guid id, uint flags, IntPtr token, out IntPtr path);
 public static string Get(string id) {
  IntPtr p; int hr = SHGetKnownFolderPath(new Guid(id), 0, IntPtr.Zero, out p);
  if (hr != 0) return null;
  try { return Marshal.PtrToStringUni(p); } finally { Marshal.FreeCoTaskMem(p); }
 }
}
'@
$downloads = [KnownFolders]::Get('374DE290-123F-4565-9164-39C4925E467B')
$screenshots = [KnownFolders]::Get('B7BEDE81-DF94-4682-A7D8-57A52620B86F')
$pictures = [KnownFolders]::Get('33E28130-4E1E-4676-835A-98395C3BC3BB')
$fallback = if ($pictures) { Join-Path $pictures 'Screenshots' } else { $null }
[ordered]@{
 downloads = $downloads
 downloadsExists = [bool]($downloads -and (Test-Path -LiteralPath $downloads -PathType Container))
 screenshotsKnownFolder = $screenshots
 screenshotsExists = [bool]($screenshots -and (Test-Path -LiteralPath $screenshots -PathType Container))
 picturesFallback = $fallback
 fallbackExists = [bool]($fallback -and (Test-Path -LiteralPath $fallback -PathType Container))
 status = 'Suggestions only. Take a screenshot and confirm where that file appears; third-party tools and clipboard-only captures can differ.'
} | ConvertTo-Json -Compress
