use libloading;
mod common;
use std::path::{Path,PathBuf};
use std::fs::DirEntry;
use std::env;
#[macro_use]
use log;
use log4rs;
fn callback(path:DirEntry){

}

#[cfg(debug_assertions)]
fn debug_path_adjust(path:&mut PathBuf){
    path.pop();
    path.pop();
}
use std::error::Error;
fn main() {
    let init_data = common::application_init();
    let mut pulgin_dir = init_data.get_install_directory();
    
    #[cfg(debug_assertions)]
    {
        debug_path_adjust(&mut pulgin_dir);
    }
    let mut config_dir = pulgin_dir.clone();
    config_dir.push("config/log4rs.toml");
    log4rs::init_file(config_dir, Default::default()).unwrap();
    log::info!("booting up");
    pulgin_dir.push("pulgins");
    let dlllist = common::dll_scan(&pulgin_dir).unwrap();
    let mut plugin_instances = Vec::new();
    for dll in dlllist{
        let lib = match libloading::Library::new(dll.clone()){
            Ok(lib)=>{
                let path=dll.to_str().unwrap().to_owned();
                log::info!("{} がロードされました。",path);
                lib
            },
            Err(e)=>{
                // GetLastErrorで取得される「%1 は有効なアプリケーションではありません」コード193　が返却されることを想定。
                let path=dll.to_str().unwrap().to_owned();
                log::warn!("{}",e.to_string().replace("%1",&path));
                continue;
            }
        };
        plugin_instances.push(lib);
    }
    for lib in plugin_instances{
        unsafe {
            let func: libloading::Symbol<unsafe extern fn()> = lib.get(b"hello").unwrap();
            func();
        }
    }
}
