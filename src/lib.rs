#![allow(dead_code, non_camel_case_types, non_snake_case)]

use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::ffi::c_void;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::slice;

use nwn_lib_rs::tlk::{Tlk, TlkEntry, TlkResolver};

use flexi_logger::{FileSpec, LogSpecification, Logger};
#[allow(unused_imports)]
use log::{error, info, trace, warn};

#[derive(Default)]
struct XPTlk {
    resolvers: HashMap<String, TlkResolver>,
    nwnx_path: String,
    nwn2_install_path: String,
    nwn2_home_path: String,
}
impl XPTlk {
    fn get_entry(&self, key: &str, strref: i32) -> Option<&TlkEntry> {
        match strref.try_into() {
            Ok(strref) => {
                if let Some(resolv) = self.resolvers.get(key) {
                    resolv.get_entry(strref)
                } else {
                    log::error!("No resolver with key: {:?}", key);
                    None
                }
            }
            Err(e) => {
                log::error!("Invalid STRREF {}: {}", strref, e);
                None
            }
        }
    }

    fn replace_path_tokens(&self, path: &str) -> String {
        path.to_string()
            .replace("${NWNX}", &self.nwnx_path)
            .replace("${NWN2INST}", &self.nwn2_install_path)
            .replace("${NWN2HOME}", &self.nwn2_home_path)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NWNXCPlugin_InitInfo {
    pub dll_path: *const c_char,
    pub nwnx_path: *const c_char,
    pub nwn2_install_path: *const c_char,
    pub nwn2_home_path: *const c_char,
    pub nwn2_module_path: *const c_char,
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static nwnxcplugin_abi_version: u32 = 1;

// static mut current_plugin: Box<XPTlk> = None;

#[no_mangle]
#[allow(unused_attributes, unused_variables)]
pub extern "C" fn NWNXCPlugin_New(info: NWNXCPlugin_InitInfo) -> *mut c_void {
    let dll_path = unsafe { CStr::from_ptr(info.dll_path).to_str().unwrap_or("") };
    let nwnx_path = unsafe { CStr::from_ptr(info.nwnx_path).to_str().unwrap_or("") };
    let nwn2_install_path = unsafe {
        CStr::from_ptr(info.nwn2_install_path)
            .to_str()
            .unwrap_or("")
    };
    let nwn2_home_path = unsafe { CStr::from_ptr(info.nwn2_home_path).to_str().unwrap_or("") };
    let nwn2_module_path = unsafe { CStr::from_ptr(info.nwn2_module_path).to_str().unwrap_or("") };

    let log_path: PathBuf = [nwnx_path, "xp_tlk.txt"].iter().collect();

    if Logger::try_with_env_or_str("trace")
        .unwrap_or(Logger::with(LogSpecification::info()))
        .log_to_file(FileSpec::try_from(log_path).unwrap())
        .format(flexi_logger::detailed_format)
        .start()
        .is_err()
    {
        // TODO: maybe just ignore and continue?
        return std::ptr::null_mut();
    }
    log::trace!(
        "NWNXCPlugin_New(dll_path={:?}, nwnx_path={:?}, nwn2_install_path={:?}, nwn2_home_path={:?}, nwn2_module_path={:?})",
        dll_path,
        nwnx_path,
        nwn2_install_path,
        nwn2_home_path,
        nwn2_module_path
    );

    let mut plugin = XPTlk {
        resolvers: HashMap::new(),
        nwnx_path: nwnx_path.to_string(),
        nwn2_install_path: nwn2_install_path.to_string(),
        nwn2_home_path: nwn2_home_path.to_string(),
    };
    // std::mem::forget(&plugin);
    // &mut plugin as *mut _ as *mut c_void

    let plugin = std::boxed::Box::new(plugin);
    std::boxed::Box::into_raw(plugin) as *mut _ as *mut c_void
}

#[no_mangle]
#[allow(unused_attributes)]
pub extern "C" fn NWNXCPlugin_Delete(cplugin: *mut c_void) {
    unsafe { Box::from_raw(cplugin) };
    // let plugin: &mut XPTlk = unsafe { &mut *(cplugin as *mut _ as *mut XPTlk) };
    // std::mem::drop(plugin);
}

#[no_mangle]
pub extern "C" fn NWNXCPlugin_GetID(_cplugin: *mut c_void) -> *const c_char {
    // CString::new("tlk-rs").unwrap().into_raw()
    "tlk-rs\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn NWNXCPlugin_GetVersion() -> *const c_char {
    use git_version::git_version;
    CString::new(git_version!()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn NWNXCPlugin_GetInfo() -> *const c_char {
    "Load additional TLK files and resolve strrefs\0".as_ptr() as *const c_char
}

#[no_mangle]
#[allow(unused_attributes)]
pub fn NWNXCPlugin_GetInt(
    cplugin: *mut c_void,
    sFunction: *const c_char,
    sParam1: *const c_char,
    nParam2: i32,
) -> i32 {
    let plugin: &mut XPTlk = unsafe { &mut *(cplugin as *mut _ as *mut XPTlk) };
    let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
    let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
    let param2 = nParam2;
    log::trace!(
        "NWNXCPlugin_GetInt({:p}, {:?}, {:?}, {})",
        plugin,
        function,
        param1,
        param2
    );

    match function {
        "load" => {
            let mut args = param1.split("\n");
            let key = args.next().unwrap_or("");
            let base_path = plugin.replace_path_tokens(args.next().unwrap_or(""));
            let user_path = args
                .next()
                .and_then(|s| Some(plugin.replace_path_tokens(s)));

            match load_resolver(&base_path, user_path.as_deref()) {
                Ok(resolver) => {
                    // TODO: replace string portions with known paths
                    plugin.resolvers.insert(key.to_string(), resolver);
                    log::info!(
                        "Loaded new resolver {:?} with:\nBase TLK: {}\nUser TLK: {}",
                        key,
                        base_path,
                        user_path.as_deref().unwrap_or("<No user TLK>")
                    );
                    1
                }
                Err(e) => {
                    log::error!(
                        "Failed to load resolver {:?}: {}\n\tBase TLK: {}\n\tUser TLK: {}",
                        key,
                        e,
                        base_path,
                        user_path.as_deref().unwrap_or("<No user TLK>")
                    );
                    0
                }
            }
        }
        "is_loaded" => {
            let key = param1;
            plugin.resolvers.contains_key(key) as i32
        }
        "unload" => {
            let key = param1;
            let val = plugin.resolvers.remove(key);
            val.is_some() as i32
        }
        "get_lang" => {
            let key = param1;
            let tlk_index = param2;
            let lang = || -> Result<i32, Box<dyn Error>> {
                let resolv: &TlkResolver =
                    plugin.resolvers.get(key).ok_or(Box::new(NWNXError {
                        msg: format!("No resolver with key: {:?}", key),
                    }))?;

                let tlk: &Tlk = match tlk_index {
                    0 => &resolv.base,
                    1 => resolv.user.as_ref().ok_or(Box::new(NWNXError {
                        msg: format!("No user TLK for resolver {:?}", key),
                    }))?,
                    _ => {
                        return Err(Box::new(NWNXError {
                            msg: format!("Invalid tlk index: {}", tlk_index),
                        }))
                    }
                };
                Ok(tlk.header.language_id.try_into()?)
            }();

            match lang {
                Ok(i) => i,
                Err(e) => {
                    log::error!("{}", e);
                    -1
                }
            }
        }
        "get_flags" => {
            let key = param1;
            let strref = param2;
            if let Some(entry) = plugin.get_entry(key, strref) {
                entry.flags.try_into().unwrap_or(-1)
            } else {
                -1
            }
        }
        _ => -1,
    }
}

#[no_mangle]
#[allow(unused_attributes)]
pub fn NWNXCPlugin_GetFloat(
    cplugin: *mut c_void,
    sFunction: *const c_char,
    sParam1: *const c_char,
    nParam2: i32,
) -> f32 {
    let plugin: &mut XPTlk = unsafe { &mut *(cplugin as *mut _ as *mut XPTlk) };
    let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
    let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
    let param2 = nParam2;
    log::trace!(
        "NWNXCPlugin_GetFloat({:p}, {:?}, {:?}, {})",
        plugin,
        function,
        param1,
        param2
    );

    match function {
        "get_sound_length" => {
            let key = param1;
            let strref = param2;
            if let Some(entry) = plugin.get_entry(key, strref) {
                entry.sound_length.try_into().unwrap_or(-1.0)
            } else {
                -1.0
            }
        }
        _ => -1.0,
    }
}

#[no_mangle]
#[allow(unused_attributes)]
pub extern "C" fn NWNXCPlugin_GetString(
    cplugin: *mut c_void,
    sFunction: *const c_char,
    sParam1: *const c_char,
    nParam2: i32,
    result: *mut c_char,
    resultSize: usize,
) -> () {
    let plugin: &mut XPTlk = unsafe { &mut *(cplugin as *mut _ as *mut XPTlk) };
    let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
    let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
    let param2 = nParam2;
    log::trace!(
        "NWNXCPlugin_GetString({:p}, {:?}, {:?}, {})",
        plugin,
        function,
        param1,
        param2
    );

    let result_str: &str = match function {
        "get" => {
            let key = param1;
            let strref = param2;
            if let Ok(strref) = strref.try_into() {
                if let Some(resolv) = plugin.resolvers.get(key) {
                    resolv.get_str(strref).unwrap_or("")
                } else {
                    log::error!("No resolver with key: {:?}", key);
                    ""
                }
            } else {
                log::error!("Invalid STRREF: {}", param2);
                ""
            }
        }
        "get_sound_resref" => {
            let key = param1;
            let strref = param2;
            if let Some(entry) = plugin.get_entry(key, strref) {
                entry.sound_resref.as_str()
            } else {
                ""
            }
        }
        _ => "",
    };

    match CString::new(result_str) {
        Ok(c_result) => {
            let bytes: &[u8] = c_result.as_bytes_with_nul();
            unsafe {
                let result_bytes = slice::from_raw_parts_mut(result as *mut u8, resultSize);
                result_bytes[..bytes.len()].copy_from_slice(bytes);
            }
        }
        Err(e) => {
            println!("Unable to convert {:?} to C string: {}", result_str, e);
        }
    }
}

fn load_resolver(base_path: &str, user_path: Option<&str>) -> Result<TlkResolver, Box<dyn Error>> {
    let mut tlk_data = vec![];
    File::open(base_path)?.read_to_end(&mut tlk_data)?;
    let base_tlk = Tlk::from_bytes(&tlk_data).unwrap().1;

    let user_tlk = if let Some(user_path) = user_path {
        let mut tlk_data = vec![];
        File::open(user_path)?.read_to_end(&mut tlk_data)?;
        Some(Tlk::from_bytes(&tlk_data).unwrap().1)
    } else {
        None
    };

    Ok(TlkResolver::new(base_tlk, user_tlk))
}

#[derive(Debug, Clone)]
pub struct NWNXError {
    msg: String,
}
impl std::fmt::Display for NWNXError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}
impl Error for NWNXError {
    fn description(&self) -> &str {
        &self.msg
    }
}
