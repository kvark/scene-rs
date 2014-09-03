use std::collections::TreeMap;

#[deriving(Decodable)]
pub struct Buffer {
    pub uri: String,
    pub byteLength: uint,
    pub name: String,
    //pub type: String,
}

#[deriving(Decodable)]
pub struct Accessor {
    pub bufferView: String,
    pub byteOffset: uint,
    pub byteStride: uint,
    pub componentType: uint,
    pub count: uint,
    //pub type: String,
    pub name: String,
    pub max: (f32, f32, f32),
    pub min: (f32, f32, f32),
}

#[deriving(Decodable)]
pub struct Primitive {
    pub attributes: TreeMap<String, String>,
    pub indices: String,
    pub material: String,
    pub primitive: uint,
}

#[deriving(Decodable)]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
}
