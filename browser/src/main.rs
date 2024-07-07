use harmony_modding_api as api;
use std::io::Result;
use std::process::Stdio;

mod command;
use command::CargoCommand;

struct State {
    component_id_counter: u32,
    memory: wasmer::Memory,
}
type Context<'a> = wasmer::FunctionEnvMut<'a, State>;

fn reserve_component_id(mut context: Context) -> u32 {
    let state = context.data_mut();
    state.component_id_counter += 1;
    state.component_id_counter
}

fn submit_manifest(mut context: Context, manifest_ptr: u64) {
    let pointer: api::WasmPointer = manifest_ptr.into();
    println!("submit_manifest pointer: {:?}", pointer);

    let (data, store) = context.data_and_store_mut();

    let memory_view = data.memory.view(&store);
    let encoded = memory_view
        .copy_range_to_vec(pointer.into())
        .expect("failed to copy range");

    let manifest: api::ModManifest<'_> = bitcode::decode(&encoded).unwrap();
    println!("{:?}", manifest);
}

#[tokio::main]
async fn main() -> Result<()> {
    // Build a debug release of mods for manifest generation
    let ex = CargoCommand::new("build")
        .packages(&["the_cube"])
        .command
        .args(&["--target", "wasm32-unknown-unknown"])
        .args(&["--features", "generate_manifest"])
        .env("RUSTFLAGS", "-C link-arg=--import-memory")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait()
        .await?;

    // Load using wasmer
    if ex.success() {
        println!("Loading the_cube.wasm");

        // Read the Wasm bytes from the file.
        let bytes = std::fs::read("./target/wasm32-unknown-unknown/debug/the_cube.wasm")
            .expect("Wasm file not found");

        // Create a Store.
        let mut store = wasmer::Store::default();

        // We then use our store and Wasm bytes to compile a `Module`.
        // A `Module` is a compiled WebAssembly module that isn't ready to execute yet.
        let module = wasmer::Module::new(&store, bytes).expect("invalid wasm");

        // Initiate shared memory pool
        let memory = wasmer::Memory::new(&mut store, wasmer::MemoryType::new(17, None, false))
            .expect("wasm memory allocation failed");

        let state = State {
            component_id_counter: 0,
            memory: memory.clone(),
        };
        let env = wasmer::FunctionEnv::new(&mut store, state);
        let import_object = wasmer::imports! {
            "harmony_mod" => {
                "reserve_component_id" => wasmer::Function::new_typed_with_env(&mut store, &env, reserve_component_id),
                "submit_manifest" => wasmer::Function::new_typed_with_env(&mut store, &env, submit_manifest),
            },
            "env" => {
                "memory" => memory,
            },
        };

        // We then use the `Module` and the import object to create an `Instance`.
        //
        // An `Instance` is a compiled WebAssembly module that has been set up
        // and is ready to execute.
        let instance = wasmer::Instance::new(&mut store, &module, &import_object)
            .expect("wasm instantiation failed");

        let init: wasmer::TypedFunction<(), ()> = instance
            .exports
            .get_typed_function(&store, "harmony_mod_generate_manifest")
            .expect("could not find harmony_mod_generate_manifest function");

        init.call(&mut store)
            .expect("failed to call harmony_mod_generate_manifest");
    }

    Ok(())
}
