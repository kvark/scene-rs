use serde::de;
use std::collections::TreeMap;

/*pub struct Type(pub uint);

impl<D: de::Deserializer<E>, E> de::Deserializable<D, E> for Type {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<Type, E> {
        let val =
    }
}*/

#[deriving_deserializable]
pub struct Buffer {
    pub uri: String,
    pub byteLength: uint,
    pub name: String,
    //pub type: String,
}

#[deriving_deserializable]
pub struct Accessor {
    pub bufferView: String,
    pub byteOffset: uint,
    pub byteStride: uint,
    pub componentType: uint,
    pub count: uint,
    //pub type: uint,
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
    //pub type: uint,
}
