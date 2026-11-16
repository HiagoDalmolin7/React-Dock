import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './style.css';
import {buildDockItems, normalizeKey, dockWidth, waveAt} from './dock-model.mjs';

const $ = (id) => document.getElementById(id);
const isPanel = getCurrentWindow().label === 'panel';
document.body.classList.toggle('is-panel', isPanel);
let panelSerial=-1, previewTimeout=null, windowPollBusy=false, lastExternalHwnd=null;
const htmlEscape = (s='') => String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;').replaceAll("'",'&#39;');
let settings, running = [], battery = null, bridge = null, bridgeIcons = [];
let opened = '', currentEdit = '', messageTimeout = null, lastWinSignature='';
let displays=[], currentDisplay=null, quickDetail='', installedApps=[], startCurrentEntries=[];
const cachedIcons = new Map(), pendingIcons = new Set(), retriedIcons = new Set();
const api = (name, payload={}) => invoke(name,payload);

const svgPaths={
    folder: "<path d=\"M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v10H3z\"/>",
    globe: "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M3 12h18M12 3a15 15 0 0 1 0 18M12 3a15 15 0 0 0 0 18\"/>",
    settings: "<path d=\"M12 3v2m0 14v2M3 12h2m14 0h2m-15.4-6.4 1.4 1.4m10 10 1.4 1.4m0-12.8-1.4 1.4m-10 10-1.4 1.4\"/><circle cx=\"12\" cy=\"12\" r=\"4\"/>",
    shield: "<path d=\"M12 2l9 4v6c0 5-3.5 8.5-9 10-5.5-1.5-9-5-9-10V6z\"/><path d=\"m8 12 3 3 5-6\"/>",
    cpu: "<rect x=\"6\" y=\"6\" width=\"12\" height=\"12\" rx=\"2\"/><path d=\"M9 2v4m6-4v4M9 18v4m6-4v4M2 9h4M18 9h4M2 15h4m12 0h4\"/>",
    app: "<rect x=\"3\" y=\"3\" width=\"18\" height=\"18\" rx=\"5\"/><path d=\"M8 8h3v3H8zm5 0h3v3h-3zm-5 5h3v3H8zm5 0h3v3h-3z\"/>",
    wifi: "<path d=\"M2 8a16 16 0 0 1 20 0M5 12a11 11 0 0 1 14 0m-11 4a6 6 0 0 1 8 0\"/><circle cx=\"12\" cy=\"20\" r=\"1\" fill=\"currentColor\" stroke=\"none\"/>",
    bluetooth: "<path d=\"m7 7 10 10-5 5V2l5 5L7 17\"/>",
    volume: "<path d=\"M4 9v6h4l5 4V5L8 9H4m12-2a7 7 0 0 1 0 10m2-13a11 11 0 0 1 0 16\"/>",
    bell: "<path d=\"M18 8a6 6 0 0 0-12 0c0 8-3 8-3 10h18c0-2-3-2-3-10zM10 21h4\"/>",
    clock: "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 7v5l3 2\"/>",
    battery: "<rect x=\"2\" y=\"7\" width=\"19\" height=\"10\" rx=\"2\"/><path d=\"M23 10v4\"/>",
    menu: "<circle cx=\"5\" cy=\"12\" r=\"1\"/><circle cx=\"12\" cy=\"12\" r=\"1\"/><circle cx=\"19\" cy=\"12\" r=\"1\"/>",
    monitor: "<rect x=\"3\" y=\"4\" width=\"18\" height=\"13\" rx=\"2\"/><path d=\"M9 21h6m-3-4v4\"/>",
    power: "<path d=\"M12 2v10M6 5a9 9 0 1 0 12 0\"/>",
    moon: "<path d=\"M20 15A8 8 0 0 1 9 4 8 8 0 1 0 20 15z\"/>",
    airplane: "<path d=\"m22 2-8 20-3-9-9-3z\"/>",
    pin: "<path d=\"M12 21v-8M6 4h12l-2 8 3 3H5l3-3z\"/>",
    terminal: "<rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"2\"/><path d=\"m7 9 3 3-3 3m5 0h5\"/>",
    mute: "<path d=\"M4 9v6h4l5 4V5L8 9H4m5-2 6 6m0-6-6 6\"/>",
    search: "<circle cx=\"10\" cy=\"10\" r=\"7\"/><path d=\"m15 15 6 6\"/>",
    tray: "<rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"2\"/><path d=\"M3 13h5l2 3h4l2-3h5\"/>",
    add: "<path d=\"M12 5v14M5 12h14\"/>",
    caret: "<path d=\"m6 15 6-6 6 6\"/>"
};
function glyph(type){
  const path=svgPaths[type] || svgPaths.app;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" aria-hidden="true">${path}</svg>`;
}

