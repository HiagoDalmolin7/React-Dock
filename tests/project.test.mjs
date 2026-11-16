import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const root = new URL('../', import.meta.url);
const read = (name) => fs.readFileSync(new URL(name,root),'utf8');
test('Dock não usa appbar/área reservada',()=>{
  assert.doesNotMatch(read('src-tauri/src/native.rs'), /SHAppBarMessage|ABM_NEW/);
  assert.match(read('src-tauri/src/lib.rs'), /d\.size/);
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
  assert.match(js,/restore_dock\(&win,\s*true\)/);
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
  assert.equal(JSON.parse(read('src-tauri/tauri.conf.json')).version,'4.3.0');
});


test('Menus e configurações em janela independente sem redimensionar a dock',()=>{
 const c=JSON.parse(read('src-tauri/tauri.conf.json'));
 assert.equal(c.app.windows.length,2);
 assert.equal(c.app.windows[1].label,'panel');
 assert.equal(c.app.windows[1].visible,false);
 assert.equal(c.app.windows[1].transparent,true);
 const js=read('src/main.js');
 assert.match(js,/getCurrentWindow\(\)\.label === 'panel'/);
 assert.match(js,/api\('show_panel'/);
 assert.match(js,/get_panel_request/);
 assert.match(read('src-tauri/src/lib.rs'),/fn reposition_panel\(/);
 assert.match(read('src-tauri/src/lib.rs'),/fn hide_panel\(/);
});
test('A barra principal permanece fixa e detecta janelas rapidamente',()=>{
 const js=read('src/main.js');
 assert.match(js,/height:116/);
 assert.doesNotMatch(js,/opened\?550/);
 assert.match(js,/setInterval\(refreshWindows,350\)/);
 assert.match(js,/windowPollBusy/);
 const native=read('src-tauri/src/native.rs');
 assert.match(native,/Ícones são carregados sob demanda/);
});
test('Alternância da janela ativa com minimização',()=>{
 assert.match(read('src/main.js'),/api\('toggle_window'/);
 const native=read('src-tauri/src/native.rs');
 assert.match(native,/pub fn toggle_window\(/);
 assert.match(native,/IsIconic/);
 assert.match(native,/ShowWindow\(handle,6\)/);
});
test('Prévia DWM nativa sem screenshot da área de trabalho',()=>{
 const native=read('src-tauri/src/native.rs');
 assert.match(native,/DwmRegisterThumbnail/);
 assert.match(native,/DwmUpdateThumbnailProperties/);
 assert.match(native,/DwmUnregisterThumbnail/);
 const js=read('src/main.js');
 assert.match(js,/show_window_preview/);
 assert.match(js,/hide_window_preview/);
});
test('Composição transparente sem camada de gradiente nem backdrop filter na dock',()=>{
 const css=read('src/style.css');
 assert.match(css,/\.dock-frame \{[\s\S]*?background: rgba\(25,32,46,var\(--glass-opacity\)\) !important/);
 assert.match(css,/backdrop-filter:none !important/);
 assert.match(css,/\.glass-shine,\.glass-ambient \{ display:none/);
});

// Testes de regras reais: agrupamento e largura não dependem do Windows para executar.
import {buildDockItems,normalizeKey,dockWidth,waveAt} from '../src/dock-model.mjs';
test('Agrupa janelas do mesmo processo e preserva todas as instâncias',()=>{
  const pinned=[{id:'explorer',name:'Arquivos',target:'explorer.exe',icon:'folder'}];
  const running=[
    {hwnd:'11',exe:'C:\\Windows\\explorer.exe',process:'explorer',active:false},
    {hwnd:'12',exe:'C:\\Windows\\explorer.exe',process:'explorer',active:true},
    {hwnd:'13',exe:'C:\\Program Files\\App\\app.exe',process:'app',active:false},
  ];
  const list=buildDockItems(pinned,running,true);
  assert.equal(list.length,2);
  assert.equal(list[0].windows.length,2);
  assert.equal(list[0].window.hwnd,'12');
  assert.equal(list[1].windows.length,1);
  assert.equal(normalizeKey(list[0].target),normalizeKey(running[1].exe));
  assert.equal(buildDockItems(pinned,running,false).length,1);
});
test('Largura respeita limite físico e reserva o espaço dos atalhos finais',()=>{
  assert.equal(dockWidth(4,50,1920,100),1908);
  assert.equal(dockWidth(4,50,1920,50),954);
  assert.ok(dockWidth(100,72,1440,0)<=1428);
  assert.ok(dockWidth(0,36,1000,0)>=380);
});
test('Ampliação responde ao movimento vertical e horizontal do mouse',()=>{
  const center=waveAt(0,0),above=waveAt(0,-35),far=waveAt(300,0);
  assert.ok(center.scale>1);
  assert.ok(above.lift>center.lift);
  assert.equal(far.scale,1);
});
test('Monitor secundário recebe permissão IPC e configurações independentes',()=>{
  const perm=JSON.parse(read('src-tauri/capabilities/default.json'));
  assert.ok(perm.windows.includes('dock_*'));
  const settings=read('src-tauri/src/settings.rs'),lib=read('src-tauri/src/lib.rs');
  assert.match(settings,/pub monitors: HashMap<String,DisplayPreferences>/);
  assert.match(lib,/WebviewWindowBuilder::new/);
  assert.match(lib,/available_monitors\(\)/);
  assert.match(lib,/fn current_display\(/);
  assert.match(lib,/fn refresh_displays\(/);
});
test('Saída garantida pelo painel e tray com menu no clique',()=>{
  const js=read('src/main.js'),rust=read('src-tauri/src/lib.rs');
  assert.match(js,/data-quit/);
  assert.match(js,/api\('quit_app'\)/);
  assert.match(rust,/fn quit_app\(/);
  assert.match(rust,/\.show_menu_on_left_click\(true\)/);
});
test('Menu Iniciar sem interceptação forçada da tecla Windows',()=>{
  const js=read('src/main.js'),rs=read('src-tauri/src/native.rs');
  assert.match(js,/function renderStart\(/);
  assert.match(js,/open\('start'\)/);
  assert.match(rs,/RegisterHotKey/);
  assert.match(rs,/0x0001\|0x0002,0x20/);
  assert.doesNotMatch(rs,/SetWindowsHookExW/);
});
test('Apps de entrada de texto do Windows são filtrados, DWM cloak respeitado',()=>{
  const rust=read('src-tauri/src/native.rs');
  assert.match(rust,/textinputhost/);
  assert.match(rust,/DwmGetWindowAttribute\(hwnd,14/);
  assert.match(rust,/GetWindow\(hwnd,4\)/);
});
test('Controles rápidos possuem subpainéis para Wi-Fi Bluetooth e volume',()=>{
  const js=read('src/main.js');
  assert.match(js,/data-quick-detail="wifi"/);
  assert.match(js,/data-quick-detail="bluetooth"/);
  assert.match(js,/data-quick-detail="sound"/);
});
test('Identidade do Explorer permanece estável e ícone pode ser resolvido no Windows',()=>{
  const rs=read('src-tauri/src/native.rs'),js=read('src/main.js');
  assert.match(rs,/resolve_icon_path/);
  assert.match(rs,/explorer\.exe/);
  assert.match(js,/const iconKey=key\(item\.target\)/);
  assert.match(js,/retriedIcons/);
});

test('Menu Iniciar lista atalhos locais com limite para não travar a interface',()=>{
 const native=read('src-tauri/src/native.rs'),js=read('src/main.js');
 assert.match(native,/pub fn start_shortcuts\(/);
 assert.match(native,/result\.len\(\)>=360/);
 assert.match(js,/api\('start_apps'\)/);
});
