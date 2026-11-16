use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all="camelCase")]
pub struct LaunchItem {
    pub id: String,
    pub name: String,
    pub target: String,
    pub icon: String,
}
impl Default for LaunchItem {
    fn default() -> Self { Self { id:String::new(), name:String::new(), target:String::new(), icon:"app".into() } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all="camelCase")]
pub struct Settings {
    pub icon_size: u32,
    pub opacity: u8,
    pub accent: String,
    pub bottom_gap: u32,
    pub reduce_motion: bool,
    pub show_running: bool,
    pub show_clock: bool,
    pub auto_start: bool,
    pub tray_mode: String,
    pub bridge_failures: u32,
    pub pinned: Vec<LaunchItem>,
    pub tray_pinned: Vec<LaunchItem>,
    pub tray_hidden: Vec<String>,
    // Atenções do shell, não quantidade de mensagens não lidas.
    pub attention: HashMap<String, u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            icon_size: 50, opacity: 32, accent: "#b2bbff".into(), bottom_gap: 8,
            reduce_motion: false, show_running: true, show_clock: true, auto_start: false, tray_mode: "stable".into(), bridge_failures: 0,
            pinned: vec![
                LaunchItem { id:"explorer".into(), name:"Arquivos".into(), target:"explorer.exe".into(), icon:"folder".into() },
                LaunchItem { id:"edge".into(), name:"Navegador".into(), target:"msedge.exe".into(), icon:"globe".into() },
                LaunchItem { id:"settings".into(), name:"Configurações".into(), target:"ms-settings:".into(), icon:"settings".into() },
            ],
            tray_pinned: vec![
                LaunchItem { id:"security".into(), name:"Segurança".into(), target:"windowsdefender:".into(), icon:"shield".into() },
                LaunchItem { id:"taskmgr".into(), name:"Gerenciador de tarefas".into(), target:"taskmgr.exe".into(), icon:"cpu".into() },
            ],
            tray_hidden: vec![], attention: HashMap::new(),
        }
    }
}

pub fn config_path() -> PathBuf {
    let root = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    root.join("ReactDock").join("config.json")
}

pub fn load() -> Settings {
    // Upgrade sem perder preferências de uma instalação antiga da NovaDock.
    let primary = config_path();
    if let Ok(s) = fs::read_to_string(&primary) {
        return serde_json::from_str(&s).unwrap_or_default();
    }
    let legacy = std::env::var_os("APPDATA")
        .map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
        .join("NovaDock").join("config.json");
    fs::read_to_string(&legacy).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(settings: &Settings) -> Result<(), String> {
    let path=config_path();
    if let Some(parent)=path.parent() { fs::create_dir_all(parent).map_err(|e|e.to_string())?; }
    let payload=serde_json::to_vec_pretty(settings).map_err(|e|e.to_string())?;
    // Escrita em arquivo temporário reduz risco de corrupção; não é transação atômica no Windows.
    let temp=path.with_extension("tmp");
    fs::write(&temp,payload).map_err(|e|e.to_string())?;
    if path.exists() { let _=fs::remove_file(&path); }
    fs::rename(temp,path).map_err(|e|e.to_string())
}

pub fn validate(s:&mut Settings) {
    if s.tray_mode!="experimental" {s.tray_mode="stable".into();}
    s.bridge_failures=s.bridge_failures.min(3);
    s.icon_size=s.icon_size.clamp(36,72);
    s.opacity=s.opacity.clamp(6,90);
    s.bottom_gap=s.bottom_gap.clamp(0,60);
    if !s.accent.starts_with('#') || s.accent.len()!=7 || !s.accent[1..].bytes().all(|c|c.is_ascii_hexdigit()) {
        s.accent="#b2bbff".into();
    }
    s.pinned.truncate(30);
    s.tray_pinned.truncate(40);
    s.tray_hidden.truncate(100);
    s.attention.retain(|_,v|*v>0);
}
