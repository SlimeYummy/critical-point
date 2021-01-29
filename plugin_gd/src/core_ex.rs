#![allow(dead_code)]

use anyhow::{anyhow, Result};
use core::agent::{AsyncLogicAgent, SyncLogicAgent};
use core::resource::ResCache;
use m::fi;
use std::ptr;
use std::sync::Arc;

static mut PTR_RES_CACHE: *const Arc<ResCache> = ptr::null_mut();
static mut PTR_SYNC_AGENT: *mut SyncLogicAgent = ptr::null_mut();
static mut PTR_ASYNC_AGENT: *mut AsyncLogicAgent = ptr::null_mut();

pub fn load_res_cache() -> Result<()> {
    if !unsafe { PTR_RES_CACHE.is_null() } {
        return Err(anyhow!("ResCache loaded"));
    }
    let res_cache = Box::new(ResCache::restore(
        "./critical_point/resource.yml",
        "./critical_point/id.yml",
    )?);
    unsafe { PTR_RES_CACHE = Box::into_raw(res_cache) };
    return Ok(());
}

#[allow(non_snake_case)]
pub fn RES_CACHE() -> &'static Arc<ResCache> {
    return unsafe { &*PTR_RES_CACHE };
}

pub fn init_sync_agent() -> Result<()> {
    if !unsafe { PTR_SYNC_AGENT.is_null() } {
        return Err(anyhow!("SyncLogicAgent inited"));
    }
    let agent = Box::new(SyncLogicAgent::new(RES_CACHE().clone(), fi(60))?);
    unsafe { PTR_SYNC_AGENT = Box::into_raw(agent) };
    return Ok(());
}

#[allow(non_snake_case)]
pub fn SYNC_AGENT() -> &'static mut SyncLogicAgent {
    return unsafe { &mut *PTR_SYNC_AGENT };
}

pub fn init_async_agent() -> Result<()> {
    if !unsafe { PTR_ASYNC_AGENT.is_null() } {
        return Err(anyhow!("AsyncLogicAgent inited"));
    }
    let agent = Box::new(AsyncLogicAgent::new(RES_CACHE().clone(), fi(60)));
    unsafe { PTR_ASYNC_AGENT = Box::into_raw(agent) };
    return Ok(());
}

#[allow(non_snake_case)]
pub fn ASYNC_AGENT() -> &'static AsyncLogicAgent {
    return unsafe { &*PTR_ASYNC_AGENT };
}
