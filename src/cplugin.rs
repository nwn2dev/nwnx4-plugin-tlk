#![allow(dead_code, non_camel_case_types, non_snake_case)]

use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::ffi;
use std::ffi::c_void;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::slice;

use flexi_logger::{FileSpec, LogSpecification, Logger};
#[allow(unused_imports)]
use log::{error, info, trace, warn};

pub struct InitInfo<'a> {
    pub dll_path: &'a str,
    pub nwnx_path: &'a str,
    pub nwn2_install_path: &'a str,
    pub nwn2_home_path: &'a str,
    pub nwn2_module_path: &'a str,
}

#[derive(Debug)]
struct UnimplementedError {
    func_name: &'static str,
}
impl std::error::Error for UnimplementedError {}
impl fmt::Display for UnimplementedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function is not implemented in CPlugin trait")
    }
}

pub enum COptStr<'a> {
    CStr(&'a ffi::CStr),
    Str(&'a str),
}

#[allow(unused_variables)]
pub trait CPlugin<'a>: Sized {
    fn new(info: InitInfo) -> Result<Self, Box<dyn Error>>;
    fn get_id(&mut self) -> &'a CStr {
        &CStr::from_bytes_with_nul("\0".as_bytes()).unwrap()
    }
    fn get_info() -> &'static CStr {
        &CStr::from_bytes_with_nul("\0".as_bytes()).unwrap()
    }
    fn get_version() -> &'static CStr {
        &CStr::from_bytes_with_nul("\0".as_bytes()).unwrap()
    }
    fn get_int(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
    ) -> Result<i32, Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "get_int",
        }))
    }
    fn set_int(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
        value: i32,
    ) -> Result<(), Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "set_int",
        }))
    }
    fn get_float(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
    ) -> Result<f32, Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "get_float",
        }))
    }
    fn set_float(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
        value: f32,
    ) -> Result<(), Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "set_float",
        }))
    }
    fn get_str(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
    ) -> Result<&str, Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "get_str",
        }))
    }
    fn set_str(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
        value: &str,
    ) -> Result<(), Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "set_str",
        }))
    }
    fn get_gff_size(&mut self, var_name: &str) -> Result<usize, Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "get_gff_size",
        }))
    }
    fn get_gff(&mut self, var_name: &str, buffer: &mut [u8]) {}
    fn set_gff(&mut self, var_name: &str, value: &[u8]) -> Result<(), Box<dyn Error>> {
        Err(Box::new(UnimplementedError {
            func_name: "set_gff",
        }))
    }
}

#[derive(Debug)]
pub enum CPluginEndpoints {
    GetID,
    GetInfo,
    GetVersion,
    GetInt,
    SetInt,
    GetFloat,
    SetFloat,
    GetString,
    SetString,
    GetGFFSize,
    GetGFF,
    SetGFF,
}

