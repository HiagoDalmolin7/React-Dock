//! Supervisor de extensão EXPERIMENTAL. Nenhum código é injetado por React Dock.
//! O helper externo, caso instalado conscientemente, roda isolado e recebe timeout.
use crate::settings::{self, Settings};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::PathBuf, process::{Command, Stdio}, thread, time::{Duration,Instant}};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct TrayEntry {
    pub id:String, pub name:String, pub icon:Option<String>,
    pub command:Option<String>, pub tooltip:Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct BridgeStatus {
    pub mode:String,pub available:bool,pub failures:u32,pub detail:String,
}

pub fn helper_path()->Option<PathBuf>{
    let exe=std::env::current_exe().ok()?;
    Some(exe.parent()?.join("react-dock-tray-bridge.exe"))
}
pub fn available()->bool {helper_path().is_some_and(|p|p.is_file())}
pub fn status(settings:&Settings, info:&str)->BridgeStatus {
    BridgeStatus {mode:settings.tray_mode.clone(), available:available(), failures:settings.bridge_failures,
        detail:if info.is_empty() {if available(){"Extensão disponível; requer ativação manual"}else{"Helper externo não instalado; modo estável ativo"}} else {info}.to_string()}
}

/// Executa um snapshot via processo separado. Protocolo:
///   react-dock-tray-bridge.exe --snapshot
///   stdout: {"version":1,"icons":[{"id":"...","name":"...","icon":null,"command":null,"tooltip":null}]}
/// O helper não recebe credenciais, nem argumentos fornecidos pela UI.
pub fn snapshot()->Result<Vec<TrayEntry>,String>{
    let path=helper_path().ok_or("Localização da extensão indisponível")?;
    if !path.is_file(){return Err("Extensão experimental não instalada".into());}
    let mut child=Command::new(&path).arg("--snapshot").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null())
        .spawn().map_err(|e|format!("Não foi possível iniciar a extensão: {e}"))?;
    let start=Instant::now();
    let state=loop{
        match child.try_wait(){
            Ok(Some(status))=>break status,
            Ok(None) if start.elapsed()<Duration::from_millis(1600)=>thread::sleep(Duration::from_millis(40)),
            Ok(None)=>{let _=child.kill();let _=child.wait();return Err("Tempo limite da extensão excedido".into());},
            Err(e)=>{let _=child.kill();return Err(e.to_string());}
        }
    };
    if !state.success(){return Err("Extensão retornou erro".into());}
    let mut stdout=Vec::new();
    if let Some(mut out)=child.stdout.take(){out.take(1024*1024).read_to_end(&mut stdout).map_err(|e|e.to_string())?;}
    #[derive(Deserialize)] struct Response {version:u32,icons:Vec<TrayEntry>}
    let mut data:Response=serde_json::from_slice(&stdout).map_err(|_|"Resposta inválida da extensão")?;
    if data.version!=1 {return Err("Protocolo da extensão incompatível".into());}
    data.icons.truncate(80);
    for icon in &mut data.icons {
        fn clip(s:&mut String,max:usize) {
            if s.len()>max {
                let mut end=max;
                while !s.is_char_boundary(end) {end-=1;}
                s.truncate(end);
            }
        }
        clip(&mut icon.id,200);clip(&mut icon.name,120);
        if let Some(tip)=icon.tooltip.as_mut() {clip(tip,300);}
        if icon.icon.as_ref().is_some_and(|data|data.len()>240_000 || !data.starts_with("data:image/png;base64,")){icon.icon=None;}
        // Comandos vindos da extensão NÃO são executados automaticamente.
        icon.command=None;
    }
    Ok(data.icons)
}

pub fn request_mode(mode:&str, settings:&mut Settings)->Result<BridgeStatus,String>{
    match mode {
        "stable"=>{settings.tray_mode="stable".into();settings.bridge_failures=0;settings::save(settings)?;Ok(status(settings,"Modo estável ativado"))},
        "experimental"=>{
            if !available(){return Err("Instale uma extensão compatível e verificada para habilitar o hook experimental. Nenhuma DLL é fornecida nesta versão.".into());}
            snapshot()?; // prova de vida ANTES de ativar
            settings.tray_mode="experimental".into();settings.bridge_failures=0;settings::save(settings)?;
            Ok(status(settings,"Extensão experimental ativa; recuperação automática habilitada"))
        },
        _=>Err("Modo desconhecido".into()),
    }
}

pub fn guarded_snapshot(settings:&mut Settings)->Result<Vec<TrayEntry>,String>{
    if settings.tray_mode!="experimental" {return Ok(Vec::new());}
    match snapshot() {
        Ok(icons)=>{if settings.bridge_failures!=0{settings.bridge_failures=0;let _=settings::save(settings);}Ok(icons)},
        Err(e)=>{
            settings.bridge_failures+=1;
            if settings.bridge_failures>=3 {settings.tray_mode="stable".into();settings.bridge_failures=0;}
            let _=settings::save(settings);
            Err(format!("{e}. {}",if settings.tray_mode=="stable"{"Retornou automaticamente ao modo estável."}else{"O modo estável será ativado após 3 falhas."}))
        }
    }
}

pub fn start_session(settings:&mut Settings)->String {
    let marker=crate::settings::config_path().with_file_name("experimental-session.lock");
    let was_unclean=marker.exists();
    if was_unclean && settings.tray_mode=="experimental" {
        settings.tray_mode="stable".into();settings.bridge_failures=0;
        let _=settings::save(settings);
    }
    if let Some(parent)=marker.parent(){let _=std::fs::create_dir_all(parent);}
    let _=std::fs::write(marker,b"active");
    if was_unclean{"Uma sessão anterior não encerrou corretamente; modo estável restaurado".into()}else{String::new()}
}
pub fn end_session(){let _=std::fs::remove_file(crate::settings::config_path().with_file_name("experimental-session.lock"));}
