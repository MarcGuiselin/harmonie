use futures::{join, StreamExt};
use harmony_modloader_api as api;
use std::io::Result;
use std::path::PathBuf;
use std::process::Stdio;

mod command;
use command::CargoCommand;

pub async fn build(release: bool, directory: PathBuf, packages: Vec<String>) -> Result<()> {
    // Clear/prep harmony-build directory
    let temp_dir = directory.join("target/harmony-build/temp");
    let target_dir: PathBuf = directory.join(if release {
        "target/harmony-build/release"
    } else {
        "target/harmony-build/debug"
    });
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let _ = tokio::fs::remove_dir_all(&target_dir).await;
    tokio::fs::create_dir_all(&temp_dir).await?;
    tokio::fs::create_dir_all(&target_dir).await?;

    // Build a debug release of mods for manifest generation
    build_raw(
        directory.clone(),
        packages.clone(),
        BuildType::GenerateManifest,
    )
    .await?;

    // Move generated wasm files to a temporary directory
    let build_dir = directory.join("target/wasm32-unknown-unknown/debug");
    for package in packages.iter() {
        let filename = format!("{}.wasm", package);
        let from = build_dir.join(&filename);
        let to = temp_dir.join(&filename);
        tokio::fs::rename(from, to).await?;
    }

    // 1. Generate the manifest for each mod
    let generate_manifests_fut =
        generate_manifests(temp_dir.clone(), target_dir.clone(), packages.clone());

    // 2. Generate the wasm files for each mod
    let generate_wasm_fut = generate_wasm(
        release,
        directory.clone(),
        target_dir.clone(),
        packages.clone(),
    );

    // Do 1 and 2 concurrently
    let (result1, result2) = join![generate_manifests_fut, generate_wasm_fut];

    result1?;
    result2?;

    Ok(())
}

async fn generate_manifests(
    temp_dir: PathBuf,
    target_dir: PathBuf,
    packages: Vec<String>,
) -> Result<()> {
    let encoded_manifests = futures::stream::iter(
        packages
            .iter()
            .map(|package| temp_dir.join(format!("{}.wasm", package)))
            .map(|path| tokio::spawn(wasm_export_encoded_manifest(path))),
    )
    .buffer_unordered(4)
    .map(|r| r?)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;

    for (package, encoded_manifest) in packages.iter().zip(encoded_manifests) {
        let manifest: api::ModManifest<'_> = bitcode::decode(&encoded_manifest).unwrap();
        let as_string = format!("{:#?}", manifest);
        let path = target_dir.join(format!("{}.manifest.txt", package));
        tokio::fs::write(&path, as_string).await?;

        let path = target_dir.join(format!("{}.manifest", package));
        tokio::fs::write(&path, encoded_manifest).await?;
    }

    Ok(())
}

async fn generate_wasm(
    release: bool,
    directory: PathBuf,
    target_dir: PathBuf,
    packages: Vec<String>,
) -> Result<()> {
    let build_type = if release {
        BuildType::Release
    } else {
        BuildType::Debug
    };
    build_raw(directory.clone(), packages.clone(), build_type).await?;

    // Write manifest files to release directory
    let build_dir = directory.join(if release {
        "target/wasm32-unknown-unknown/release"
    } else {
        "target/wasm32-unknown-unknown/debug"
    });
    for package in packages.iter() {
        let filename = format!("{}.wasm", package);
        let from = build_dir.join(&filename);
        let to = target_dir.join(&filename);
        tokio::fs::rename(from, to).await?;
    }

    Ok(())
}

enum BuildType {
    Debug,
    Release,
    GenerateManifest,
}

async fn build_raw(directory: PathBuf, packages: Vec<String>, build_type: BuildType) -> Result<()> {
    let mut command = CargoCommand::new("build");
    command
        .packages(packages.into_iter())
        .inner
        .current_dir(directory)
        .args(&["--target", "wasm32-unknown-unknown"])
        .env("RUSTFLAGS", "-C link-arg=--import-memory")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match build_type {
        BuildType::Debug => {}
        BuildType::Release => {
            command.inner.arg("--release");
        }
        BuildType::GenerateManifest => {
            command.inner.args(&["--features", "generate_manifest"]);
        }
    };

    if command.inner.spawn()?.wait().await?.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to build mods",
        ))
    }
}

struct State {
    component_id_counter: u32,
    memory: wasmer::Memory,
    encoded_manifest: Option<Vec<u8>>,
}
type Context<'a> = wasmer::FunctionEnvMut<'a, State>;

fn reserve_component_id(mut context: Context) -> u32 {
    let state = context.data_mut();
    state.component_id_counter += 1;
    state.component_id_counter
}

fn submit_manifest(mut context: Context, manifest_ptr: u64) {
    let pointer: api::WasmPointer = manifest_ptr.into();

    let (data, store) = context.data_and_store_mut();

    let memory_view = data.memory.view(&store);
    let encoded = memory_view
        .copy_range_to_vec(pointer.into())
        .expect("failed to copy range");

    data.encoded_manifest = Some(encoded);
}

async fn wasm_export_encoded_manifest(path: PathBuf) -> Result<Vec<u8>> {
    let bytes = tokio::fs::read(&path).await?;

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
        encoded_manifest: None,
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

    let encoded = env
        .as_mut(&mut store)
        .encoded_manifest
        .take()
        .expect("mod never called submit_manifest");

    Ok(encoded)
}
