#![allow(dead_code, non_snake_case)]
#![allow(unused_imports)]

mod cplugin;

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

use nwn_lib_rs::tlk;

use flexi_logger::{FileSpec, LogSpecification, Logger};
#[allow(unused_imports)]
use log::{error, info, trace, warn};

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

#[derive(Default)]
struct XPTlk {
    resolvers: HashMap<String, tlk::Resolver>,
    nwnx_path: String,
    nwn2_install_path: String,
    nwn2_home_path: String,
}
impl XPTlk {
    fn get_tlk_entry(
        &self,
        key: &str,
        strrefgender: i32,
    ) -> Result<Option<&tlk::Entry>, Box<dyn Error>> {
        let strref = strrefgender as u32 & !(1u32 << 31);
        let gender: tlk::Gender = match (strrefgender as u32 >> 31) & 1u32 {
            0 => tlk::Gender::Male,
            1 => tlk::Gender::Female,
            _ => panic!(),
        };

        match strref.try_into() {
            Ok(strref) => {
                if let Some(resolv) = self.resolvers.get(key) {
                    Ok(resolv.get_entry(strref, gender))
                } else {
                    Err(Box::new(NWNXError {
                        msg: format!("No resolver with key: {:?}", key),
                    }))
                }
            }
            Err(e) => Err(Box::new(NWNXError {
                msg: format!("Invalid STRREF {}: {}", strref, e),
            })),
        }
    }
    fn get_tlk_str(&self, key: &str, strrefgender: i32) -> Result<Option<&str>, Box<dyn Error>> {
        let strref = strrefgender as u32 & !(1u32 << 31);
        let gender: tlk::Gender = match (strrefgender as u32 >> 31) & 1u32 {
            0 => tlk::Gender::Male,
            1 => tlk::Gender::Female,
            _ => panic!(),
        };

        log::error!("get_tlk_str strref={} gender={:?}", strref, gender);

        match strref.try_into() {
            Ok(strref) => {
                if let Some(resolv) = self.resolvers.get(key) {
                    Ok(resolv.get_str(strref, gender))
                } else {
                    Err(Box::new(NWNXError {
                        msg: format!("No resolver with key: {:?}", key),
                    }))
                }
            }
            Err(e) => Err(Box::new(NWNXError {
                msg: format!("Invalid STRREF {}: {}", strref, e),
            })),
        }
    }

    fn replace_path_tokens(&self, path: &str) -> String {
        path.to_string()
            .replace("${NWNX}", &self.nwnx_path)
            .replace("${NWN2INST}", &self.nwn2_install_path)
            .replace("${NWN2HOME}", &self.nwn2_home_path)
    }

    fn load_resolver(
        base_path: &str,
        base_f_path: Option<&str>,
        user_path: Option<&str>,
    ) -> Result<tlk::Resolver, Box<dyn Error>> {
        let mut tlk_data = vec![];
        File::open(base_path)?.read_to_end(&mut tlk_data)?;
        let base_tlk = tlk::Tlk::from_bytes(&tlk_data).unwrap().1;

        let base_f_tlk = if let Some(base_f_path) = base_f_path {
            let mut tlk_data = vec![];
            File::open(base_f_path)?.read_to_end(&mut tlk_data)?;
            Some(tlk::Tlk::from_bytes(&tlk_data).unwrap().1)
        } else {
            None
        };

        let user_tlk = if let Some(user_path) = user_path {
            let mut tlk_data = vec![];
            File::open(user_path)?.read_to_end(&mut tlk_data)?;
            Some(tlk::Tlk::from_bytes(&tlk_data).unwrap().1)
        } else {
            None
        };

        Ok(tlk::Resolver::new(base_tlk, base_f_tlk, user_tlk))
    }
}