function toast(text,bad=false) {
  const el=$('toast'); el.textContent=text; el.className=`toast visible ${bad?'bad':''}`;
  clearTimeout(messageTimeout); messageTimeout=setTimeout(()=>el.classList.remove('visible'),4300);
}
function accent(){
  document.documentElement.style.setProperty('--accent',settings.accent);
  document.documentElement.style.setProperty('--glass-opacity',(settings.opacity/100).toFixed(2));
  document.documentElement.style.setProperty('--icon-size',settings.iconSize+'px');
  document.body.classList.toggle('reduced',settings.reduceMotion);
}
// Uma entrada por aplicativo e seleção de instâncias no painel próprio.
function items(){return buildDockItems(settings.pinned,running,settings.showRunning);}
const key=normalizeKey;
function findWindow(target){return running.find(w=>key(w.exe)===key(target))||null;}
function appIcon(item){
  const iconKey=key(item.target);
  // Uma identidade visual única mesmo quando o caminho do executável muda após iniciar.
  const src=cachedIcons.get(iconKey)||null;
  const target=item.window?.exe || item.target;
  const needsRetry=cachedIcons.has(iconKey)&&cachedIcons.get(iconKey)===null&&item.window?.exe&&!retriedIcons.has(iconKey);
  if((!cachedIcons.has(iconKey)||needsRetry) && !pendingIcons.has(iconKey) && target && !/^(ms-settings|https?|windowsdefender):/i.test(target)){
    if(needsRetry)retriedIcons.add(iconKey);
    pendingIcons.add(iconKey);
    api('icon_for_path',{target}).then(data=>{
      cachedIcons.set(iconKey,data||null);
      if(isPanel){if(opened==='edit'||opened==='windows')renderPanel();}
      else renderDock();
    }).catch(()=>cachedIcons.set(iconKey,null)).finally(()=>pendingIcons.delete(iconKey));
  }
  return src ? `<img alt="" draggable="false" src="${htmlEscape(src)}"/>` : `<span class="glyph">${glyph(item.icon)}</span>`;
}
function badgeFor(item){const n=settings.attention?.[key(item.target)]||0;return n>0?`<span class="badge">${n>99?'99+':n}</span>`:'';}
function dockApp(item,index){
  const active=item.window?.active, count=item.windows?.length||0;
  return `<button class="app-icon ${active?'active':''} ${settings.attention?.[key(item.target)]?'attention':''}" data-item="${index}" aria-label="${htmlEscape(item.name)}" title="${htmlEscape(item.name)}">
    <div class="icon-art">${appIcon(item)}${badgeFor(item)}</div><i class="running-dot ${count?'running':''}"></i>
    ${count>1?`<small class="instance-count">${count}</small>`:''}</button>`;
}
function renderDock(){
  if(isPanel)return;
  // Ao reconstruir os ícones, cancelar prévias associadas a elementos antigos.
  if(previewTimeout!==null){clearTimeout(previewTimeout);previewTimeout=null;api('hide_window_preview').catch(()=>{});}
  const list=items();
  const apps=$('app-list');apps.innerHTML=list.map(dockApp).join('');
  apps.querySelectorAll('[data-item]').forEach(button=>{
    const item=list[Number(button.dataset.item)];
    button.addEventListener('click',()=>activateApp(item,button));
    button.addEventListener('contextmenu',e=>{e.preventDefault(); editApp(item);});
    button.addEventListener('pointerenter',()=>{
      clearTimeout(previewTimeout);
      if(item.window?.hwnd){
        previewTimeout=setTimeout(()=>api('show_window_preview',{hwnd:item.window.hwnd}).catch(()=>{}),260);
      }
    });
    button.addEventListener('pointerleave',()=>{
      clearTimeout(previewTimeout);previewTimeout=null;
      api('hide_window_preview').catch(()=>{});
    });
  });
  const clock=$('clock');clock.textContent=new Date().toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'});
  clock.style.display=settings.showClock?'':'none';
  $('battery-level').textContent=battery?.percent!=null?`${battery.percent}%`:'--';
  $('battery-icon').textContent=battery?.charging?'ϟ':'▰';
  resize();
}
async function activateApp(item,button){
  try {
    const windows=item.windows||[];
    clearTimeout(previewTimeout);
    await api('hide_window_preview').catch(()=>{});
    if(windows.length>1){
      await api('show_panel',{page:'windows',target:item.target});return;
    }
    const open=windows[0]||item.window||findWindow(item.target);
    if(open){
      // Anima o ícone em direção à sua posição; não manipula pixels da janela alheia.
      if(button && !settings.reduceMotion && (open.active||lastExternalHwnd===open.hwnd)){
        button.classList.add('icon-minimize');
        await new Promise(resolve=>setTimeout(resolve,150));
      }
      await api('toggle_window',{hwnd:open.hwnd,wasActive:!!open.active || lastExternalHwnd===open.hwnd});
      button?.classList.remove('icon-minimize');
    } else {
      button?.classList.add('icon-launch');
      await api('launch',{target:item.target});
      setTimeout(()=>button?.classList.remove('icon-launch'),400);
    }
    if(settings.attention?.[key(item.target)]){await api('clear_attention',{exe:key(item.target)});delete settings.attention[key(item.target)];}
  }catch(err){button?.classList.remove('icon-minimize');toast(String(err),true);}
}
function renderPanel(){
  const panel=$('panel');panel.classList.toggle('hidden',!opened);
  if(!opened)return;
  const names={tray:'Bandeja personalizada',quick:'Controles rápidos',settings:'Aparência e preferências',edit:'Gerenciar aplicativo',windows:'Janelas abertas',start:'Iniciar · React Dock'};
  $('panel-title').textContent=names[opened]||'React Dock';
  document.body.classList.toggle('start-open',opened==='start');
  $('tab-tray').classList.toggle('selected',opened==='tray');
  $('tab-quick').classList.toggle('selected',opened==='quick');
  $('tab-settings').classList.toggle('selected',opened==='settings');
  const content=$('panel-body');
  if(opened==='tray')content.innerHTML=renderTray();
  if(opened==='quick')content.innerHTML=renderQuick();
  if(opened==='settings')content.innerHTML=renderSettings();
  if(opened==='edit')content.innerHTML=renderEdit();
  if(opened==='windows')content.innerHTML=renderWindows();
  if(opened==='start')content.innerHTML=renderStart();
  installActions(content);
  if(opened==='start')$('start-search')?.focus();
}
function renderTray(){
  return `<p class="help">Itens fixados por você. Adicione aplicativos, atalhos e painéis do Windows sem precisar abrir a bandeja original.</p>
  <section class="tray-grid">${settings.trayPinned.map((item,i)=>`<div class="tray-item">
    <button class="tray-open" data-launch-tray="${i}"><span>${glyph(item.icon)}</span>${htmlEscape(item.name)}</button>
    <button class="tiny" title="Remover" data-remove-tray="${i}">×</button></div>`).join('')||'<p>Nenhum item fixado.</p>'}</section>
  <div class="form-row"><input id="new-tray-name" placeholder="Nome, ex.: Steam" maxlength="45"/>
  <input id="new-tray-target" placeholder="Caminho, executável ou protocolo" maxlength="500"/>
  <button class="action" data-add-tray>Adicionar</button></div>
  <div class="section-title">Ícones adicionais do Explorer</div>
  <div class="status-line"><span class="status ${bridge?.mode==='experimental'?'warn':''}">${bridge?.mode==='experimental'?'Experimental':'Estável'}</span><span>${htmlEscape(bridge?.detail||'Usando a bandeja personalizada')}</span></div>
  ${bridge?.mode==='experimental'?`<div class="tray-grid">${bridgeIcons.map(i=>`<div class="tray-item"><span class="symbol">${i.icon?`<img src="${htmlEscape(i.icon)}" alt=""/>`:glyph('tray')}</span><span>${htmlEscape(i.name)}</span></div>`).join('')||'<small>Sem ícones adicionais.</small>'}</div>`:''}
  <p class="micro">Modo experimental requer extensão externa confiável. Os ícones adicionais são somente leitura nesta versão.</p>
  <button class="secondary" data-quick="quick">Abrir Central de configurações rápidas do Windows</button>`;
}
const actions=[['wifi','Wi-Fi','wifi'],['bluetooth','Bluetooth','bluetooth'],['airplane','Modo avião','airplane'],['sound','Som','volume'],['notifications','Notificações','bell'],['focus','Não perturbe','moon'],['display','Tela','monitor'],['battery','Energia','battery']];
function renderQuick(){
 if(quickDetail){
   const info={wifi:['Wi-Fi','wifi','Abrir as redes Wi-Fi do Windows'],bluetooth:['Bluetooth','bluetooth','Abrir dispositivos Bluetooth'],sound:['Volume','sound','Abrir o mixer de volume do Windows']};
   const [label,action,description]=info[quickDetail]||info.sound;
   return `<button class="secondary" data-quick-back>← Voltar</button><h3>${label}</h3><p class="help">${description}</p>
     ${quickDetail==='sound'?'<div class="volume-row"><button data-quick="volume-down">− Volume</button><button data-quick="volume-mute">Silenciar</button><button data-quick="volume-up">+ Volume</button></div>':''}
     <button class="action" data-quick="${action}">Abrir configuração oficial</button><p class="micro">A dock não inventa estados de Wi-Fi/Bluetooth: as opções detalhadas são administradas pelo Windows.</p>`;
 }
 return `<div class="tiles">
   <button data-quick-detail="wifi" class="tile"><b>${glyph('wifi')}</b><span>Wi-Fi</span></button>
   <button data-quick-detail="bluetooth" class="tile"><b>${glyph('bluetooth')}</b><span>Bluetooth</span></button>
   <button data-quick-detail="sound" class="tile"><b>${glyph('volume')}</b><span>Volume</span></button>
   ${actions.filter(a=>!['wifi','bluetooth','sound'].includes(a[0])).map(([id,label,symbol])=>`<button data-quick="${id}" class="tile"><b>${glyph(symbol)}</b><span>${label}</span></button>`).join('')}
   </div><button class="secondary" data-quick="quick">Painel completo do Windows (Win+A)</button>`;
}
function renderWindows(){
 const list=running.filter(w=>key(w.exe)===key(currentEdit));
 return `<p class="help">${list.length} janela(s) deste programa. Escolha individualmente qual deseja ativar ou minimizar.</p>
 <div class="window-picker">${list.map(w=>`<div class="window-line"><span>${glyph('monitor')} ${htmlEscape(w.title)}</span>
 <div><button class="secondary" data-window-focus="${w.hwnd}">Abrir</button><button class="secondary" data-window-min="${w.hwnd}">Minimizar</button></div></div>`).join('')||'<p>Não há janelas abertas.</p>'}</div>`;
}
function startEntries(){
 const seen=new Set();
 return [...settings.pinned,...settings.trayPinned,...running.map(w=>({name:w.process,target:w.exe,icon:'app'})),...installedApps.map(p=>({...p,icon:'app',catalog:true}))].filter(i=>{
   const k=key(i.target);if(seen.has(k))return false;seen.add(k);return true;
 });
}
function renderStart(){
 const entries=startEntries();startCurrentEntries=entries;
 return `<div class="start-search-row">${glyph('search')}<input id="start-search" placeholder="Pesquisar programas..." autocomplete="off"/></div>
 <div class="section-title">Aplicativos</div><div class="start-grid">${entries.map((p,i)=>`<button class="start-item" data-start-target="${i}" data-search="${htmlEscape(p.name.toLowerCase())}">${p.catalog?`<span class="glyph">${glyph('app')}</span>`:appIcon({...p,window:findWindow(p.target)})}<span>${htmlEscape(p.name)}</span></button>`).join('')}</div>
 <div class="section-title">Atalhos do Windows</div><div class="button-row"><button class="secondary" data-start-system="explorer.exe">Arquivos</button><button class="secondary" data-start-system="ms-settings:">Configurações</button><button class="secondary" data-start-system="taskmgr.exe">Gerenciador</button></div>
 <p class="micro">Iniciar personalizado: ícone do Windows na dock ou Ctrl+Alt+Espaço em qualquer aplicativo. A tecla Windows isolada permanece com o Iniciar original.</p>`;
}
function range(name,title,min,max,unit=''){return `<label class="range"><span>${title}<strong id="v-${name}">${settings[name]}${unit}</strong></span><input type="range" min="${min}" max="${max}" value="${settings[name]}" data-setting="${name}" data-unit="${unit}" /></label>`;}
function renderSettings(){
 const checked=(key)=>settings[key]?'checked':'';
 return `<div class="preferences">
 ${range('iconSize','Tamanho dos ícones',36,72,' px')}
 ${range('opacity','Opacidade do vidro',6,90,'%')}
 <label class="color-field">Cor de destaque <input type="color" data-setting="accent" value="${htmlEscape(settings.accent)}" /></label>
 <label class="switch"><input type="checkbox" data-setting="showRunning" ${checked('showRunning')}> Mostrar aplicativos abertos</label>
 <label class="switch"><input type="checkbox" data-setting="showClock" ${checked('showClock')}> Mostrar relógio</label>
 <label class="switch"><input type="checkbox" data-setting="reduceMotion" ${checked('reduceMotion')}> Reduzir animações</label>
 <hr/><div class="section-title">Monitores</div>
 <p class="help">Configurações independentes de visibilidade, largura e posição por tela. Largura 0% = automática; 100% = comprimento completo.</p>
 ${displays.map(d=>{
   const p=settings.monitors?.[d.key]||{enabled:true,widthPercent:0,bottomGap:8};
   const id=htmlEscape(d.key);
   return `<div class="monitor-pref"><strong>${htmlEscape(d.name)} · ${d.logicalWidth}px</strong>
   <label class="switch"><input type="checkbox" data-display-setting="enabled" data-display-key="${id}" ${p.enabled?'checked':''}> Mostrar dock neste monitor</label>
   <label class="range"><span>Largura <strong class="display-value">${p.widthPercent?p.widthPercent+'%':'Automática'}</strong></span><input type="range" min="0" max="100" step="5" data-display-setting="widthPercent" data-display-key="${id}" value="${p.widthPercent}"/></label>
   <label class="range"><span>Distância inferior <strong class="display-value">${p.bottomGap} px</strong></span><input type="range" min="0" max="90" data-display-setting="bottomGap" data-display-key="${id}" value="${p.bottomGap}"/></label></div>`;
 }).join('')}
 <hr/><div class="section-title">Encerrar programa</div>
 <p class="help">Use este botão para fechar completamente o React Dock, independentemente do ícone na bandeja nativa.</p>
 <button class="danger" data-quit>Encerrar React Dock</button>
 <hr/><div class="section-title">Bandeja experimental</div>
 <p class="help">A extensão do Explorer continua opcional, desativada por padrão, e depende de um auxiliar externo.</p>
 <div class="status-line"><span class="status ${bridge?.mode==='experimental'?'warn':''}">${bridge?.mode==='experimental'?'Experimental':'Estável'}</span><span>${htmlEscape(bridge?.detail||'')}</span></div>
 <div class="button-row"><button class="secondary" data-mode="stable">Usar estável</button><button class="action" data-mode="experimental" ${bridge?.available?'':'disabled'}>Ativar experimental</button></div>
 <hr/><div class="section-title">Aplicativos fixados</div>
 <div class="pinned-editor">${settings.pinned.map((p,i)=>`<div class="pin-line"><span>${glyph(p.icon)} ${htmlEscape(p.name)}</span><div><button class="tiny" data-up-pin="${i}" title="Mover à esquerda">←</button><button class="tiny" data-remove-pin="${i}" title="Remover">×</button></div></div>`).join('')}</div>
 <div class="form-row"><input id="new-pin-name" placeholder="Nome do app" maxlength="45"/><input id="new-pin-target" placeholder="Caminho do .exe ou atalho .lnk" maxlength="500"/><button class="action" data-add-pin>Fixar</button></div>
 </div>`;
}
function editApp(item){
  currentEdit=item?.target||'';
  if(!isPanel){api('show_panel',{page:'edit',target:currentEdit}).catch(e=>toast(String(e),true));return;}
  opened='edit';renderPanel();
}
function renderEdit(){
 const item=items().find(a=>a.target===currentEdit);
 if(!item)return '<p>O aplicativo não está mais disponível.</p>';
 const pinned=!!settings.pinned.find(p=>key(p.target)===key(item.target));
 return `<div class="edit-app"><div class="app-preview">${appIcon(item)}</div><h3>${htmlEscape(item.name)}</h3><p>${htmlEscape(item.target)}</p>
 <button class="action" data-pin-current>${pinned?'Remover da dock':'Fixar na dock'}</button>
 <button class="secondary" data-tray-current>Adicionar à bandeja</button></div>`;
}
async function save(){try{settings=await api('save_settings',{value:settings});accent();if(isPanel){if(opened!=='settings')renderPanel();}else renderDock();}catch(e){toast(String(e),true);}}
function installActions(el){
 el.querySelectorAll('[data-quick-detail]').forEach(b=>b.addEventListener('click',()=>{quickDetail=b.dataset.quickDetail;renderPanel();}));
 el.querySelector('[data-quick-back]')?.addEventListener('click',()=>{quickDetail='';renderPanel();});
 el.querySelector('[data-quit]')?.addEventListener('click',()=>api('quit_app').catch(e=>toast(String(e),true)));
 el.querySelectorAll('[data-display-setting]').forEach(input=>input.addEventListener(input.type==='checkbox'?'change':'input',()=>{
   const key=input.dataset.displayKey;
   settings.monitors||={};settings.monitors[key]||={enabled:true,widthPercent:0,bottomGap:8};
   const field=input.dataset.displaySetting;
   settings.monitors[key][field]=input.type==='checkbox'?input.checked:Number(input.value);
   const val=input.closest('label')?.querySelector('.display-value');
   if(val)val.textContent=field==='widthPercent'?(Number(input.value)?input.value+'%':'Automática'):input.value+' px';
   clearTimeout(window.displayPreferenceDebounce);window.displayPreferenceDebounce=setTimeout(save,260);
 }));
 el.querySelectorAll('[data-window-focus]').forEach(b=>b.addEventListener('click',async()=>{
   await api('focus_window',{hwnd:b.dataset.windowFocus}).catch(e=>toast(String(e),true));
   await api('hide_panel').catch(()=>{});
 }));
 el.querySelectorAll('[data-window-min]').forEach(b=>b.addEventListener('click',async()=>{
   await api('toggle_window',{hwnd:b.dataset.windowMin,wasActive:true}).catch(e=>toast(String(e),true));
   await api('hide_panel').catch(()=>{});
 }));
 el.querySelectorAll('[data-start-target]').forEach(b=>b.addEventListener('click',async()=>{
   const p=startCurrentEntries[Number(b.dataset.startTarget)];if(!p)return;
   const win=findWindow(p.target);
   await api(win?'focus_window':'launch',win?{hwnd:win.hwnd}:{target:p.target}).catch(e=>toast(String(e),true));
   await api('hide_panel').catch(()=>{});
 }));
 el.querySelectorAll('[data-start-system]').forEach(b=>b.addEventListener('click',async()=>{
   await api('launch',{target:b.dataset.startSystem}).catch(e=>toast(String(e),true));
   await api('hide_panel').catch(()=>{});
 }));
 el.querySelector('#start-search')?.addEventListener('input',e=>{
   const q=e.target.value.trim().toLowerCase();
   el.querySelectorAll('[data-start-target]').forEach(b=>b.hidden=!b.dataset.search.includes(q));
 });
 el.querySelectorAll('[data-quick]').forEach(b=>b.addEventListener('click',async()=>{
   try{await api('quick_action',{action:b.dataset.quick});}catch(e){toast(String(e),true);}
 }));
 el.querySelectorAll('[data-setting]').forEach(input=>input.addEventListener(input.type==='range'?'input':'change',()=>{
   const name=input.dataset.setting;
   settings[name]=input.type==='checkbox'?input.checked:input.type==='range'?Number(input.value):input.value;
   const counter=$('v-'+name);if(counter)counter.textContent=String(settings[name])+(input.dataset.unit||'');
   accent();if(input.type==='range')clearTimeout(window.preferenceDebounce);
   window.preferenceDebounce=setTimeout(save,180);
 }));
 el.querySelectorAll('[data-mode]').forEach(b=>b.addEventListener('click',async()=>{
   try{bridge=await api('set_bridge_mode',{mode:b.dataset.mode});toast(bridge.detail);renderPanel();}catch(e){toast(String(e),true);}
 }));
 el.querySelectorAll('[data-launch-tray]').forEach(b=>b.addEventListener('click',async()=>{
   try{await api('launch',{target:settings.trayPinned[Number(b.dataset.launchTray)].target});}catch(e){toast(String(e),true);}
 }));
 el.querySelectorAll('[data-remove-tray]').forEach(b=>b.addEventListener('click',async()=>{
   settings.trayPinned.splice(Number(b.dataset.removeTray),1);await save();renderPanel();
 }));
 el.querySelectorAll('[data-remove-pin]').forEach(b=>b.addEventListener('click',async()=>{
   settings.pinned.splice(Number(b.dataset.removePin),1);await save();renderPanel();
 }));
 el.querySelectorAll('[data-up-pin]').forEach(b=>b.addEventListener('click',async()=>{
   const i=Number(b.dataset.upPin);if(i){[settings.pinned[i-1],settings.pinned[i]]=[settings.pinned[i],settings.pinned[i-1]];await save();renderPanel();}
 }));
 function add(kind){
   const name=$(kind==='tray'?'new-tray-name':'new-pin-name')?.value.trim();
   const target=$(kind==='tray'?'new-tray-target':'new-pin-target')?.value.trim();
   if(!name||!target){toast('Preencha o nome e o executável/caminho.',true);return;}
   const item={id:`user-${Date.now()}`,name,target,icon:'app'};
   if(kind==='tray')settings.trayPinned.push(item);else settings.pinned.push(item);
   save().then(renderPanel);
 }
 el.querySelector('[data-add-tray]')?.addEventListener('click',()=>add('tray'));
 el.querySelector('[data-add-pin]')?.addEventListener('click',()=>add('pin'));
 el.querySelector('[data-pin-current]')?.addEventListener('click',async()=>{
   const item=items().find(a=>a.target===currentEdit);if(!item)return;
   const idx=settings.pinned.findIndex(p=>key(p.target)===key(item.target));
   if(idx>=0)settings.pinned.splice(idx,1);
   else settings.pinned.push({id:`user-${Date.now()}`,name:item.name,target:item.target,icon:item.icon});
   await save();opened='settings';renderPanel();
 });
 el.querySelector('[data-tray-current]')?.addEventListener('click',async()=>{
   const item=items().find(a=>a.target===currentEdit);if(!item)return;
   if(!settings.trayPinned.some(p=>key(p.target)===key(item.target))){
     settings.trayPinned.push({id:`user-${Date.now()}`,name:item.name,target:item.target,icon:item.icon}); await save();
   }
   opened='tray';renderPanel();
 });
}
function open(name){
  if(!isPanel){
    clearTimeout(previewTimeout);
    api('hide_window_preview').catch(()=>{});
    api('show_panel',{page:name}).catch(e=>toast(String(e),true));
    return;
  }
  opened=name;
  if(name==='quick')quickDetail='';
  renderPanel();
}
let layoutDebounce;
function resize(){
 if(!settings||isPanel||!currentDisplay)return;
 clearTimeout(layoutDebounce);
 layoutDebounce=setTimeout(async()=>{
   const available=Math.max(300,currentDisplay.logicalWidth-12);
   const profile=settings.monitors?.[currentDisplay.key]||{widthPercent:0};
   const width=dockWidth(items().length,settings.iconSize,available+12,profile.widthPercent);
   document.documentElement.style.setProperty('--dock-width',Math.max(250,width-18)+'px');
   try{await api('resize_dock',{width,height:116});}catch(e){console.warn('Posicionamento:',e);}
 },50);
}
function installMagnification(){
 const dock=$('dock-frame');
 dock.addEventListener('pointermove',event=>{
   if(settings.reduceMotion)return;
   dock.querySelectorAll('.app-icon').forEach(btn=>{
     const rect=btn.getBoundingClientRect(),cx=rect.left+rect.width/2,cy=rect.top+rect.height*.65;
     const {scale,lift:rise}=waveAt(event.clientX-cx,event.clientY-cy);
     btn.style.setProperty('--magnify',scale.toFixed(3));
     btn.style.setProperty('--lift',Math.min(30,rise).toFixed(1)+'px');
   });
 });
 dock.addEventListener('pointerleave',()=>dock.querySelectorAll('.app-icon').forEach(btn=>{
   btn.style.setProperty('--magnify',1);btn.style.setProperty('--lift','0px');
 }));
}
async function refreshWindows(){
 if(windowPollBusy)return;
 windowPollBusy=true;
 try{
   const list=await api('windows_list');
   const signature=list.map(i=>`${i.hwnd}:${i.active}`).join('|');
   running=list;
   const foreground=list.find(w=>w.active);if(foreground)lastExternalHwnd=foreground.hwnd;
   if(signature!==lastWinSignature){lastWinSignature=signature;if(!isPanel)renderDock();else if(opened==='edit'||opened==='windows')renderPanel();}
 }catch(e){console.warn(e);}finally{windowPollBusy=false;}
}
async function refreshBridge(){
 try{
   bridge=await api('get_bridge_status');
   if(bridge.mode==='experimental')bridgeIcons=await api('tray_snapshot');
 }catch(e){toast(String(e),true);bridge=await api('get_bridge_status').catch(()=>bridge);bridgeIcons=[];}
 if(isPanel&&(opened==='tray'||opened==='settings'))renderPanel();
}
async function checkPanelRequest(){
  if(!isPanel)return;
  try{
    const request=await api('get_panel_request');
    if(request.serial!==panelSerial){
      panelSerial=request.serial;
      opened=request.page;
      currentEdit=request.target||'';
      if(opened==='edit'||opened==='windows'||opened==='start')await refreshWindows();
      if(opened==='start' && !installedApps.length){
        installedApps=await api('start_apps').catch(()=>[]);
      }
      renderPanel();
    }
  }catch(e){console.warn('Painel:',e);}
}
async function syncSettings(){
  if(isPanel)return;
  try{
    const next=await api('get_settings');
    if(JSON.stringify(next)!==JSON.stringify(settings)){
      settings=next;accent();renderDock();
    }
  }catch(e){console.warn(e);}
}
async function init(){
 settings=await api('get_settings');
 displays=await api('get_displays');
 if(!isPanel)currentDisplay=await api('current_display');
 bridge=await api('get_bridge_status');
 battery=await api('battery_status');
 accent();
 if(isPanel){
   // O painel é outra janela WebView: nenhum resize da dock ao abrir controles.
   $('panel-close').addEventListener('click',()=>api('hide_panel').catch(e=>toast(String(e),true)));
   $('tab-tray').addEventListener('click',()=>open('tray'));
   $('tab-quick').addEventListener('click',()=>open('quick'));
   $('tab-settings').addEventListener('click',()=>open('settings'));
   await refreshBridge();
   await checkPanelRequest();
   setInterval(checkPanelRequest,140);
   setInterval(refreshBridge,9000);
   setInterval(async()=>{battery=await api('battery_status').catch(()=>battery);if(opened==='quick')renderPanel();},30000);
   return;
 }
 renderDock();installMagnification();
 for(const [id,page] of [['open-tray','tray'],['open-quick','quick'],['open-settings','settings']])$(id).addEventListener('click',()=>open(page));
 $('home-button').addEventListener('click',()=>open('start'));
 await refreshWindows();
 // 350 ms para aplicativos abertos. Ícones são extraídos sob demanda e guardados em cache.
 setInterval(refreshWindows,350);
 setInterval(syncSettings,700);
 setInterval(async()=>{battery=await api('battery_status').catch(()=>battery);renderDock();},30000);
 setInterval(()=>{if(settings.showClock){$('clock').textContent=new Date().toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'});}},12000);
 if(bridge.detail.includes('sessão anterior'))toast(bridge.detail,true);
}
init().catch(error=>{document.body.innerHTML=`<main class="fatal">Não foi possível iniciar a React Dock: ${htmlEscape(String(error))}</main>`;});
