//! Integrações Win32. A dock NÃO registra APPBAR e não reserva WorkArea.
//! APIs documentadas, sem injetar código no explorer.exe.
use base64::Engine;
use serde::Serialize;
use std::{collections::HashMap, ffi::{c_void, OsStr}, io::Cursor, os::windows::ffi::OsStrExt, ptr, sync::{Mutex, OnceLock}};

type Handle = *mut c_void;

#[link(name="user32")]
extern "system" {
    fn EnumWindows(callback: unsafe extern "system" fn(Handle, isize)->i32, param:isize)->i32;
    fn IsWindowVisible(hwnd:Handle)->i32;
    fn GetWindowTextLengthW(hwnd:Handle)->i32;
    fn GetWindowTextW(hwnd:Handle,text:*mut u16,max:i32)->i32;
    fn GetWindowThreadProcessId(hwnd:Handle,pid:*mut u32)->u32;
    fn GetForegroundWindow()->Handle;
    fn GetWindowLongW(hwnd:Handle,index:i32)->i32;
    fn IsWindow(hwnd:Handle)->i32;
    fn IsIconic(hwnd:Handle)->i32;
    fn GetCursorPos(point:*mut Point)->i32;
    fn MonitorFromPoint(point:Point,flags:u32)->Handle;
    fn GetMonitorInfoW(monitor:Handle,info:*mut MonitorInfo)->i32;
    fn CreateWindowExW(exstyle:u32,class:*const u16,name:*const u16,style:u32,x:i32,y:i32,width:i32,height:i32,parent:Handle,menu:Handle,instance:Handle,param:*const c_void)->Handle;
    fn DestroyWindow(hwnd:Handle)->i32;
    fn ShowWindow(hwnd:Handle,cmd:i32)->i32;
    fn SetForegroundWindow(hwnd:Handle)->i32;
    fn keybd_event(vk:u8,scan:u8,flags:u32,extra:usize);
    fn DestroyIcon(icon:Handle)->i32;
    fn DrawIconEx(dc:Handle,x:i32,y:i32,icon:Handle,w:i32,h:i32,step:u32,brush:Handle,flags:u32)->i32;
    fn PrivateExtractIconsW(filename:*const u16,index:i32,w:i32,h:i32,icons:*mut Handle,ids:*mut u32,count:u32,flags:u32)->u32;

}
#[link(name="kernel32")]
extern "system" {
    fn OpenProcess(access:u32,inherit:i32,pid:u32)->Handle;
    fn CloseHandle(h:Handle)->i32;
    fn QueryFullProcessImageNameW(h:Handle,flags:u32,filename:*mut u16,size:*mut u32)->i32;
    fn GetSystemPowerStatus(status:*mut SystemPowerStatus)->i32;
}
#[link(name="shell32")]
extern "system" {
    fn ShellExecuteW(hwnd:Handle,operation:*const u16,file:*const u16,params:*const u16,dir:*const u16,show:i32)->Handle;
    fn ExtractIconExW(path:*const u16,index:i32,large:*mut Handle,small:*mut Handle,count:u32)->u32;
}
#[link(name="gdi32")]
extern "system" {
    fn CreateCompatibleDC(dc:Handle)->Handle;
    fn DeleteDC(dc:Handle)->i32;
    fn CreateDIBSection(dc:Handle,info:*const BitmapInfo,usage:u32,bits:*mut *mut c_void,section:Handle,offset:u32)->Handle;
    fn SelectObject(dc:Handle,obj:Handle)->Handle;
    fn DeleteObject(obj:Handle)->i32;
}
#[repr(C)] struct SystemPowerStatus { ac_line_status:u8,battery_flag:u8,battery_life_percent:u8,system_status_flag:u8,battery_life_time:u32,battery_full_life_time:u32 }
#[repr(C)] struct BitmapInfoHeader { size:u32,width:i32,height:i32,planes:u16,bit_count:u16,compression:u32,image_size:u32,xppm:i32,yppm:i32,clr_used:u32,clr_important:u32 }
#[repr(C)] struct BitmapInfo { header:BitmapInfoHeader, color:[u32;1] }

pub fn wide(s:&str)->Vec<u16>{OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all="camelCase")]
pub struct WindowItem { pub hwnd:String,pub title:String,pub exe:String,pub process:String,pub active:bool,pub icon:Option<String> }

fn process_exe(pid:u32)->Option<String> {
    unsafe {
        let handle=OpenProcess(0x1000,0,pid); // PROCESS_QUERY_LIMITED_INFORMATION
        if handle.is_null(){return None;}
        let mut buf=vec![0u16;32768];let mut size=buf.len() as u32;
        let ok=QueryFullProcessImageNameW(handle,0,buf.as_mut_ptr(),&mut size);
        CloseHandle(handle);
        if ok==0 {None} else {Some(String::from_utf16_lossy(&buf[..size as usize]))}
    }
}

