#![allow(dead_code, non_camel_case_types, non_snake_case)]

use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::ffi::c_void;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::os::raw::c_char;
use std::slice;

use nwn_lib_rs::tlk::{Tlk, TlkResolver};

use flexi_logger::{FileSpec, LogSpecification, Logger};
#[allow(unused_imports)]
use log::{error, info, trace, warn};

#[derive(Default)]
struct XPTlk {
    resolvers: HashMap<String, TlkResolver>,
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

#[no_mangle]
#[allow(unused_attributes, unused_variables)]
pub extern "C" fn NWNXCPlugin_New(info: NWNXCPlugin_InitInfo) -> *mut c_void {
    if Logger::try_with_env_or_str("info")
        .unwrap_or(Logger::with(LogSpecification::info()))
        .log_to_file(
            FileSpec::default()
                .basename("xp_tlk")
                .suffix("txt")
                .suppress_timestamp(),
            // FileSpec::try_from("xp_tlk.txt").unwrap(),
        )
        .start()
        .is_err()
    {
        // TODO: maybe just ignore and continue?
        return std::ptr::null_mut();
    }
    log::trace!("NWNXCPlugin_New({:?})", info);
    log::error!("Start {:?}", FileSpec::default()
                .basename("xp_tlk")
                .suffix("txt")
                .suppress_timestamp().as_pathbuf(None));

    let mut plugin = XPTlk {
        resolvers: HashMap::new(),
    };
    std::mem::forget(&plugin);
    &mut plugin as *mut _ as *mut c_void
}

#[no_mangle]
pub extern "C" fn NWNXCPlugin_GetID(_cplugin: &mut c_void) -> *const c_char {
    CString::new("tlk-rs").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn NWNXCPlugin_GetVersion() -> *const c_char {
    use git_version::git_version;
    CString::new(git_version!()).unwrap().into_raw()
}

#[no_mangle]
#[allow(unused_attributes)]
pub fn NWNXCPlugin_GetInt(
    cplugin: &mut c_void,
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
            let key = args.next().unwrap();
            let base_path = args.next().unwrap();
            let user_path = args.next();

            match load_resolver(base_path, user_path) {
                Ok(resolver) => {
                    plugin.resolvers.insert(key.to_string(), resolver);
                    log::info!(
                        "Loaded new resolver {:?} with:\nBase TLK: {}\nUser TLK: {}",
                        key,
                        base_path,
                        user_path.unwrap_or("<No user TLK>")
                    );
                    1
                }
                Err(e) => {
                    log::error!(
                        "Failed to load resolver {:?}: {}\nBase TLK: {}\nUser TLK: {}",
                        key,
                        e,
                        base_path,
                        user_path.unwrap_or("<No user TLK>")
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
        _ => 0,
    }
}
#[no_mangle]
#[allow(unused_attributes)]
pub extern "C" fn NWNXCPlugin_GetString(
    cplugin: &mut c_void,
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
            if let Ok(strref) = param2.try_into() {
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
