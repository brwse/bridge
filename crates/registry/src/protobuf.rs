pub mod registry {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/brwse.bridge.registry.v1.rs"));
    }
}