struct WindowAccumulator(Vec<WindowItem>);
unsafe extern "system" fn collect(hwnd:Handle,param:isize)->i32 {
    let output=&mut *(param as *mut WindowAccumulator);
    if IsWindowVisible(hwnd)==0 || hwnd==GetForegroundWindow() && GetWindowTextLengthW(hwnd)==0 {return 1;}
    let exstyle=GetWindowLongW(hwnd,-20) as u32;
    if exstyle & 0x00000080 !=0 {return 1;} // WS_EX_TOOLWINDOW
    let len=GetWindowTextLengthW(hwnd);
    if len<=0 || len>4096{return 1;}
    let mut chars=vec![0u16;(len+1) as usize];
    let got=GetWindowTextW(hwnd,chars.as_mut_ptr(),len+1);
    if got<=0 {return 1;}
    let mut pid=0;GetWindowThreadProcessId(hwnd,&mut pid);
    let Some(exe)=process_exe(pid) else{return 1;};
    let name=std::path::Path::new(&exe).file_stem().and_then(|s|s.to_str()).unwrap_or("Aplicativo").to_string();
    if name.eq_ignore_ascii_case("react_dock") || name.eq_ignore_ascii_case("novadock") {return 1;}
    output.0.push(WindowItem {
        hwnd:(hwnd as usize).to_string(),title:String::from_utf16_lossy(&chars[..got as usize]),
        exe,process:name,active:hwnd==GetForegroundWindow(),icon:None,
    });
    1
}

pub fn windows_list()->Vec<WindowItem> {
    let mut out=WindowAccumulator(Vec::new());
    unsafe {EnumWindows(collect,&mut out as *mut WindowAccumulator as isize);}
    // O levantamento de janelas deve ser leve. Ícones são carregados sob demanda pelo frontend.
    out.0
}

pub fn toggle_window(hwnd:&str,was_active:bool)->Result<(),String>{
    let id=hwnd.parse::<usize>().map_err(|_|"Identificador de janela inválido")?;
    unsafe {
        let handle=id as Handle;
        if IsWindow(handle)==0{return Err("A janela já foi fechada".into());}
        if IsIconic(handle)==0 && (was_active || handle==GetForegroundWindow()){
            ShowWindow(handle,6); // SW_MINIMIZE
            return Ok(());
        }
    }
    focus_window(hwnd)
}

pub fn focus_window(hwnd:&str)->Result<(),String>{
    let id=hwnd.parse::<usize>().map_err(|_|"Identificador de janela inválido")?;
    unsafe {
        if IsWindow(id as Handle)==0 {return Err("A janela já foi fechada".into());}
        ShowWindow(id as Handle,9); // SW_RESTORE
        if SetForegroundWindow(id as Handle)==0 {return Err("O Windows não permitiu trazer a janela para frente".into());}
    }
    Ok(())
}

pub fn launch(target:&str)->Result<(),String>{
    let input=target.trim();
    if input.is_empty(){return Err("Informe o aplicativo ou endereço".into());}
    if input.len()>2600 {return Err("Endereço muito longo".into());}
    // ShellExecuteW aceita .exe, .lnk, arquivo, ms-settings:, protocolos e caminhos.
    // Não passamos input por cmd /c ou interpretador de shell.
    let path=wide(input);
    unsafe {
        let result=ShellExecuteW(ptr::null_mut(),ptr::null(),path.as_ptr(),ptr::null(),ptr::null(),1);
        if (result as isize)<=32 {Err(format!("O Windows não conseguiu abrir: {input}"))} else{Ok(())}
    }
}

