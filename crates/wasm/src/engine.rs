use anyhow::Result;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};
use tracing::{debug, info};
use wasmtime::{Config, Engine, InstanceAllocationStrategy};

// global engine hashmap with string key with sync mutex
lazy_static! {
    pub static ref ENGINE_MAP: Mutex<HashMap<String, Engine>> = Mutex::new(HashMap::new());
}

/// MODULE_VERSION is the module version
pub const MODULE_VERSION: &str = "w20";

// 10 ms to trigger epoch increment
pub const EPOCH_INC_INTERVAL: u64 = 10;

/// init_engines initialize default engine
pub fn init_engines() -> Result<()> {
    // try use std to run this loop. not tokio
    std::thread::spawn(|| {
        increment_epoch_loop_inner();
    });
    let _ = get("default")?;
    Ok(())
}

/// increment_epoch_loop
fn increment_epoch_loop_inner() {
    loop {
        // if ENGINE_MAP is empty, sleep 3 seconds to wait for new engine
        if ENGINE_MAP.lock().unwrap().is_empty() {
            debug!("ENGINE_MAP is empty, sleep 3 seconds to wait for new engine");
            std::thread::sleep(std::time::Duration::from_secs(3));
            continue;
        }

        // iterate ENGINE_MAP to increment epoch for every EPOCH_INC_INTERVAL ms
        for (_, engine) in ENGINE_MAP.lock().unwrap().iter() {
            engine.increment_epoch();
        }
        // use std thread to sleep 3 seconds
        std::thread::sleep(std::time::Duration::from_millis(EPOCH_INC_INTERVAL));
    }
}

fn create_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config.epoch_interruption(true);
    config.module_version(wasmtime::ModuleVersionStrategy::Custom(
        MODULE_VERSION.to_string(),
    ))?;

    const MB: usize = 1 << 20;
    const KB: usize = 1 << 10;

    // set 512MB memory static allocation when starting
    // in 64-bit mode, wasmtime default maximum memory size is 4GB
    config.static_memory_maximum_size(512 * MB as u64);
    config.static_memory_guard_size(512 * MB as u64);
    config.dynamic_memory_reserved_for_growth(1 * MB as u64);
    config.dynamic_memory_guard_size(64 * KB as u64);

    // SIMD support requires SSE3 and SSSE3 on x86_64.
    // in docker container, it will cause error
    // config.wasm_simd(false);

    // let mut pooling_allocation_config = PoolingAllocationConfig::default();

    // Core wasm programs have 1 memory
    // pooling_allocation_config.max_memories_per_module(1);
    // Total memory size 64 MB, allow for up to memory_limit of linear memory. Wasm pages are 64KB
    // pooling_allocation_config.memory_pages(1000);

    // Core wasm programs have 1 table
    // pooling_allocation_config.max_tables_per_module(1);

    // Some applications create a large number of functions, in particular
    // when compiled in debug mode or applications written in swift. Every
    // function can end up in the table
    // pooling_allocation_config.table_elements(10000);

    // Maximum number of slots in the pooling allocator to keep "warm", or those
    // to keep around to possibly satisfy an affine allocation request or an
    // instantiation of a module previously instantiated within the pool.
    // pooling_allocation_config.max_unused_warm_slots(100);

    // Use a large pool, but one smaller than the default of 1000 to avoid runnign out of virtual
    // memory space if multiple engines are spun up in a single process. We'll likely want to move
    // to the on-demand allocator eventually for most purposes; see
    // https://github.com/fastly/Viceroy/issues/255
    // pooling_allocation_config.total_core_instances(1000);

    config.allocation_strategy(InstanceAllocationStrategy::pooling());

    debug!("Create new config: {:?}", config);
    
    Ok(config)
}

/// get engine by key
pub fn get(key: &str) -> Result<Engine> {
    let mut map = ENGINE_MAP.lock().unwrap();
    if map.contains_key(key) {
        return Ok(map.get(key).unwrap().clone());
    }
    info!("Create new engine for key: {}", key);
    let config = create_config()?;
    let engine = Engine::new(&config).unwrap();
    map.insert(key.to_string(), engine.clone());
    Ok(engine)
}
