use super::Harmony;

// IMPROVEMENT: Into systems should probably return a value for use in execution, and another for
// the manifest (instead of doing them both together) to avoid this conversion step (and maybe
// allow for better compiler opt)
#[doc(hidden)]
#[cfg(feature = "generate_manifest")]
pub fn __internal_generate_manifest(harmony: Harmony) {
    use super::FeatureBuilder;
    use crate::ecs::system::Descriptors;
    use harmony_modloader_api as api;

    fn new_descriptor(
        (id, descriptor): (api::StableId<'static>, Descriptors),
    ) -> api::ScheduleDescriptor<'static> {
        let Descriptors { sets, systems } = descriptor;
        let systems = systems.into_iter().map(|(desc, _)| desc).collect();
        api::ScheduleDescriptor { id, sets, systems }
    }

    fn new_feature(feature: FeatureBuilder) -> api::FeatureDescriptor<'static> {
        let FeatureBuilder {
            name,
            resources,
            descriptors,
        } = feature;
        let descriptors = descriptors.into_iter().map(new_descriptor).collect();
        api::FeatureDescriptor {
            name,
            resources,
            descriptors,
        }
    }

    let manifest = api::ModManifest {
        wasm_hash: api::FileHash::empty(),
        features: harmony.features.into_iter().map(new_feature).collect(),
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