impl<'a> crate::cplugin::CPlugin<'a> for XPTlk {
    fn new(info: crate::cplugin::InitInfo) -> Result<Self, Box<dyn Error>> {
        let log_path: PathBuf = [info.nwnx_path, "xp_tlk.txt"].iter().collect();

        Logger::try_with_env_or_str("trace")
            .unwrap_or(Logger::with(LogSpecification::info()))
            .log_to_file(FileSpec::try_from(log_path).unwrap())
            .format(flexi_logger::detailed_format)
            .start()?;

        Ok(Self {
            resolvers: HashMap::new(),
            nwnx_path: info.nwnx_path.to_string(),
            nwn2_install_path: info.nwn2_install_path.to_string(),
            nwn2_home_path: info.nwn2_home_path.to_string(),
        })
    }
    fn get_id(&mut self) -> &'a CStr {
        &CStr::from_bytes_with_nul("tlk-rs\0".as_bytes()).unwrap()
    }
    fn get_info() -> &'static CStr {
        &CStr::from_bytes_with_nul("Load additional TLK files and resolve strrefs\0".as_bytes())
            .unwrap()
    }
    fn get_version() -> &'static CStr {
        use git_version::git_version;
        &CStr::from_bytes_with_nul(git_version!(suffix = "\0").as_bytes()).unwrap()
    }
    fn get_int(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
    ) -> Result<i32, Box<dyn Error>> {
        match function {
            "load" => {
                let mut args = param1.split("\n");
                let key = args.next().unwrap_or("");
                let base_path = self.replace_path_tokens(args.next().unwrap_or(""));
                let base_f_path = args
                    .next()
                    .filter(|s| s.len() > 0)
                    .and_then(|s| Some(self.replace_path_tokens(s)));
                let user_path = args
                    .next()
                    .filter(|s| s.len() > 0)
                    .and_then(|s| Some(self.replace_path_tokens(s)));

                match Self::load_resolver(&base_path, base_f_path.as_deref(), user_path.as_deref())
                {
                    Ok(resolver) => {
                        // TODO: replace string portions with known paths
                        self.resolvers.insert(key.to_string(), resolver);
                        log::info!(
                            "Loaded new resolver {:?} with:\n\tBase TLK: {}\n\tBaseF TLK: {}\n\tUser TLK: {}",
                            key,
                            base_path,
                            base_f_path.as_deref().unwrap_or("<None>"),
                            user_path.as_deref().unwrap_or("<None>"),
                        );
                        Ok(1)
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to load resolver {:?}: {}\n\tBase TLK: {}\n\tBaseF TLK: {}\n\tUser TLK: {}",
                            key,
                            e,
                            base_path,
                            base_f_path.as_deref().unwrap_or("<None>"),
                            user_path.as_deref().unwrap_or("<None>"),
                        );
                        Ok(0)
                    }
                }
            }
            "is_loaded" => {
                let key = param1;
                Ok(self.resolvers.contains_key(key) as i32)
            }
            "unload" => {
                let key = param1;
                let val = self.resolvers.remove(key);
                Ok(val.is_some() as i32)
            }
            "get_lang" => {
                let key = param1;
                let tlk_index = param2;

                let resolv: &tlk::Resolver =
                    self.resolvers.get(key).ok_or(Box::new(NWNXError {
                        msg: format!("No resolver with key: {:?}", key),
                    }))?;

                let tlk: &tlk::Tlk = match tlk_index {
                    0 => &resolv.base,
                    1 => resolv.base_f.as_ref().ok_or(Box::new(NWNXError {
                        msg: format!("No base_f TLK for resolver {:?}", key),
                    }))?,
                    2 => resolv.user.as_ref().ok_or(Box::new(NWNXError {
                        msg: format!("No user TLK for resolver {:?}", key),
                    }))?,
                    _ => {
                        return Err(Box::new(NWNXError {
                            msg: format!("Invalid tlk index: {}", tlk_index),
                        }))
                    }
                };
                Ok(tlk.header.language_id.try_into()?)
            }
            "get_flags" => {
                let key = param1;
                let strrefgender = param2;

                Ok(self
                    .get_tlk_entry(key, strrefgender)?
                    .map_or(0, |entry| entry.flags)
                    .try_into()?)
            }
            _ => Err(Box::new(NWNXError {
                msg: format!("Unknown function: {:?}", function),
            })),
        }
    }

    fn get_float(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
    ) -> Result<f32, Box<dyn Error>> {
        match function {
            "get_sound_length" => {
                let key = param1;
                let strrefgender = param2;

                if let Some(entry) = self.get_tlk_entry(key, strrefgender)? {
                    if (entry.flags & tlk::EntryFlag::HasSndLen as u32) > 0 {
                        Ok(entry.sound_length)
                    } else {
                        Ok(0.0)
                    }
                } else {
                    Ok(0.0)
                }
            }
            _ => Err(Box::new(NWNXError {
                msg: format!("Unknown function: {:?}", function),
            })),
        }
    }

    fn get_str(
        &mut self,
        function: &str,
        param1: &str,
        param2: i32,
    ) -> Result<&str, Box<dyn Error>> {
        match function {
            "get" => {
                let key = param1;
                let strrefgender = param2;

                Ok(self.get_tlk_str(key, strrefgender)?.unwrap_or(""))
            }
            "get_sound_resref" => {
                let key = param1;
                let strrefgender = param2;

                if let Some(entry) = self.get_tlk_entry(key, strrefgender)? {
                    if (entry.flags & tlk::EntryFlag::HasSnd as u32) > 0 {
                        Ok(entry.sound_resref.as_str())
                    } else {
                        Ok("")
                    }
                } else {
                    Ok("")
                }
            }
            _ => Err(Box::new(NWNXError {
                msg: format!("Unknown function: {:?}", function),
            })),
        }
    }
}

// Implement the nwnx4 CPlugin ABI and forward to the XPTlk class
cplugin_hook!(
    XPTlk,
    [GetID, GetInfo, GetVersion, GetInt, GetFloat, GetString]
);
