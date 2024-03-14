use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};
use wasmtime::{Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig};

// global engine hashmap with string key with sync mutex
lazy_static! {
    pub static ref ENGINE_MAP: Mutex<HashMap<String, Engine>> = Mutex::new(HashMap::new());
}

fn create_config() -> Config {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);

    // SIMD support requires SSE3 and SSSE3 on x86_64.
    // in docker container, it will cause error
    // config.wasm_simd(false);

    // const MB: usize = 1 << 20;
    // let mut pooling_allocation_config = PoolingAllocationConfig::default();
    // pooling_allocation_config.max_core_instance_size(MB);
    // pooling_allocation_config.max_memories_per_component(128 * (MB as u32) / (64 * 1024));
    let pooling_allocation_config = PoolingAllocationConfig::default();
    config.allocation_strategy(InstanceAllocationStrategy::Pooling(
        pooling_allocation_config,
    ));

    config
}

/// get engine by key
pub fn get(key: &str) -> Engine {
    let mut map = ENGINE_MAP.lock().unwrap();
    if map.contains_key(key) {
        return map.get(key).unwrap().clone();
    }
    let engine = Engine::new(&create_config()).unwrap();
    map.insert(key.to_string(), engine.clone());
    engine
}
