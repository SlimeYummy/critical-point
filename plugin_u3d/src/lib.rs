#![feature(try_blocks)]

extern crate anyhow;
extern crate core;
extern crate simplelog;
#[macro_use]
extern crate log;
extern crate math;

use anyhow::Result;
use core::agent::SyncAgent;
use core::engine::DataPool;
use core::ffi::{FFIArray, FFISlice};
use core::id::ResID;
use core::resource::ResCache;
use libc::{c_char, c_uint};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::ffi::CStr;
use std::fs::File;
use std::sync::Arc;
use std::{mem, ptr};

#[no_mangle]
unsafe extern "stdcall" fn init_logger(log_path: *const c_char) -> bool {
    let res: Result<()> = try {
        let log_path = CStr::from_ptr(log_path).to_str()?;
        let log_file = File::create(log_path)?;
        WriteLogger::init(LevelFilter::Debug, Config::default(), log_file)?;
    };
    if let Err(err) = res {
        error!("init_logger() => {:?}", err);
        return false;
    }
    return true;
}

#[no_mangle]
unsafe extern "stdcall" fn create_res_cache(
    root_path: *const c_char,
    res_file: *const c_char,
    id_file: *const c_char,
) -> *const ResCache {
    let res: Result<Arc<ResCache>> = try {
        let root_path = CStr::from_ptr(root_path).to_str()?;
        let res_file = CStr::from_ptr(res_file).to_str()?;
        let id_file = CStr::from_ptr(id_file).to_str()?;
        ResCache::restore(root_path, res_file, id_file)?
    };
    match res {
        Ok(cache) => {
            let cache = Arc::into_raw(cache);
            info!("create_res_cache() => {:?}", cache);
            return cache;
        }
        Err(err) => {
            error!("create_res_cache() => {:?}", err);
            return ptr::null_mut();
        }
    }
}

#[no_mangle]
unsafe extern "stdcall" fn destroy_res_cache(cache: *const ResCache) {
    if cache.is_null() {
        error!("destroy_res_cache() => ResCache is null");
        return;
    }
    Arc::from_raw(cache);
    info!("destroy_res_cache() => {:?}", cache);
}

#[no_mangle]
unsafe extern "stdcall" fn create_sync_agent(
    cache: *const ResCache,
    fps: c_uint,
    init_id: *const c_char,
) -> *mut SyncAgent {
    if cache.is_null() {
        error!("create_sync_agent() => ResCache is null");
        return ptr::null_mut();
    }
    let cache = Arc::from_raw(cache);
    mem::forget(cache.clone());
    let res: Result<SyncAgent> = try {
        let init_id = CStr::from_ptr(init_id).to_str()?;
        SyncAgent::new(cache, fps, ResID::from(init_id))?
    };
    match res {
        Ok(agent) => {
            let agent = Box::into_raw(Box::new(agent));
            info!("create_sync_agent() => {:?}", agent);
            return agent;
        }
        Err(err) => {
            error!("create_sync_agent() => {:?}", err);
            return ptr::null_mut();
        }
    }
}

#[no_mangle]
unsafe extern "stdcall" fn destroy_sync_agent(agent: *mut SyncAgent) {
    if agent.is_null() {
        error!("destroy_sync_agent() => SyncAgent is null");
        return;
    }
    Box::from_raw(agent);
    info!("destroy_sync_agent() => {:?}", agent);
}

#[no_mangle]
unsafe extern "stdcall" fn sync_agent_update(agent: *mut SyncAgent) -> *mut DataPool {
    if agent.is_null() {
        error!("sync_agent_update() => SyncAgent is null");
        return ptr::null_mut();
    }

    match (*agent).update() {
        Err(err) => {
            error!("sync_agent_update() => {:?}", err);
            return ptr::null_mut();
        }
        Ok(pool) => return Box::into_raw(Box::new(pool)),
    }
}

#[no_mangle]
unsafe extern "stdcall" fn free_data_pool(pool: *mut DataPool) {
    if !pool.is_null() {
        Box::from_raw(pool);
    }
}

// #[no_mangle]
// unsafe extern "stdcall" fn sync_enter_scene(cmd_id: *const c_char) -> bool {
//     if SYNC_AGENT.is_null() {
//         error!("sync_enter_scene() => SyncAgent is null");
//         return false;
//     }
//     let res: Result<()> = try {
//         let cmd_id = CStr::from_ptr(cmd_id).to_str()?;
//         let agent = &mut *SYNC_AGENT;
//         agent.enter_scene(cmd_id);
//     };
//     if let Err(err) = res {
//         error!("sync_enter_scene() => {:?}", err);
//         return false;
//     }
//     return true;
// }

// #[no_mangle]
// unsafe extern "stdcall" fn sync_leave_scene() {
//     if SYNC_AGENT.is_null() {
//         error!("sync_leave_scene() => SyncAgent is null");
//         return;
//     }
//     let agent = &mut *SYNC_AGENT;
//     agent.leave_scene();
// }