#[macro_export]
macro_rules! cplugin_hook {
    ($plugin_class:ty, [$($endpoints:ident),*]) => {

        #[repr(C)]
        #[derive(Debug)]
        struct NWNXCPlugin_InitInfo {
            pub dll_path: *const c_char,
            pub nwnx_path: *const c_char,
            pub nwn2_install_path: *const c_char,
            pub nwn2_home_path: *const c_char,
            pub nwn2_module_path: *const c_char,
        }

        #[allow(non_upper_case_globals)]
        #[no_mangle]
        static nwnxcplugin_abi_version: u32 = 1;

        #[no_mangle]
        extern "C" fn NWNXCPlugin_New(info: NWNXCPlugin_InitInfo) -> *mut c_void {
            use crate::cplugin::{InitInfo, CPlugin};

            let dll_path = unsafe { CStr::from_ptr(info.dll_path).to_str().unwrap_or("") };
            let nwnx_path = unsafe { CStr::from_ptr(info.nwnx_path).to_str().unwrap_or("") };
            let nwn2_install_path = unsafe {
                CStr::from_ptr(info.nwn2_install_path)
                    .to_str()
                    .unwrap_or("")
            };
            let nwn2_home_path =
                unsafe { CStr::from_ptr(info.nwn2_home_path).to_str().unwrap_or("") };
            let nwn2_module_path =
                unsafe { CStr::from_ptr(info.nwn2_module_path).to_str().unwrap_or("") };
            let init_info = InitInfo {
                dll_path,
                nwnx_path,
                nwn2_install_path,
                nwn2_home_path,
                nwn2_module_path,
            };

            match <$plugin_class>::new(init_info) {
                Ok(plugin) => {
                    let plugin = std::boxed::Box::new(plugin);
                    log::trace!(
                        "NWNXCPlugin_New(dll_path={:?}, nwnx_path={:?}, nwn2_install_path={:?}, nwn2_home_path={:?}, nwn2_module_path={:?}) -> {:p}",
                        dll_path,
                        nwnx_path,
                        nwn2_install_path,
                        nwn2_home_path,
                        nwn2_module_path,
                        plugin,
                    );
                    std::boxed::Box::into_raw(plugin) as *mut _ as *mut c_void
                },
                Err(e) => {
                    log::error!("NWNXCPlugin_New(dll_path={:?}, nwnx_path={:?}, nwn2_install_path={:?}, nwn2_home_path={:?}, nwn2_module_path={:?}) -> Error {}",
                        dll_path,
                        nwnx_path,
                        nwn2_install_path,
                        nwn2_home_path,
                        nwn2_module_path,
                        e,
                    );
                    std::ptr::null_mut()
                },
            }
        }

        #[no_mangle]
        #[allow(unused_attributes)]
        pub extern "C" fn NWNXCPlugin_Delete(cplugin: *mut c_void) {
            unsafe { Box::from_raw(cplugin) };
        }

        // Implement functions
        $(
             cplugin_hook!($plugin_class, implement $endpoints);
        )*
    };

    ($plugin_class:ty, implement New) => {
        compile_error!("New and Delete are always implemented");
    };

    ($plugin_class:ty, implement Delete) => {
        compile_error!("New and Delete are always implemented");
    };

    ($plugin_class:ty, implement GetID) => {
        #[no_mangle]
        pub extern "C" fn NWNXCPlugin_GetID(cplugin: *mut c_void) -> *const c_char {
            use crate::cplugin::CPlugin;
            let plugin: &mut $plugin_class = unsafe { &mut *(cplugin as *mut _ as *mut $plugin_class) };
            plugin.get_id().as_ptr()
        }
    };

    ($plugin_class:ty, implement GetVersion) => {
        #[no_mangle]
        pub extern "C" fn NWNXCPlugin_GetVersion() -> *const c_char {
            use crate::cplugin::CPlugin;
            <$plugin_class>::get_version().as_ptr()
        }
    };

    ($plugin_class:ty, implement GetInfo) => {
        #[no_mangle]
        pub extern "C" fn NWNXCPlugin_GetInfo() -> *const c_char {
            use crate::cplugin::CPlugin;
            <$plugin_class>::get_info().as_ptr()
        }
    };

    ($plugin_class:ty, implement GetInt) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub fn NWNXCPlugin_GetInt(
            cplugin: *mut c_void,
            sFunction: *const c_char,
            sParam1: *const c_char,
            nParam2: i32,
        ) -> i32 {
            use crate::cplugin::CPlugin;
            let plugin: &mut $plugin_class = unsafe { &mut *(cplugin as *mut _ as *mut $plugin_class) };
            let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
            let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
            log::trace!(
                "NWNXCPlugin_GetInt({:p}, {:?}, {:?}, {})",
                plugin,
                function,
                param1,
                nParam2,
            );
            match plugin.get_int(function, param1, nParam2) {
                Ok(i) => i,
                Err(e) => {
                    log::error!("NWNXCPlugin_GetInt -> Error {}", e);
                    0
                }
            }
        }
    };

    ($plugin_class:ty, implement SetInt) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub fn NWNXCPlugin_SetInt(
            cplugin: *mut c_void,
            sFunction: *const c_char,
            sParam1: *const c_char,
            nParam2: i32,
            nValue: i32,
        ) {
            use crate::cplugin::CPlugin;
            let plugin: &mut $plugin_class = unsafe { &mut *(cplugin as *mut _ as *mut $plugin_class) };
            let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
            let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
            log::trace!(
                "NWNXCPlugin_SetInt({:p}, {:?}, {:?}, {}, {})",
                plugin,
                function,
                param1,
                nParam2,
                nValue,
            );
            match plugin.set_int(function, param1, nParam2, nValue) {
                Ok(()) => {},
                Err(e) => {
                    log::error!("NWNXCPlugin_SetInt -> Error {}", e);
                }
            }
        }
    };

    ($plugin_class:ty, implement GetFloat) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub fn NWNXCPlugin_GetFloat(
            cplugin: *mut c_void,
            sFunction: *const c_char,
            sParam1: *const c_char,
            nParam2: i32,
        ) -> f32 {
            use crate::cplugin::CPlugin;
            let plugin: &mut $plugin_class = unsafe { &mut *(cplugin as *mut _ as *mut $plugin_class) };
            let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
            let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
            log::trace!(
                "NWNXCPlugin_GetFloat({:p}, {:?}, {:?}, {})",
                plugin,
                function,
                param1,
                nParam2,
            );
            match plugin.get_float(function, param1, nParam2) {
                Ok(f) => f,
                Err(e) => {
                    log::error!("NWNXCPlugin_GetFloat -> Error {}", e);
                    0.0
                }
            }
        }
    };

    ($plugin_class:ty, implement SetFloat) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub fn NWNXCPlugin_SetFloat(
            cplugin: *mut c_void,
            sFunction: *const c_char,
            sParam1: *const c_char,
            nParam2: i32,
            fValue: f32,
        ) {
            use crate::cplugin::CPlugin;
            let plugin: &mut $plugin_class = unsafe { &mut *(cplugin as *mut _ as *mut $plugin_class) };
            let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
            let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
            log::trace!(
                "NWNXCPlugin_SetFloat({:p}, {:?}, {:?}, {}, {})",
                plugin,
                function,
                param1,
                nParam2,
                fValue,
            );
            match plugin.set_float(function, param1, nParam2, fValue) {
                Ok(()) => {},
                Err(e) => {
                    log::error!("NWNXCPlugin_SetFloat -> Error {}", e);
                }
            }
        }
    };

    ($plugin_class:ty, implement GetString) => {

        #[no_mangle]
        #[allow(unused_attributes)]
        pub extern "C" fn NWNXCPlugin_GetString(
            cplugin: *mut c_void,
            sFunction: *const c_char,
            sParam1: *const c_char,
            nParam2: i32,
            result: *mut c_char,
            resultSize: usize,
        ) {
            use crate::cplugin::{CPlugin, COptStr};
            let plugin: &mut XPTlk = unsafe { &mut *(cplugin as *mut _ as *mut XPTlk) };
            let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
            let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
            let result_bytes = unsafe { slice::from_raw_parts_mut(result as *mut u8, resultSize) };
            log::trace!(
                "NWNXCPlugin_GetString({:p}, {:?}, {:?}, {})",
                plugin,
                function,
                param1,
                nParam2
            );

            match plugin.get_str(function, param1, nParam2) {
                Ok(s) => {
                    log::error!("plugin.get_str returned {:?}", s);

                    let bytes = s.as_bytes();
                    if bytes.len() + 1 < resultSize {
                        result_bytes[..bytes.len()].copy_from_slice(bytes);
                        result_bytes[bytes.len()] = 0;
                    } else{
                        log::error!(
                            "NWNXCPlugin_GetString -> Data is too long to fit result buffer. buffer_length={} result_length={}",
                            resultSize,
                            bytes.len() + 1
                        );
                    }
                },
                Err(e) => {
                    log::error!("NWNXCPlugin_GetString -> Error {}", e);
                    result_bytes[0] = 0;
                }
            }
        }
    };

    ($plugin_class:ty, implement SetString) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub fn NWNXCPlugin_SetString(
            cplugin: *mut c_void,
            sFunction: *const c_char,
            sParam1: *const c_char,
            nParam2: i32,
            sValue: *const c_char,
        ) {
            use crate::cplugin::CPlugin;
            let plugin: &mut $plugin_class = unsafe { &mut *(cplugin as *mut _ as *mut $plugin_class) };
            let function: &str = unsafe { CStr::from_ptr(sFunction).to_str().unwrap_or("") };
            let param1: &str = unsafe { CStr::from_ptr(sParam1).to_str().unwrap_or("") };
            let value: &str = unsafe { CStr::from_ptr(sValue).to_str().unwrap_or("") };
            log::trace!(
                "NWNXCPlugin_SetString({:p}, {:?}, {:?}, {}, {})",
                plugin,
                function,
                param1,
                nParam2,
                value,
            );
            match plugin.set_str(function, param1, nParam2, value) {
                Ok(()) => {},
                Err(e) => {
                    log::error!("NWNXCPlugin_SetString -> Error {}", e);
                }
            }
        }
    };

    ($plugin_class:ty, implement GetGFFSize) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub fn NWNXCPlugin_GetGFFSize(
            cplugin: *mut c_void,
            sVarName: *const c_char,
        ) -> usize {
            use crate::cplugin::CPlugin;
            let plugin: &mut $plugin_class = unsafe { &mut *(cplugin as *mut _ as *mut $plugin_class) };
            let varname: &str = unsafe { CStr::from_ptr(sVarName).to_str().unwrap_or("") };
            log::trace!(
                "NWNXCPlugin_GetGFFSize({:p}, {:?})",
                plugin,
                varname,
            );
            match plugin.get_gff_size(varname) {
                Ok(len) => { len },
                Err(e) => {
                    log::error!("NWNXCPlugin_GetGFFSize -> Error {}", e);
                    0
                }
            }
        }
    };

    ($plugin_class:ty, implement GetGFF) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub extern "C" fn NWNXCPlugin_GetGFF(
            cplugin: *mut c_void,
            sVarName: *const c_char,
            gffData: *mut u8,
            gffDataSize: usize,
        ) {
            use crate::cplugin::{CPlugin, COptStr};
            let plugin: &mut XPTlk = unsafe { &mut *(cplugin as *mut _ as *mut XPTlk) };
            let varname: &str = unsafe { CStr::from_ptr(sVarName).to_str().unwrap_or("") };
            let gff_data = unsafe { slice::from_raw_parts_mut(gffData as *mut u8, gffDataSize) };
            log::trace!(
                "NWNXCPlugin_GetGFF({:p}, {:?})",
                plugin,
                varname,
            );

            plugin.get_gff(varname, gff_data);
        }
    };

    ($plugin_class:ty, implement SetGFF) => {
        #[no_mangle]
        #[allow(unused_attributes)]
        pub fn NWNXCPlugin_SetGFF(
            cplugin: *mut c_void,
            sVarName: *const c_char,
            gffData: *const u8,
            gffDataSize: usize,
        ) {
            use crate::cplugin::CPlugin;
            let plugin: &mut XPTlk = unsafe { &mut *(cplugin as *mut _ as *mut XPTlk) };
            let varname: &str = unsafe { CStr::from_ptr(sVarName).to_str().unwrap_or("") };
            let gff_data = unsafe { slice::from_raw_parts(gffData as *const u8, gffDataSize) };
            log::trace!(
                "NWNXCPlugin_SetGFF({:p}, {:?}, {:?})",
                plugin,
                varname,
                std::str::from_utf8(gff_data.get(0..8).unwrap_or(&[])).unwrap_or(""),
            );
            match plugin.set_gff(varname, gff_data) {
                Ok(()) => {},
                Err(e) => {
                    log::error!("NWNXCPlugin_SetGFF -> Error {}", e);
                }
            }
        }
    };

    ($plugin_class:ty, inject $endpoint:ident) => {
        compile_error!(stringify!(Unknown hook: $endpoint));
    };
}
