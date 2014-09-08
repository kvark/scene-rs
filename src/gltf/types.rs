use std::collections::TreeMap;

#[deriving_deserializable]
pub struct Buffer {
    pub uri: String,
    #[serial_name = "byteLength"]
    pub byte_length: uint,
    pub name: String,
    #[serial_name = "type"]
    pub ty: String,
}

#[deriving_deserializable]
pub struct Accessor {
    #[serial_name = "bufferView"]
    pub buffer_view: String,
    #[serial_name = "byteOffset"]
    pub byte_offset: uint,
    #[serial_name = "byteStride"]
    pub byte_stride: uint,
    #[serial_name = "componentType"]
    pub component_type: uint,
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

#[deriving_deserializable]
pub struct Program {
    pub name: String,
    pub attributes: Vec<String>,
    #[serial_name = "vertexShader"]
    pub vertex_shader: String,
    #[serial_name = "fragmentShader"]
    pub fragment_shader: String,
}