pub fn media_key(action:&str)->Result<(),String>{
    let vk=match action{"up"=>0xAF,"down"=>0xAE,"mute"=>0xAD,_=>return Err("Comando de volume inválido".into())};
    unsafe {keybd_event(vk,0,0,0);keybd_event(vk,0,2,0);}
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Battery {pub percent:Option<u8>,pub charging:bool,pub on_ac:bool}
pub fn battery()->Battery{
    let mut s=SystemPowerStatus {ac_line_status:255,battery_flag:255,battery_life_percent:255,system_status_flag:0,battery_life_time:0,battery_full_life_time:0};
    let ok=unsafe {GetSystemPowerStatus(&mut s)};
    Battery {percent:if ok!=0&&s.battery_life_percent<=100{Some(s.battery_life_percent)}else{None},charging:ok!=0&&(s.battery_flag & 8)!=0,on_ac:ok!=0&&s.ac_line_status==1}
}

// Icon.ExtractAssociatedIcon não está disponível diretamente no Rust.
// ExtractIconExW + DrawIconEx grava a imagem diretamente numa DIB RGBA.
static ICONS:OnceLock<Mutex<HashMap<String,Option<String>>>>=OnceLock::new();
pub fn extract_icon(path:&str)->Option<String>{
    if !std::path::Path::new(path).exists(){return None;}
    let cache=ICONS.get_or_init(||Mutex::new(HashMap::new()));
    if let Ok(guard)=cache.lock(){if let Some(cached)=guard.get(path){return cached.clone();}}
    let result=extract_icon_uncached(path);
    if let Ok(mut guard)=cache.lock(){if guard.len()>180{guard.clear();} guard.insert(path.to_string(),result.clone());}
    result
}
fn extract_icon_uncached(path:&str)->Option<String>{
    unsafe{
        let mut large:Handle=ptr::null_mut();
        let mut resource_id=0u32;
        // Preferir recurso de 256px, sem ampliar à força um bitmap de 32/48px.
        // Caso a API não encontre esse ícone, usar extração tradicional.
        let extracted=PrivateExtractIconsW(wide(path).as_ptr(),0,256,256,&mut large,&mut resource_id,1,0);
        if extracted==0 || extracted==u32::MAX || large.is_null(){
            large=ptr::null_mut();
            if ExtractIconExW(wide(path).as_ptr(),0,&mut large,ptr::null_mut(),1)==0 || large.is_null(){return None;}
        }
        // 160px: excelente nitidez em telas HiDPI com ícones de 36-72px.
        const N:usize=160;
        let dc=CreateCompatibleDC(ptr::null_mut());
        if dc.is_null(){DestroyIcon(large);return None;}
        let info=BitmapInfo{header:BitmapInfoHeader{size:40,width:N as i32,height:-(N as i32),planes:1,bit_count:32,compression:0,image_size:(N*N*4) as u32,xppm:0,yppm:0,clr_used:0,clr_important:0},color:[0]};
        let mut pixels:*mut c_void=ptr::null_mut();
        let bitmap=CreateDIBSection(dc,&info,0,&mut pixels,ptr::null_mut(),0);
        if bitmap.is_null() || pixels.is_null(){DeleteDC(dc);DestroyIcon(large);return None;}
        let old=SelectObject(dc,bitmap);
        ptr::write_bytes(pixels,0,N*N*4);
        let drawn=DrawIconEx(dc,0,0,large,N as i32,N as i32,0,ptr::null_mut(),3);
        let raw=std::slice::from_raw_parts(pixels as *const u8,N*N*4);
        let mut rgba=vec![0u8;N*N*4];
        let mut has_alpha=false;
        for (src,dst) in raw.chunks_exact(4).zip(rgba.chunks_exact_mut(4)) {
            dst[0]=src[2];dst[1]=src[1];dst[2]=src[0];dst[3]=src[3];
            if src[3]!=0 {has_alpha=true;}
        }
        if !has_alpha {for p in rgba.chunks_exact_mut(4){ if p[0]!=0||p[1]!=0||p[2]!=0 {p[3]=255;} }}
        SelectObject(dc,old);DeleteObject(bitmap);DeleteDC(dc);DestroyIcon(large);
        if drawn==0 {return None;}
        let mut bytes=Cursor::new(Vec::new());
        { let mut png=png::Encoder::new(&mut bytes,N as u32,N as u32);png.set_color(png::ColorType::Rgba);png.set_depth(png::BitDepth::Eight);
            let mut wr=png.write_header().ok()?;wr.write_image_data(&rgba).ok()?; }
        Some(format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())))
    }
}

/// Abre painéis nativos sem acesso privilegiado: Win+A / Win+N.
pub fn system_combo(letter:char)->Result<(),String>{
    let code=match letter {'A'=>0x41,'N'=>0x4E,_=>return Err("Atalho indisponível".into())};
    unsafe{
        keybd_event(0x5B,0,0,0); // VK_LWIN
        keybd_event(code,0,0,0);
        keybd_event(code,0,2,0);
        keybd_event(0x5B,0,2,0);
    }
    Ok(())
}


