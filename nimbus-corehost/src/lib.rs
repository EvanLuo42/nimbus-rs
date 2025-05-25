#[cfg(windows)]
use std::ffi::OsString;

#[cfg(not(windows))]
use std::ffi::CStr;

use crate::errors::CoreHostError;
use std::path::PathBuf;
use std::ptr;

mod binding;
mod errors;

pub const PATH_MAX: usize = 512;

pub struct HostFxr {
    
}

impl HostFxr {
    pub fn load() -> Result<HostFxr, CoreHostError> {
        let hostfxr_path = get_hostfxr_path()?;
        todo!()
    }
    
    pub fn load_from_path() -> Result<HostFxr, CoreHostError> {
        todo!()
    }
}

#[cfg(windows)]
type CharT = u16;

#[cfg(not(windows))]
type CharT = u8;

fn get_hostfxr_path() -> Result<PathBuf, CoreHostError> {
    let mut buffer = vec![0 as CharT; PATH_MAX];
    let mut buffer_size = buffer.len();
    
    let result = unsafe {
        binding::get_hostfxr_path(
            buffer.as_mut_ptr().cast(),
            &mut buffer_size,
            ptr::null()
        )
    };
    
    if result != 0 {
        return Err(CoreHostError::BufferTooSmall)
    }
    
    #[cfg(windows)]
    {
        let wide = &buffer[..buffer_size];
        let os_string = OsString::from_vec(wide.to_vec());
        Ok(PathBuf::from(os_string))
    }
    
    #[cfg(not(windows))]
    {
        let raw = &buffer[..buffer_size];
        let c_str = CStr::from_bytes_with_nul(raw)
            .map_err(CoreHostError::from)?;
        Ok(PathBuf::from(c_str.to_str().unwrap()))
    }
}
