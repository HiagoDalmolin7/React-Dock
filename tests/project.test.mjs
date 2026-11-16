import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const root = new URL('../', import.meta.url);
const read = (name) => fs.readFileSync(new URL(name,root),'utf8');
test('Dock não usa appbar/área reservada',()=>{
  assert.doesNotMatch(read('src-tauri/src/native.rs'), /SHAppBarMessage|ABM_NEW/);
  assert.match(read('src-tauri/src/lib.rs'), /monitor\.size\(\)/);
});
test('Fallback após 3 falhas e sessão interrompida',()=>{
  assert.match(read('src-tauri/src/bridge.rs'),/bridge_failures>=3/);
  assert.match(read('src-tauri/src/bridge.rs'),/was_unclean && settings\.tray_mode=="experimental"/);
  assert.match(read('src-tauri/src/bridge.rs'),/Duration::from_millis\(1600\)/);
});
test('Hook desligado por padrão',()=>{
  assert.match(read('src-tauri/src/settings.rs'),/tray_mode: "stable"\.into\(\)/);
  assert.match(read('src-tauri/src/bridge.rs'),/!path\.is_file\(\)/);
});
test('Sistema inclui interface, bundle e permissões',()=>{
  assert.equal(JSON.parse(read('src-tauri/tauri.conf.json')).app.windows[0].skipTaskbar,true);
  assert.ok(read('src/main.js').includes('renderTray'));
  assert.ok(read('src/main.js').includes('renderQuick'));
});

test('Identidade React Dock consistente em build e interface',()=>{
  const config=JSON.parse(read('src-tauri/tauri.conf.json'));
  assert.equal(config.productName,'React Dock');
  assert.equal(config.app.windows[0].title,'React Dock');
  assert.equal(JSON.parse(read('package.json')).name,'react-dock-liquidglass');
  assert.match(read('src-tauri/Cargo.toml'), /name = "react_dock"/);
  assert.match(read('src-tauri/src/main.rs'), /react_dock::run\(\)/);
  assert.match(read('.github/workflows/build-windows.yml'), /react_dock\.exe/);
});

// O código que causava o erro Win32 E0308 foi eliminado por não ser utilizado.
test('Win32: remoção dos helpers de janela sem uso',()=>{
  const source=read('src-tauri/src/native.rs');
  assert.doesNotMatch(source,/pub fn foreground_exe/);
  assert.doesNotMatch(source,/pub fn exe_for_hwnd/);
  assert.match(source,/hwnd\.parse::<usize>/);
});

// Regressões identificadas em teste no Windows 11: desaparecimento, instâncias e DPI.
test('Dock persistente: janela topmost, recuperação e fechamento protegido',()=>{
  const js=read('src-tauri/src/lib.rs');
  assert.match(js,/tauri_plugin_single_instance::init/);
  assert.match(js,/restore_dock\(&win, true\)/);
  assert.match(js,/CloseRequested\{api/);
  assert.match(js,/api\.prevent_close\(\)/);
  assert.match(js,/std::thread::spawn/);
  const conf=JSON.parse(read('src-tauri/tauri.conf.json'));
  const dock=conf.app.windows[0];
  assert.equal(dock.alwaysOnTop,true);
  assert.equal(dock.transparent,true);
  assert.equal(dock.skipTaskbar,true);
});
test('Vidro: transparência do desktop sem efeito DWM em janela inteira',()=>{
  const dock=JSON.parse(read('src-tauri/tauri.conf.json')).app.windows[0];
  assert.equal(dock.windowEffects,undefined);
  assert.equal(dock.noRedirectionBitmap,undefined);
  assert.match(read('src/style.css'),/background-color:transparent!important/);
});
test('Ícones HiDPI: extração 256px, PNG 160px e SVG como fallback',()=>{
  const native=read('src-tauri/src/native.rs');
  assert.match(native,/PrivateExtractIconsW/);
  assert.match(native,/const N:usize=160/);
  assert.match(read('src/main.js'),/svgPaths/);
  assert.match(read('src/main.js'),/cachedIcons/);
});
test('Versões e workflow modernos sem Actions Node 20',()=>{
  const workflow=read('.github/workflows/build-windows.yml');
  assert.match(workflow,/setup-node@v7/);
  assert.match(workflow,/upload-artifact@v7/);
  assert.match(workflow,/checkout@v7/);
  assert.match(workflow,/node-version: '24'/);
  assert.equal(JSON.parse(read('src-tauri/tauri.conf.json')).version,'4.1.0');
});
