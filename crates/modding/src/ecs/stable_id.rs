use harmony_modloader_api as api;

/// A id shared between mods, used to identify objects defined in the manifest
pub trait StableId {
    const CRATE_NAME: &'static str;
    const VERSION: &'static str;
    const NAME: &'static str;

    fn get_stable_id(&self) -> api::StableId<'static> {
        api::StableId {
            crate_name: Self::CRATE_NAME,
            version: Self::VERSION,
            name: Self::NAME,
        }
    }
}
