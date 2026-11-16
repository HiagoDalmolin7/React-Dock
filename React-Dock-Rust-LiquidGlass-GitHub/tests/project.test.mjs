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
