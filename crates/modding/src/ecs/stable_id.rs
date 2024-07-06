use harmony_modding_api as api;

/// A id shared between mods, used to identify objects defined in the manifest
pub trait StableId {
    const CRATE_NAME: &'static str;
    const VERSION: &'static str;
    const NAME: &'static str;

    fn get_stable_id(&self) -> api::StableId<'static> {
        api::StableId {
            crate_name: Self::CRATE_NAME,
            version: Self::CRATE_NAME,
            name: Self::CRATE_NAME,
        }
    }
}

pub struct StableIdWithData<T> {
    crate_name: &'static str,
    version: &'static str,
    name: &'static str,
    pub data: T,
}

impl<T> StableIdWithData<T> {
    pub fn new<S: StableId>(data: T) -> StableIdWithData<T> {
        StableIdWithData {
            crate_name: S::CRATE_NAME,
            version: S::VERSION,
            name: S::NAME,
            data,
        }
    }
}
