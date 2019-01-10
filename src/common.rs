use std::io;
use std::fs::{self};
use std::path::{Path,PathBuf};
use std::ffi::OsStr;
#[cfg(windows)]
use kernel32;
use winapi;
use log;
use log4rs;

trait OsStrExtension{
    fn to_wide_chars(&self) -> Vec<u16>;
}

fn from_wide_ptr(ptr: *const u16,size:isize) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    unsafe {
        assert!(!ptr.is_null());
        let len = (0..size).position(|i| *ptr.offset(i) == 0).unwrap();
        let slice = std::slice::from_raw_parts(ptr, len);
        OsString::from_wide(slice).to_string_lossy().into_owned()
    }
}

#[cfg(windows)]
impl OsStrExtension for &str {
    fn to_wide_chars(&self) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(self).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
    }
}

pub struct InitData{
    module_path:PathBuf,
}

impl InitData{
    pub fn get_module_path(&self)->PathBuf{
        self.module_path.clone()
    }
    pub fn get_install_directory(&self)->PathBuf{
        let mut install_path = self.module_path.clone();
        install_path.pop();
        install_path
    }
}

// DLLサーチパスのカレントディレクトリ無効化
// モジュールの完全修飾パス名を取得
// DLL Hijacking対策
#[cfg(windows)]
fn windows_init()->InitData{
    let mut buf;
    unsafe{
        // 関数：SetDllDirectoryW
        // 成功：非零
        // 失敗：0
        // 詳細：https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-setdlldirectoryw
        let ret = kernel32::SetDllDirectoryW("".to_wide_chars().as_ptr());
        if ret == 0{panic!("SetDllDirectoryW(\"\") failure!");}
        // 関数：SetSearchPathMode
        // 成功：非零
        // 失敗：0
        // 詳細：https://docs.microsoft.com/ja-jp/windows/desktop/api/winbase/nf-winbase-setsearchpathmode
        let ret = kernel32::SetSearchPathMode(0x00000001);
        if ret == 0{panic!("SetSearchPathMode(BASE_SEARCH_PATH_ENABLE_SAFE_SEARCHMODE) failure!");}
        // 関数：GetModuleFileNameW
        // 成功：非零
        // 失敗：0
        //　詳細：https://docs.microsoft.com/en-us/windows/desktop/api/libloaderapi/nf-libloaderapi-getmodulefilenamew
        buf = vec![0u16;4096];
        kernel32::GetModuleFileNameW(std::ptr::null_mut() ,buf.as_mut_ptr(),buf.len() as u32);
        if ret == 0{panic!("GetModuleFileNameW failure!");}
    }
    let execmodule_path_string = from_wide_ptr(buf.as_ptr(),buf.len() as isize).to_owned();
    InitData{module_path:Path::new(&execmodule_path_string).to_path_buf()}
}

#[cfg(unix)]
fn unix_init()->InitData{

}

pub fn application_init()->InitData{
    #[cfg(windows)]
    let data = windows_init();
    #[cfg(unix)]
    let data = unix_init();
    data
}

trait PathExtention{
    fn is_extension(&self,ext:&str)->bool;
}

impl PathExtention for PathBuf{
    fn is_extension(&self,ext: &str)->bool{
        let ext_self = self.extension().unwrap_or(OsStr::new(""));
        if ext == ext_self{
            true
        }else{
            false
        }
    }
}

pub fn dll_scan(dir: &PathBuf) -> io::Result<Vec<PathBuf>> {
    // ビルド先のOSによってDLLの拡張子を変える。
    // 動的ライブラリの拡張子は将来に渡って変更がないと思われるのでハードコード
    #[cfg(windows)]
    let dll_ext="dll";
    #[cfg(unix)]
    let dll_ext="so";

    let mut dirs = Vec::new();
    let mut files = Vec::new();
    dirs.push(dir.clone());
    while let Some(dir) = dirs.pop(){
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                let is_dll = path.is_extension(dll_ext);
                if path.metadata().unwrap().file_type().is_file()&&is_dll{
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}