// Prévia real de janelas por DWM. O helper não captura a tela e não injeta código em outros processos.
// Uma janela nativa sem ativação hospeda o thumbnail, em vez de depender de screenshots WebView.
#[repr(C)] #[derive(Clone,Copy,Default)] struct Rect {left:i32,top:i32,right:i32,bottom:i32}
#[repr(C)] #[derive(Clone,Copy,Default)] struct Point {x:i32,y:i32}
#[repr(C)] struct MonitorInfo {size:u32,monitor:Rect,work:Rect,flags:u32}
#[repr(C)] struct ThumbSize {cx:i32,cy:i32}
#[repr(C)] struct ThumbProps {flags:u32,dest:Rect,source:Rect,opacity:u8,visible:i32,client_only:i32}
#[link(name="dwmapi")]
extern "system" {
    fn DwmRegisterThumbnail(destination:Handle,source:Handle,thumb:*mut Handle)->i32;
    fn DwmUnregisterThumbnail(thumb:Handle)->i32;
    fn DwmQueryThumbnailSourceSize(thumb:Handle,size:*mut ThumbSize)->i32;
    fn DwmUpdateThumbnailProperties(thumb:Handle,props:*const ThumbProps)->i32;
    fn DwmSetWindowAttribute(hwnd:Handle,attribute:u32,value:*const c_void,size:u32)->i32;
}
struct Preview {win:usize,thumb:usize}
static PREVIEW:OnceLock<Mutex<Option<Preview>>>=OnceLock::new();
fn preview_state()->&'static Mutex<Option<Preview>>{PREVIEW.get_or_init(||Mutex::new(None))}
pub fn hide_preview(){
    if let Ok(mut state)=preview_state().lock(){
        if let Some(old)=state.take(){unsafe{
            DwmUnregisterThumbnail(old.thumb as Handle);
            DestroyWindow(old.win as Handle);
        }}
    }
}
pub fn show_preview(source:usize,dock_top:i32){
    hide_preview();
    unsafe {
        if IsWindow(source as Handle)==0{return;}
        const WIDTH:i32=320;const HEIGHT:i32=204;
        let mut point=Point::default(); if GetCursorPos(&mut point)==0{return;}
        let mut monitor=MonitorInfo{size:std::mem::size_of::<MonitorInfo>() as u32,monitor:Rect::default(),work:Rect::default(),flags:0};
        let mon=MonitorFromPoint(point,2);
        if mon.is_null() || GetMonitorInfoW(mon,&mut monitor)==0{return;}
        let x=(point.x-WIDTH/2).clamp(monitor.work.left+8,(monitor.work.right-WIDTH-8).max(monitor.work.left+8));
        let y=(dock_top-HEIGHT-11).clamp(monitor.work.top+8,(monitor.work.bottom-HEIGHT-8).max(monitor.work.top+8));
        // WS_EX_TOPMOST|WS_EX_TOOLWINDOW|WS_EX_NOACTIVATE|WS_EX_TRANSPARENT
        let exstyle=0x00000008|0x00000080|0x08000000|0x00000020;
        // WS_POPUP|WS_BORDER|SS_BLACKRECT: janela não clicável, sem barra de título.
        let win=CreateWindowExW(exstyle,wide("STATIC").as_ptr(),ptr::null(),0x80000000|0x00800000|0x00000004,x,y,WIDTH,HEIGHT,ptr::null_mut(),ptr::null_mut(),ptr::null_mut(),ptr::null());
        if win.is_null(){return;}
        let corner:i32=2; let _=DwmSetWindowAttribute(win,33,&corner as *const _ as *const c_void,4);
        let mut thumb:Handle=ptr::null_mut();
        if DwmRegisterThumbnail(win,source as Handle,&mut thumb)!=0 || thumb.is_null(){DestroyWindow(win);return;}
        let mut sz=ThumbSize{cx:WIDTH,cy:HEIGHT};let _=DwmQueryThumbnailSourceSize(thumb,&mut sz);
        let aspect=if sz.cy>0 {(sz.cx.max(1) as f64)/(sz.cy as f64)} else {1.6};
        let inner_w=((HEIGHT-12) as f64*aspect).round().clamp(1.0,(WIDTH-12) as f64) as i32;
        let inner_h=((inner_w as f64)/aspect).round().clamp(1.0,(HEIGHT-12) as f64) as i32;
        let left=(WIDTH-inner_w)/2;let top=(HEIGHT-inner_h)/2;
        let props=ThumbProps{flags:0x1|0x4|0x8|0x10,dest:Rect{left,top,right:left+inner_w,bottom:top+inner_h},source:Rect::default(),opacity:255,visible:1,client_only:0};
        if DwmUpdateThumbnailProperties(thumb,&props)!=0{DwmUnregisterThumbnail(thumb);DestroyWindow(win);return;}
        ShowWindow(win,8); // SW_SHOWNA; não desvia o foco
        if let Ok(mut state)=preview_state().lock(){*state=Some(Preview{win:win as usize,thumb:thumb as usize});}
    }
}
