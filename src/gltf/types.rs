use serde::de;
use std::collections::TreeMap;

#[deriving_deserializable]
pub struct Buffer {
    pub uri: String,
    pub byteLength: uint,
    pub name: String,
    #[serial_name = "type"]
    pub ty: String,
}

#[deriving_deserializable]
pub struct Accessor {
    pub bufferView: String,
    pub byteOffset: uint,
    pub byteStride: uint,
    pub componentType: uint,
    pub count: uint,
    #[serial_name = "type"]
    pub ty: String,
    pub name: String,
    pub max: (f32, f32, f32),
    pub min: (f32, f32, f32),
}

#[deriving_deserializable]
pub struct Primitive {
    pub attributes: TreeMap<String, String>,
    pub indices: String,
    pub material: String,
    pub primitive: uint,
}

#[deriving_deserializable]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

#[deriving_deserializable]
pub struct Shader {
    pub name: String,
    pub uri: String,
    #[serial_name = "type"]
    pub ty: uint,
}
