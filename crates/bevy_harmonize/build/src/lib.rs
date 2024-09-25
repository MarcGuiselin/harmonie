use async_process::Stdio;
use async_std::{
    io::{prelude::BufReadExt, BufReader, Read},
    stream::StreamExt,
    task::spawn,
};
use bevy_utils::tracing::{error, info};
use futures_concurrency::prelude::*;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::{io::Result, time::Instant};

mod command;
use command::CargoCommand;

mod fs_utils;

pub async fn build(release: bool, directory: PathBuf, packages: Vec<String>) -> Result<()> {
    let start = Instant::now();
    info!("Building mods {:?}", packages);

    // Clear/prep harmony-build directory
    let temp_dir = directory.join("target/harmony-build/temp");
    let target_dir: PathBuf = directory.join(if release {
        "target/harmony-build/release"
    } else {
        "target/harmony-build/debug"
    });
    let _ = fs_utils::remove_dir_contents(&temp_dir).await;
    let _ = fs_utils::remove_dir_contents(&target_dir).await;
    async_fs::create_dir_all(&temp_dir).await?;
    async_fs::create_dir_all(&target_dir).await?;

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
        let src = build_dir.join(&filename);
        let dst = temp_dir.join(&filename);
        async_fs::rename(src, dst).await?;
    }

    // 1. Generate the manifest for each mod
    let generate_manifests_fut = generate_manifests(temp_dir.clone(), packages.clone());

    // 2. Generate the wasm files for each mod
    let generate_wasm_fut = generate_wasm(release, directory.clone(), packages.clone());

    // Do 1 and 2 concurrently
    let (result1, result2) = (generate_manifests_fut, generate_wasm_fut).join().await;
    let encoded_manifests = result1?;
    result2?;

    // Do final processing of manifest and write files to target directory
    packages
        .clone()
        .into_iter()
        .zip(encoded_manifests)
        .collect::<Vec<_>>()
        .into_co_stream()
        .map(|(package, encoded_manifest)| {
            let build_dir = directory.join(if release {
                "target/wasm32-unknown-unknown/release"
            } else {
                "target/wasm32-unknown-unknown/debug"
            });
            final_processing(package, encoded_manifest, build_dir, target_dir.clone())
        })
        .collect::<Vec<Result<()>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<()>>>()?;

    let duration = start.elapsed();
    info!("Successfully built mods {:?} in {:?}", packages, duration);

    Ok(())
}

async fn generate_manifests(temp_dir: PathBuf, packages: Vec<String>) -> Result<Vec<Vec<u8>>> {
    packages
        .into_co_stream()
        .map(|package| {
            let path = temp_dir.join(format!("{}.wasm", package));
            wasm_export_encoded_manifest(path)
        })
        .collect::<Vec<Result<Vec<u8>>>>()
        .await
        .into_iter()
        .collect()
}

async fn generate_wasm(release: bool, directory: PathBuf, packages: Vec<String>) -> Result<()> {
    let build_type = if release {
        BuildType::Release
    } else {
        BuildType::Debug
    };
    build_raw(directory.clone(), packages.clone(), build_type).await?;

    Ok(())
}

async fn final_processing(
    package: String,
    encoded_manifest: Vec<u8>,
    build_dir: PathBuf,
    target_dir: PathBuf,
) -> Result<()> {
    let wasm_path = build_dir.join(format!("{}.wasm", package));
    let wasm_bytes = async_fs::read(&wasm_path).await?;
    let wasm_hash = common::FileHash::from_sha256(Sha256::digest(&wasm_bytes).into());

    let old_manifest: common::ModManifest<'_> = bitcode::decode(&encoded_manifest).unwrap();
    let manifest = common::ModManifest {
        wasm_hash,
        features: old_manifest.features,
    };

    let as_string = format!("{:#?}", manifest);
    let path = target_dir.join(format!("{}.manifest.txt", package));
    async_fs::write(&path, as_string).await?;

    let encoded_manifest = bitcode::encode(&manifest);
    let path = target_dir.join(format!("{}.manifest", package));
    async_fs::write(&path, encoded_manifest).await?;

    // Move wasm file to target directory
    let src = wasm_path;
    let dst = target_dir.join(format!("{}.wasm", package));
    async_fs::rename(src, dst).await?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        .stderr(Stdio::piped());

    command
        .inner
        .arg("--features")
        .arg(if build_type == BuildType::GenerateManifest {
            "generate_manifest"
        } else {
            "wasm_runtime"
        });

    if build_type == BuildType::Release {
        command.inner.arg("--release");
    }

    let mut child = command.inner.spawn()?;

    // All human readable output for cargo is sent to stderr
    let stderr = child.stderr.take().unwrap();
    let stderr_handle = spawn(output_cargo_stderr(stderr));

    let (status, _) = (child.status(), stderr_handle).join().await;
    if status?.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to build mods",
        ))
    }
}

async fn output_cargo_stderr(output: impl Read + Unpin) {
    let reader = BufReader::new(output);
    let mut lines = reader.lines();

    let mut err = false;
    while let Some(line) = lines.next().await {
        let line = line.expect("Failed to read line");
        err |= line.contains("error");
        if err {
            error!("{}", line);
        } else {
            info!("{}", line);
        }
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
    let pointer: common::WasmPointer = manifest_ptr.into();

    let (data, store) = context.data_and_store_mut();

    let memory_view = data.memory.view(&store);
    let encoded = memory_view
        .copy_range_to_vec(pointer.into())
        .expect("failed to copy range");

    data.encoded_manifest = Some(encoded);
}

async fn wasm_export_encoded_manifest(path: PathBuf) -> Result<Vec<u8>> {
    let bytes = async_fs::read(&path).await?;

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
