// Regras puras da dock: testáveis sem Windows, WebView2 ou Tauri.
export function normalizeKey(path=''){
  return String(path).replaceAll('\\','/').split('/').pop().replace(/\.exe$/i,'').toLowerCase();
}
export function buildDockItems(pinned, running, showRunning=true){
  const groups=new Map();
  for(const w of running){
    const k=normalizeKey(w.exe);
    if(!groups.has(k))groups.set(k,[]);
    groups.get(k).push(w);
  }
  const fixed=pinned.map(p=>{
    const windows=groups.get(normalizeKey(p.target))||[];
    return {...p,pinned:true,windows,window:windows.find(w=>w.active)||windows[0]||null};
  });
  if(!showRunning)return fixed;
  const known=new Set(fixed.map(item=>normalizeKey(item.target)));
  const extra=[];
  for(const [id,windows] of groups){
    if(known.has(id))continue;
    const w=windows.find(item=>item.active)||windows[0];
    extra.push({id:'running:'+id,name:w.process,target:w.exe,icon:'app',pinned:false,window:w,windows});
  }
  return [...fixed,...extra];
}
export function dockWidth(appCount,iconSize,logicalWidth,percent){
  const max=Math.max(260,logicalWidth-12);
  const natural=Math.ceil(appCount*(iconSize+14)+265);
  const asked=percent?Math.round(max*percent/100):Math.max(380,natural);
  return Math.min(max,Math.max(260,asked));
}
export function waveAt(dx,dy){
  const proximity=Math.max(0,1-Math.abs(dx)/155)*Math.max(.35,1-Math.abs(dy)/160);
  return {scale:1+.32*proximity,lift:Math.min(30,(8+Math.max(0,-dy)*.34)*proximity)};
}
