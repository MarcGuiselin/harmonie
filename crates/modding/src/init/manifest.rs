use super::Harmony;

#[doc(hidden)]
#[cfg(feature = "generate_manifest")]
pub fn __internal_generate_manifest(engine: Harmony) {
    use crate::init::FeatureBuilder;
    use harmony_modloader_api as api;

    let features = engine
        .features
        .into_iter()
        .map(|feature| {
            let FeatureBuilder {
                name,
                resources,
                schedules,
            } = feature;
            api::FeatureDescriptor {
                name,
                resources,
                schedules,
            }
        })
        .collect();

    let manifest = api::ModManifest {
        wasm_hash: api::FileHash::empty(),
        features,
    };
    let encoded = bitcode::encode(&manifest);

    #[link(wasm_import_module = "harmony_mod")]
    extern "C" {
        fn submit_manifest(ptr: u64);
    }

    unsafe {
        let pointer = api::WasmPointer::from_vec(&encoded);
        submit_manifest(pointer.into());
    }
}

#[doc(hidden)]
#[cfg(not(feature = "generate_manifest"))]
pub fn __internal_generate_manifest(_: Harmony) {
    unreachable!()
}
