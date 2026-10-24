// Runs only on an isolated Windows CI runner against the installed release app.
// CDP is enabled through a process-local WebView2 environment variable, never app config.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { spawn, execFileSync } from 'node:child_process';
const [exe, root] = process.argv.slice(2);
assert.equal(process.platform, 'win32');
assert.ok(exe && root);
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(fn, message, timeout = 20000) {
  const end = Date.now() + timeout;
  let error;
  while (Date.now() < end) {
    try { const result = await fn(); if (result) return result; } catch (e) { error = e; }
    await delay(200);
  }
  throw new Error(`${message}${error ? `: ${error.message}` : ''}`);
}
const fixtureStamp = new Date(Date.now() - 60000);
const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAACAAAAAYCAIAAAAUMWhjAAAAJ0lEQVR4nGOs2PeagZaAiaamj1owasGoBaMWjFowasGoBaMWUA0AADvnAlE5P8UQAAAAAElFTkSuQmCC', 'base64');
function pdf() {
  const content = 'BT /F1 24 Tf 30 100 Td (Feed the Pile fixture) Tj ET';
  const objects = ['<< /Type /Catalog /Pages 2 0 R >>', '<< /Type /Pages /Kids [3 0 R] /Count 1 >>', '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 180] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>', '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>', `<< /Length ${content.length} >>\nstream\n${content}\nendstream`];
  let output = '%PDF-1.4\n'; const offsets = [0];
  objects.forEach((o, i) => { offsets.push(Buffer.byteLength(output)); output += `${i + 1} 0 obj\n${o}\nendobj\n`; });
  const start = Buffer.byteLength(output);
  output += `xref\n0 6\n0000000000 65535 f \n${offsets.slice(1).map(n => `${String(n).padStart(10, '0')} 00000 n \n`).join('')}trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n${start}\n%%EOF\n`;
  return output;
}
async function fixture(pile, name, contents) {
  const file = path.join(root, pile, name);
  await fs.writeFile(file, contents);
  await fs.utimes(file, fixtureStamp, fixtureStamp);
  return file;
}
await fixture('downloads', 'a-image.png', png);
await fixture('downloads', 'b-document.pdf', pdf());
await fixture('downloads', 'c-note.txt', 'Local fixture only');
await fixture('screenshots', 'screenshot.png', png);
await fixture('keep', 'a-image.png', 'Existing file must survive');
let child, socket;
const results = [];
async function start() {
  child = spawn(exe, [], { env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9223' }, stdio: 'ignore' });
  child.on('error', e => { console.error(e); });
  const target = await until(async () => {
    const pages = await (await fetch('http://127.0.0.1:9223/json/list')).json();
    return pages.find(p => p.type === 'page' && p.webSocketDebuggerUrl);
  }, 'Installed WebView2 app did not expose a page', 30000);
  socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { socket.addEventListener('open', resolve, { once: true }); socket.addEventListener('error', reject, { once: true }); });
  let serial = 0; const pending = new Map();
  socket.addEventListener('message', ({ data }) => {
    const message = JSON.parse(data); const item = pending.get(message.id);
    if (item) { pending.delete(message.id); message.error ? item.reject(new Error(JSON.stringify(message.error))) : item.resolve(message.result); }
  });
  globalThis.evaluate = async expression => {
    const id = ++serial;
    const result = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { pending.delete(id); reject(new Error('CDP evaluation timed out')); }, 20000);
      pending.set(id, { resolve: r => { clearTimeout(timer); resolve(r); }, reject: e => { clearTimeout(timer); reject(e); } });
      socket.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: { expression, awaitPromise: true, returnByValue: true } }));
    });
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };
  await until(() => evaluate('!!document.querySelector(".file-card")'), 'App failed to render configured piles');
}
function invoke(command, args = {}) { return evaluate(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)})`); }
function key(value) { return evaluate(`window.dispatchEvent(new KeyboardEvent('keydown', {key:${JSON.stringify(value)}}))`); }
async function preview(name) {
  const started = Date.now();
  await until(() => evaluate(`document.querySelector('.file-card h2')?.textContent === ${JSON.stringify(name)} && !!document.querySelector('.file-preview img')?.naturalWidth`), `Missing image/PDF preview: ${name}`);
  results.push({ check: `preview ${name}`, milliseconds: Date.now() - started });
}
function stop() {
  socket?.close();
  if (child?.pid) { try { execFileSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' }); } catch {} }
}
try {
  await start();
  let s = await invoke('snapshot'); assert.equal(s.downloads.length, 3); assert.equal(s.warning, null);
  await preview('a-image.png');
  await key('l');
  await until(() => evaluate('document.querySelector(".file-card h2")?.textContent === "b-document.pdf"'), 'Later did not rotate');
  assert.equal((await invoke('snapshot')).downloads.length, 3);
  await preview('b-document.pdf');
  await key('z'); await preview('a-image.png');
  await key('k');
  await until(async () => (await invoke('snapshot')).downloads.length === 2, 'Keep did not move');
  assert.equal(await fs.readFile(path.join(root, 'keep', 'a-image.png'), 'utf8'), 'Existing file must survive');
  assert.deepEqual(await fs.readFile(path.join(root, 'keep', 'a-image (1).png')), png);
  // Close the installed application, reopen, then undo through the UI.
  stop(); await delay(1000); await start(); await key('z'); await preview('a-image.png');
  assert.deepEqual(await fs.readFile(path.join(root, 'downloads', 'a-image.png')), png);
  await key('b');
  await until(async () => (await invoke('snapshot')).downloads.length === 2, 'Native Bin did not complete');
  stop(); await delay(1000); await start(); await key('z'); await preview('a-image.png');
  assert.deepEqual(await fs.readFile(path.join(root, 'downloads', 'a-image.png')), png);
  await evaluate("[...document.querySelectorAll('[role=tab]')].find(e => e.textContent.includes('Screenshots')).click()");
  await preview('screenshot.png'); await key('c');
  await until(() => evaluate('document.querySelector(".speech-bubble")?.textContent.includes("Copied.")'), 'Copy command failed');
  const clipboard = execFileSync('powershell.exe', ['-NoProfile', '-STA', '-Command', 'Add-Type -AssemblyName System.Windows.Forms; $i=[Windows.Forms.Clipboard]::GetImage(); if (!$i) {exit 1}; Write-Output "$($i.Width)x$($i.Height)"; $i.Dispose()'], { encoding: 'utf8' }).trim();
  assert.equal(clipboard, '32x24');
  // A genuine filesystem event must update the React card count automatically.
  await fixture('screenshots', 'watched.png', png);
  await until(() => evaluate("[...document.querySelectorAll('[role=tab]')].find(e => e.textContent.includes('Screenshots')).textContent.includes('2')"), 'Folder watcher did not refresh UI', 15000);
  const suggestions = await invoke('folder_suggestions'); assert.ok(suggestions.downloads);
  results.push({ check: 'installed app; image + PDF; keyboard Later/Keep/Bin; no overwrite; undo after restart; native clipboard; folder watching; known folders', passed: true });
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile('reports/windows-smoke.json', JSON.stringify({ platform: process.platform, arch: process.arch, results, suggestions }, null, 2));
  console.log(JSON.stringify(results, null, 2));
} finally { stop(); }
