/// A id shared between mods, used to identify objects defined in the manifest
pub trait StableId {
    const CRATE_NAME: &'static str;
    const VERSION: &'static str;
    const NAME: &'static str;
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
