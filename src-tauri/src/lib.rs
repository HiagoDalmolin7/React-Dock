#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod native;
mod settings;

use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};


#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
struct PanelRequest { page: String, target: String, serial: u64 }

// As páginas são independentes da área interativa da dock. Não ampliamos a janela principal.
fn reposition_panel(app:&AppHandle)->Result<(),String>{
    let dock=app.get_webview_window("dock").ok_or("Dock não encontrada")?;
    let panel=app.get_webview_window("panel").ok_or("Janela de configurações não encontrada")?;
    let pos=dock.outer_position().map_err(|e|e.to_string())?;
    let sz=dock.outer_size().map_err(|e|e.to_string())?;
    let ps=panel.outer_size().map_err(|e|e.to_string())?;
    let monitor=dock.current_monitor().map_err(|e|e.to_string())?.or_else(||dock.primary_monitor().ok().flatten()).ok_or("Monitor indisponível")?;
    let point=monitor.position();
    let x=pos.x+(sz.width as i32-ps.width as i32)/2;
    let y=(pos.y-ps.height as i32-12).max(point.y+12);
    panel.set_position(tauri::PhysicalPosition::new(x,y)).map_err(|e|e.to_string())
}
#[tauri::command]
fn show_panel(app:AppHandle,state:State<'_,AppState>,page:String,target:Option<String>)->Result<(),String>{
    if !matches!(page.as_str(),"tray"|"quick"|"settings"|"edit") {return Err("Página inválida".into());}
    let _=app.run_on_main_thread(||native::hide_preview());
    {
        let mut current=state.panel_request.lock().map_err(|e|e.to_string())?;
        current.page=page;
        current.target=target.unwrap_or_default();
        current.serial=current.serial.wrapping_add(1);
    }
    reposition_panel(&app)?;
    let panel=app.get_webview_window("panel").ok_or("Painel não encontrado")?;
    panel.show().map_err(|e|e.to_string())?;
    panel.set_always_on_top(true).map_err(|e|e.to_string())?;
    panel.set_focus().map_err(|e|e.to_string())?;
    Ok(())
}
#[tauri::command]
fn hide_panel(app:AppHandle)->Result<(),String>{
    if let Some(panel)=app.get_webview_window("panel") {panel.hide().map_err(|e|e.to_string())?;}
    Ok(())
}
#[tauri::command]
fn get_panel_request(state:State<'_,AppState>)->Result<PanelRequest,String>{
    Ok(state.panel_request.lock().map_err(|e|e.to_string())?.clone())
}
#[tauri::command]
fn toggle_window(hwnd:String,was_active:bool)->Result<(),String>{ native::toggle_window(&hwnd,was_active) }
#[tauri::command]
fn show_window_preview(app:AppHandle,hwnd:String)->Result<(),String>{
    let id=hwnd.parse::<usize>().map_err(|_|"Identificador de janela inválido")?;
    let dock=app.get_webview_window("dock").ok_or("Dock não encontrada")?;
    let y=dock.outer_position().map_err(|e|e.to_string())?.y;
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
    if let Some(window)=app.get_webview_window("dock"){reposition(&window,value.bottom_gap)?;}
    let _=reposition_panel(&app);
    *lock=value.clone();Ok(value)
}
#[tauri::command]
fn windows_list()->Vec<native::WindowItem>{native::windows_list()}
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
fn reposition(window:&WebviewWindow,bottom_gap:u32)->Result<(),String>{
    let monitor=window.current_monitor().map_err(|e|e.to_string())?
        .or_else(||window.primary_monitor().ok().flatten()).ok_or("Monitor não encontrado")?;
    let size=window.outer_size().map_err(|e|e.to_string())?;
    let screen=monitor.size();let point=monitor.position();
    let bottom=((bottom_gap as f64)*monitor.scale_factor()).round() as i32;
    let x=point.x+(screen.width as i32-size.width as i32)/2;
    let y=point.y+screen.height as i32-size.height as i32-bottom;
    window.set_position(tauri::PhysicalPosition::new(x,y)).map_err(|e|e.to_string())
}
#[tauri::command]
fn resize_dock(window:WebviewWindow,state:State<'_,AppState>,width:f64,height:f64)->Result<(),String>{
    let w=width.clamp(320.0,1480.0);let h=height.clamp(86.0,570.0);
    window.set_size(tauri::LogicalSize::new(w,h)).map_err(|e|e.to_string())?;
    let gap=state.settings.lock().map_err(|e|e.to_string())?.bottom_gap;
    reposition(&window,gap)?;
    let _=reposition_panel(&window.app_handle());
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
            if let Some(win)=app.get_webview_window("dock") {restore_dock(&win, true);}
        }))
        .manage(AppState{settings:Mutex::new(saved),recovery_note:Mutex::new(note),panel_request:Mutex::new(PanelRequest{page:"settings".into(),target:String::new(),serial:0})})
        .setup(|app|{
            if let Some(win)=app.get_webview_window("dock") {
                let gap=app.state::<AppState>().settings.lock().map(|s|s.bottom_gap).unwrap_or(8);
                let _=reposition(&win,gap);
                restore_dock(&win,false);
                // Recuperação de composição/visibilidade: sem focar no usuário.
                // Não esconde, minimiza nem cria outra janela.
                let watch=win.clone();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        if watch.is_visible().is_err(){break;}
                        if watch.is_minimized().unwrap_or(false) {let _=watch.unminimize();}
                        if !watch.is_visible().unwrap_or(true){let _=watch.show();}
                        // Não repetir a operação quando a janela já é topmost.
                        if !watch.is_always_on_top().unwrap_or(true){let _=watch.set_always_on_top(true);}
                    }
                });
            }
            if let Some(icon)=app.default_window_icon().cloned() {
                let show=MenuItem::with_id(app,"show_dock","Mostrar React Dock",true,None::<&str>)?;
                let quit=MenuItem::with_id(app,"quit_dock","Sair do React Dock",true,None::<&str>)?;
                let menu=Menu::with_items(app,&[&show,&quit])?;
                tauri::tray::TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("React Dock • Sempre visível")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app,event|match event.id().as_ref(){
                        "show_dock"=>{
                            if let Some(win)=app.get_webview_window("dock"){restore_dock(&win,true);}
                        }
                        "quit_dock"=>{bridge::end_session();app.exit(0);}
                        _=>{}
                    })
                    .on_tray_icon_event(|tray,event|{
                        if let tauri::tray::TrayIconEvent::Click{..}=event {
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
            tauri::WindowEvent::Focused(false) if window.label()=="dock" => {
                let _=window.set_always_on_top(true);
            }
            tauri::WindowEvent::Destroyed => bridge::end_session(),
            _=>{}
        })
        .invoke_handler(tauri::generate_handler![
            get_settings, save_settings, windows_list, focus_window, toggle_window, launch, icon_for_path,
            show_panel, hide_panel, get_panel_request, show_window_preview, hide_window_preview,
            battery_status, quick_action, resize_dock, get_bridge_status,set_bridge_mode,tray_snapshot,clear_attention
        ])
        .run(tauri::generate_context!())
        .expect("React Dock: falha na inicialização");
}
