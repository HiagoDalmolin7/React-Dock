#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod native;
mod settings;

use std::{sync::Mutex, collections::HashMap, time::Duration};
use tauri::{AppHandle, Manager, State, WebviewWindow, WebviewWindowBuilder, WebviewUrl};
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};


#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
struct PanelRequest { page: String, target: String, source_label:String, serial: u64 }

// As configurações de cada monitor são identificadas por posição e resolução físicas.
#[derive(Clone)]
struct Display { key:String, name:String, pos:tauri::PhysicalPosition<i32>, size:tauri::PhysicalSize<u32>, scale:f64 }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct DisplayInfo { label:String, key:String, name:String, enabled:bool, width_percent:u8, bottom_gap:u32, logical_width:u32 }
fn display_key(m:&tauri::Monitor)->String{
    let p=m.position();let s=m.size();format!("{}:{}:{}:{}",p.x,p.y,s.width,s.height)
}
fn displays(app:&AppHandle)->Vec<DisplayInfo>{
    let state=app.state::<AppState>();
    let map=state.displays.lock().unwrap();
    let settings=state.settings.lock().unwrap();
    let mut list=map.iter().map(|(label,d)|{
        let profile=settings.monitors.get(&d.key).cloned().unwrap_or_default();
        DisplayInfo{label:label.clone(),key:d.key.clone(),name:d.name.clone(),enabled:profile.enabled,width_percent:profile.width_percent,bottom_gap:profile.bottom_gap,logical_width:(d.size.width as f64/d.scale).round() as u32}
    }).collect::<Vec<_>>();
    list.sort_by_key(|d|if d.label=="dock" {0}else{d.label.trim_start_matches("dock_").parse::<usize>().unwrap_or(99)+1});
    list
}
#[tauri::command]
fn get_displays(app:AppHandle)->Vec<DisplayInfo>{displays(&app)}
#[tauri::command]
fn current_display(window:WebviewWindow,app:AppHandle)->Option<DisplayInfo>{
    let label=window.label().to_string();displays(&app).into_iter().find(|d|d.label==label)
}
fn position_dock(window:&WebviewWindow, d:&Display, gap:u32)->Result<(),String>{
    let outer=window.outer_size().map_err(|e|e.to_string())?;
    let bottom=((gap as f64)*d.scale).round() as i32;
    let x=d.pos.x+(d.size.width as i32-outer.width as i32)/2;
    let y=d.pos.y+d.size.height as i32-outer.height as i32-bottom;
    window.set_position(tauri::PhysicalPosition::new(x,y)).map_err(|e|e.to_string())
}
fn refresh_displays(app:&AppHandle)->Result<(),String>{
    let state=app.state::<AppState>();
    let map=state.displays.lock().map_err(|e|e.to_string())?.clone();
    let settings=state.settings.lock().map_err(|e|e.to_string())?.clone();
    for (label,d) in &map {
        if let Some(window)=app.get_webview_window(label){
            let p=settings.monitors.get(&d.key).cloned().unwrap_or_default();
            if !p.enabled {let _=window.hide();continue;}
            if !window.is_visible().unwrap_or(true){let _=window.show();}
            let _=position_dock(&window,d,p.bottom_gap);
        }
    }
    Ok(())
}
fn reposition_panel(app:&AppHandle,origin:&str)->Result<(),String>{
    let dock=app.get_webview_window(origin).or_else(||app.get_webview_window("dock")).ok_or("Dock não encontrada")?;
    let panel=app.get_webview_window("panel").ok_or("Painel não encontrado")?;
    let pos=dock.outer_position().map_err(|e|e.to_string())?;
    let sz=dock.outer_size().map_err(|e|e.to_string())?;
    let ps=panel.outer_size().map_err(|e|e.to_string())?;
    let m=dock.current_monitor().map_err(|e|e.to_string())?.or_else(||dock.primary_monitor().ok().flatten()).ok_or("Monitor indisponível")?;
    let point=m.position();let size=m.size();
    let min_x=point.x+8;let max_x=(point.x+size.width as i32-ps.width as i32-8).max(min_x);
    let x=(pos.x+(sz.width as i32-ps.width as i32)/2).clamp(min_x,max_x);
    let y=(pos.y-ps.height as i32-10).max(point.y+8);
    panel.set_position(tauri::PhysicalPosition::new(x,y)).map_err(|e|e.to_string())
}
fn open_panel(app:&AppHandle,page:&str,target:&str,origin:&str)->Result<(),String>{
    if !matches!(page,"tray"|"quick"|"settings"|"edit"|"windows"|"start") {return Err("Página inválida".into());}
    let state=app.state::<AppState>();
    {
        let mut request=state.panel_request.lock().map_err(|e|e.to_string())?;
        request.page=page.to_string();request.target=target.to_string();request.source_label=origin.to_string();request.serial=request.serial.wrapping_add(1);
    }
    reposition_panel(app,origin)?;
    let panel=app.get_webview_window("panel").ok_or("Painel não encontrado")?;
    panel.show().map_err(|e|e.to_string())?;
    let _=panel.set_always_on_top(true);
    panel.set_focus().map_err(|e|e.to_string())?;
    Ok(())
}
#[tauri::command]
fn show_panel(app:AppHandle,window:WebviewWindow,page:String,target:Option<String>)->Result<(),String>{
    let _=app.run_on_main_thread(||native::hide_preview());
    open_panel(&app,&page,target.as_deref().unwrap_or(""),window.label())
}
#[tauri::command]
fn hide_panel(app:AppHandle)->Result<(),String>{
    if let Some(panel)=app.get_webview_window("panel") {panel.hide().map_err(|e|e.to_string())?;}
    Ok(())
}
#[tauri::command]
fn quit_app(app:AppHandle){bridge::end_session();app.exit(0);}

