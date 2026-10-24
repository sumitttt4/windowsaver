param([Parameter(Mandatory=$true)][string]$FilePath)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing @'
using System;
using System.IO;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
[StructLayout(LayoutKind.Sequential)] public struct ThumbSize { public int cx; public int cy; }
[ComImport, Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IShellItemImageFactory { [PreserveSig] int GetImage(ThumbSize size, uint flags, out IntPtr bitmap); }
public static class ShellThumbnail {
 [DllImport("shell32.dll", CharSet=CharSet.Unicode, PreserveSig=false)]
 static extern void SHCreateItemFromParsingName(string path, IntPtr context, ref Guid iid, [MarshalAs(UnmanagedType.Interface)] out IShellItemImageFactory item);
 [DllImport("gdi32.dll")] static extern bool DeleteObject(IntPtr handle);
 public static string Read(string path) {
  Guid iid = new Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b"); IShellItemImageFactory item;
  SHCreateItemFromParsingName(path, IntPtr.Zero, ref iid, out item);
  IntPtr handle = IntPtr.Zero;
  try {
   // THUMBNAILONLY: fail explicitly if PDF/image provider is absent; do not misreport an icon as a thumbnail.
   Marshal.ThrowExceptionForHR(item.GetImage(new ThumbSize {cx=512, cy=512}, 8, out handle));
   using (var bitmap = Image.FromHbitmap(handle))
   using (var stream = new MemoryStream()) { bitmap.Save(stream, ImageFormat.Png); return Convert.ToBase64String(stream.ToArray()); }
  } finally { if (handle != IntPtr.Zero) DeleteObject(handle); Marshal.ReleaseComObject(item); }
 }
}
'@
if (!(Test-Path -LiteralPath $FilePath -PathType Leaf)) { throw 'Select an existing file.' }
[Console]::Write([ShellThumbnail]::Read([IO.Path]::GetFullPath($FilePath)))
