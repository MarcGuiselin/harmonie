// Common library for modding
mod Harmony {
    use std::marker::PhantomData;

    // The init function of a mod serves several purposes, including initializing the system execution runtime
    //
    // The different necessary
    enum Visitor {
        // Generate the manifest used by harmony to build the master schedule
        GenerateManifest,
        // Find assets, so they can be packed with the mod when it is shared
        SaveDependencies,
        // Initialize system runner
        //
        // Note: After initialization, code produced by a macro is responsible for actually running systems
        InitializeRuntime,
    }

    // Implementation of a visitor pattern for the engine
    //
    // The init function of a mod serves several purposes, including initializing the system execution runtime
    //
    pub trait EngineVisitor {
        fn new() -> Self;

        fn add_translations(&mut self, _path: &str) -> &mut Self;
    }

    /// A default implementation so user can write out the init function without defining the generic type of [`Engine`]
    impl EngineVisitor for () {
        fn new() -> Self {
            unreachable!()
        }

        fn add_translations(&mut self, _path: &str) -> &mut Self {
            unreachable!()
        }
    }

    pub struct Engine<V: EngineVisitor = ()> {
        visitor: V,
    }

    impl Engine {
        pub fn add_translations(&mut self, path: &str) -> &mut Self {
            self.visitor.add_translations(path);
            self
        }

        pub fn add_feature<Feature>(&mut self) -> &mut Self {
            unimplemented!()
        }
    }

    pub trait Feature{
        fn 
    }
}

mod ModA {
    use super::Harmony;
    use bevy::prelude::*;
    use bitcode::{Decode, Encode};

    // #[macro::init]
    fn init(harmony: &mut Harmony::Engine) {
        harmony
            // TODO: implement https://github.com/projectfluent/fluent-rs
            .add_translations("./assets/file/*.ftl")
            .add_feature(the_cube_of_truth);
    }

    // public so other mod can import
    pub struct TheCubeOfTruth;

impl Feature for TheCubeOfTruth {

    fn build(&self, feature: &mut App) {
        // add things to your app here
    }
}

    fn the_cube_of_truth(feature: ) {

        
    }
}