#[tauri::command]
fn get_panel_request(state:State<'_,AppState>)->Result<PanelRequest,String>{
    Ok(state.panel_request.lock().map_err(|e|e.to_string())?.clone())
}
#[tauri::command]
fn toggle_window(hwnd:String,was_active:bool)->Result<(),String>{ native::toggle_window(&hwnd,was_active) }
#[tauri::command]
fn show_window_preview(app:AppHandle,window:WebviewWindow,hwnd:String)->Result<(),String>{
    let id=hwnd.parse::<usize>().map_err(|_|"Identificador de janela inválido")?;
    let y=window.outer_position().map_err(|e|e.to_string())?.y;
    app.run_on_main_thread(move|| { native::show_preview(id,y); }).map_err(|e|e.to_string())
}
#[tauri::command]
fn hide_window_preview(app:AppHandle)->Result<(),String>{
    app.run_on_main_thread(|| native::hide_preview()).map_err(|e|e.to_string())
}

struct AppState {
    settings: Mutex<settings::Settings>,
    recovery_note: Mutex<String>,
    panel_request: Mutex<PanelRequest>,
    displays: Mutex<HashMap<String,Display>>,
}

#[tauri::command]
fn get_settings(state:State<'_,AppState>)->Result<settings::Settings,String>{
    Ok(state.settings.lock().map_err(|e|e.to_string())?.clone())
}
#[tauri::command]
fn save_settings(app:AppHandle,state:State<'_,AppState>,mut value:settings::Settings)->Result<settings::Settings,String>{
    let mut lock=state.settings.lock().map_err(|e|e.to_string())?;
    // Não aceitar mudança de modo experimental pela operação genérica.
    value.tray_mode=lock.tray_mode.clone();value.bridge_failures=lock.bridge_failures;
    settings::validate(&mut value);settings::save(&value)?;
    *lock=value.clone();
    drop(lock);
    refresh_displays(&app)?;
    Ok(value)
}
#[tauri::command]
fn windows_list()->Vec<native::WindowItem>{native::windows_list()}
#[tauri::command]
fn start_apps()->Vec<native::StartShortcut>{native::start_shortcuts()}
#[tauri::command]
fn focus_window(hwnd:String)->Result<(),String>{native::focus_window(&hwnd)}
#[tauri::command]
fn launch(target:String)->Result<(),String>{native::launch(&target)}
#[tauri::command]
fn icon_for_path(target:String)->Option<String>{native::extract_icon(&target)}
#[tauri::command]
fn battery_status()->native::Battery{native::battery()}
#[tauri::command]
fn quick_action(action:String)->Result<(),String>{
    match action.as_str(){
        "wifi"=>native::launch("ms-settings:network-wifi"),
        "bluetooth"=>native::launch("ms-settings:bluetooth"),
        "airplane"=>native::launch("ms-settings:network-airplanemode"),
        "ethernet"=>native::launch("ms-settings:network-ethernet"),
        "sound"=>native::launch("ms-settings:sound"),
        "display"=>native::launch("ms-settings:display"),
        "battery"=>native::launch("ms-settings:powersleep"),
        "focus"=>native::launch("ms-settings:quiethours"),
        "quick"=>native::system_combo('A'),
        "notifications"=>native::system_combo('N'),
        "volume-up"=>native::media_key("up"),
        "volume-down"=>native::media_key("down"),
        "volume-mute"=>native::media_key("mute"),
        _=>Err("Ação rápida inválida".into()),
    }
}
#[tauri::command]
fn resize_dock(window:WebviewWindow,state:State<'_,AppState>,width:f64,height:f64)->Result<(),String>{
    let label=window.label().to_string();
    let d=state.displays.lock().map_err(|e|e.to_string())?.get(&label).cloned().ok_or("Monitor não cadastrado")?;
    let profile=state.settings.lock().map_err(|e|e.to_string())?.monitors.get(&d.key).cloned().unwrap_or_default();
    let max_width=(d.size.width as f64/d.scale)-12.0;
    let w=width.clamp(260.0,max_width.max(260.0));let h=height.clamp(86.0,140.0);
    window.set_size(tauri::LogicalSize::new(w,h)).map_err(|e|e.to_string())?;
    position_dock(&window,&d,profile.bottom_gap)?;
    Ok(())
}
#[tauri::command]
fn get_bridge_status(state:State<'_,AppState>)->Result<bridge::BridgeStatus,String>{
    let settings=state.settings.lock().map_err(|e|e.to_string())?;
    let note=state.recovery_note.lock().map_err(|e|e.to_string())?;
    Ok(bridge::status(&settings,&note))
}
#[tauri::command]
fn set_bridge_mode(mode:String,state:State<'_,AppState>)->Result<bridge::BridgeStatus,String>{
    let mut settings=state.settings.lock().map_err(|e|e.to_string())?;
    let result=bridge::request_mode(&mode,&mut settings)?;
    if let Ok(mut note)=state.recovery_note.lock(){note.clear();}
    Ok(result)
}
#[tauri::command]
fn tray_snapshot(state:State<'_,AppState>)->Result<Vec<bridge::TrayEntry>,String>{
    let mut settings=state.settings.lock().map_err(|e|e.to_string())?;
    bridge::guarded_snapshot(&mut settings)
}
#[tauri::command]
fn clear_attention(state:State<'_,AppState>,exe:String)->Result<(),String>{
    let mut settings=state.settings.lock().map_err(|e|e.to_string())?;
    settings.attention.remove(&exe.to_lowercase());settings::save(&settings)
}

