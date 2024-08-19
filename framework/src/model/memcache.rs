use std::collections::HashMap;
use memcache::{MemcacheError, Client, ToMemcacheValue, Stream, FromMemcacheValueExt};
use anyhow::anyhow;

pub enum MemcacheServerIndex {
    CachServerLocal,
    CachServerDell1,
    CachServerDell2,
    CachServerNode1,
    CachServerNode2,
    CachServerNode3,
    CachServerNode4,
}
const MEMCACHE_SERVER: &str = "memcache://192.168.12.6:11211?timeout=10&tcp_nodelay=true";
const MEMCACHE_SERVER1: &str = "memcache://192.168.12.6:11211?timeout=10&tcp_nodelay=true";

fn get_service_host() -> String {
    std::env::var("MEMCACHE_SERVER")
        .unwrap_or_else(|_| MEMCACHE_SERVER.to_string())
}

pub fn memcache_pool() -> Result<Client, MemcacheError>{
    let pool = vec![MEMCACHE_SERVER1];

    let client = memcache::Client::with_pool_size(pool, 4)?;

    Ok(client)
}
pub fn memcache_connect() -> Result<Client, MemcacheError>{
    let address = get_service_host();

    let client = memcache::connect(address)?;

    Ok(client)
}
pub fn memcache_read(client: &Client, key: &str) -> Result<Option<String>, MemcacheError>{
    let value:Option<String> = client.get(key)?;

    Ok(value)
}
pub fn memcache_multiread(client: &Client, keys: Vec<&str>) -> Result<HashMap<String, String>, MemcacheError>{
    let values = client.gets(&keys)?;

    Ok(values)
}
pub fn memcache_read_counter(client: &Client, key: &str) -> Result<Option<u64>, MemcacheError>{
    let value:Option<u64> = client.get(key)?;

    Ok(value)
}
pub fn memcache_write<V: ToMemcacheValue<Stream>>(client: &Client, key: &str, value: V, method: &str) -> Result<(), MemcacheError>{
    match method {
        "set" => client.set(key, value, 0)?,
        "prepend" => client.prepend(key, value)?,
        "append" => client.append(key, value)?,
        "counter" => {
            let _num = client.increment(key, 1)?;
            return Ok(());
        }
        &_ => ()
    }
    
    Ok(())
}
pub fn memcache_cas_write<V: ToMemcacheValue<Stream> + FromMemcacheValueExt>(client: &Client, key: &str, value: V) -> Result<(), anyhow::Error>{
    let result: HashMap<String, (Vec<u8>, u32, Option<u64>)> = client.gets(&[key])?;
    if let Some((_, _, cas)) = result.get(key){
        if let Some(cas) = cas{
            let _data = client.cas(key, value, 10, *cas)?;
        }else{
            return Err(anyhow!("parse error"));
        }
    }else{
        return Err(anyhow!("parse error"));
    }
    
    Ok(())
}
pub fn memcache_delete(client: &Client, flush: bool, key: &str) -> Result<(), MemcacheError>{
    client.delete(key)?;
    if flush {
        client.flush()?;
    }

    Ok(())
}