// Não forçar foco: a dock deve continuar visível sem roubar teclado do aplicativo.
fn restore_dock(window: &WebviewWindow, focus: bool) {
    if window.is_minimized().unwrap_or(false) { let _ = window.unminimize(); }
    if !window.is_visible().unwrap_or(true) { let _ = window.show(); }
    let _ = window.set_always_on_top(true);
    if focus { let _ = window.set_focus(); }
}

pub fn run(){
    let mut saved=settings::load();settings::validate(&mut saved);
    let note=bridge::start_session(&mut saved);
    tauri::Builder::default()
        // Deve ser o primeiro plugin: um segundo clique no EXE reabre a janela existente.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            for (label,d) in app.state::<AppState>().displays.lock().unwrap().iter(){
                if app.state::<AppState>().settings.lock().unwrap().monitors.get(&d.key).cloned().unwrap_or_default().enabled{
                    if let Some(win)=app.get_webview_window(label){restore_dock(&win,true);}
                }
            }
        }))
        .manage(AppState{settings:Mutex::new(saved),recovery_note:Mutex::new(note),panel_request:Mutex::new(PanelRequest{page:"settings".into(),target:String::new(),source_label:"dock".into(),serial:0}),displays:Mutex::new(HashMap::new())})
        .setup(|app|{
            if let Some(primary)=app.get_webview_window("dock") {
                let all=primary.available_monitors()?;
                let selected=primary.primary_monitor()?.or_else(||all.first().cloned());
                if let Some(ref m)=selected {
                    let d=Display{key:display_key(m),name:"Monitor principal".into(),pos:*m.position(),size:*m.size(),scale:m.scale_factor()};
                    app.state::<AppState>().displays.lock().unwrap().insert("dock".into(),d);
                }
                let mut n=1;
                for m in all {
                    if selected.as_ref().is_some_and(|p|display_key(p)==display_key(&m)){continue;}
                    let label=format!("dock_{n}");
                    let data=Display{key:display_key(&m),name:format!("Monitor adicional {n}"),pos:*m.position(),size:*m.size(),scale:m.scale_factor()};
                    app.state::<AppState>().displays.lock().unwrap().insert(label.clone(),data);
                    WebviewWindowBuilder::new(app,label.clone(),WebviewUrl::App("index.html".into()))
                        .title(format!("React Dock - Monitor {n}"))
                        .inner_size(720.0,116.0).decorations(false).transparent(true)
                        .always_on_top(true).skip_taskbar(true).shadow(false).resizable(false)
                        .visible(true).build()?;
                    n+=1;
                }
                let _=refresh_displays(app.handle());
                // Recuperação sem roubar foco e sem reviver monitores desativados.
                let handle=app.handle().clone();
                std::thread::spawn(move ||loop {
                    std::thread::sleep(Duration::from_secs(3));
                    let state=handle.state::<AppState>();
                    let maps=match state.displays.lock(){Ok(x)=>x.clone(),Err(_)=>break};
                    let cfg=match state.settings.lock(){Ok(x)=>x.clone(),Err(_)=>break};
                    for (label,d) in &maps {
                        if !cfg.monitors.get(&d.key).cloned().unwrap_or_default().enabled{continue;}
                        if let Some(win)=handle.get_webview_window(label){
                            if win.is_visible().is_err(){return;}
                            if win.is_minimized().unwrap_or(false){let _=win.unminimize();}
                            if !win.is_visible().unwrap_or(true){let _=win.show();}
                            if !win.is_always_on_top().unwrap_or(true){let _=win.set_always_on_top(true);}
                        }
                    }
                });
            }
            // Atalho global reversível do Iniciar personalizado, sem sequestrar Win puro.
            let shortcut_handle=app.handle().clone();
            native::spawn_start_shortcut(move || {
                let a=shortcut_handle.clone();
                let for_main=a.clone();
                let _=a.run_on_main_thread(move||{let _=open_panel(&for_main,"start","","dock");});
            });
            if let Some(icon)=app.default_window_icon().cloned() {
                let show=MenuItem::with_id(app,"show_dock","Mostrar React Dock",true,None::<&str>)?;
                let quit=MenuItem::with_id(app,"quit_dock","Sair do React Dock",true,None::<&str>)?;
                let menu=Menu::with_items(app,&[&show,&quit])?;
                tauri::tray::TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("React Dock • Sempre visível")
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app,event|match event.id().as_ref(){
                        "show_dock"=>{
                            if let Some(win)=app.get_webview_window("dock"){restore_dock(&win,true);}
                        }
                        "quit_dock"=>{bridge::end_session();app.exit(0);}
                        _=>{}
                    })
                    .on_tray_icon_event(|tray,event|{
                        if let tauri::tray::TrayIconEvent::DoubleClick{..}=event {
                            if let Some(win)=tray.app_handle().get_webview_window("dock"){
                                restore_dock(&win,true);
                            }
                        }
                    }).build(app)?;
            }
            Ok(())
        })
        .on_window_event(|window,event| match event {
            tauri::WindowEvent::CloseRequested{api,..} if window.label()=="panel" => {
                api.prevent_close();
                let _=window.hide();
            }
            tauri::WindowEvent::CloseRequested{api,..} => {
                // Fechamento por engano não pode fazer a dock desaparecer.
                api.prevent_close();
                let _=window.show();
                let _=window.set_always_on_top(true);
            }
            tauri::WindowEvent::Focused(false) if window.label().starts_with("dock") => {
                let _=window.set_always_on_top(true);
            }
            tauri::WindowEvent::Destroyed => bridge::end_session(),
            _=>{}
        })
        .invoke_handler(tauri::generate_handler![
            get_settings, save_settings, get_displays, current_display, quit_app, windows_list, start_apps, focus_window, toggle_window, launch, icon_for_path,
            show_panel, hide_panel, get_panel_request, show_window_preview, hide_window_preview,
            battery_status, quick_action, resize_dock, get_bridge_status,set_bridge_mode,tray_snapshot,clear_attention
        ])
        .run(tauri::generate_context!())
        .expect("React Dock: falha na inicialização");
}
