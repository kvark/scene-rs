use std::collections::TreeMap;
use gfx;
use gfx_gl::types::GLenum;

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
    pub byte_offset: gfx::attrib::Offset,
    #[serial_name = "byteStride"]
    pub byte_stride: gfx::attrib::Stride,
    #[serial_name = "componentType"]
    pub component_type: GLenum,
    pub count: gfx::VertexCount,
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
    pub ty: GLenum,
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

#[deriving_deserializable]
pub struct TechniqueParameter {
    #[serial_name = "type"]
    pub ty: uint,
    pub semantic: String,   //optional
    pub node: String,       //optional
    pub value: Vec<f32>,    //optional
}

#[deriving_deserializable]
pub struct InstanceProgram {
    pub attributes: TreeMap<String, String>,
    pub program: String,
    pub uniforms: TreeMap<String, String>,
}

#[deriving_deserializable]
pub struct StateFunctions {
    #[serial_name = "blendColor"]
    pub blend_color: (f32, f32, f32, f32),
    #[serial_name = "blendEquationSeparate"]
    pub blend_equation_separate: (GLenum, GLenum),
    #[serial_name = "blendFuncSeparate"]
    pub blend_func_separate: (GLenum, GLenum, GLenum, GLenum),
    #[serial_name = "colorMask"]
    pub color_mask: (bool, bool, bool, bool),
    #[serial_name = "cullFace"]
    pub cull_face: (GLenum, ),
    #[serial_name = "depthFunc"]
    pub depth_func: (GLenum, ),
    #[serial_name = "depthMask"]
    pub depth_mask: (bool, ),
    #[serial_name = "depthRange"]
    pub depth_range: (f32, f32),
    #[serial_name = "frontFace"]
    pub front_face: (GLenum, ),
    #[serial_name = "lineWidth"]
    pub line_width: (gfx::state::LineWidth, ),
    #[serial_name = "polygonOffset"]
    pub polygon_offset: (gfx::state::OffsetFactor, gfx::state::OffsetUnits),
    pub scissor: (u16, u16, u16, u16),
}

#[deriving_deserializable]
pub struct States {
    pub enable: Vec<GLenum>,
    pub functions: StateFunctions,
}

#[deriving_deserializable]
pub struct Pass {
    pub details: (),
    #[serial_name = "instanceProgram"]
    pub instance_program: InstanceProgram,
    pub states: States,
}

#[deriving_deserializable]
pub struct Technique {
    pub name: String,
    pub parameters: TreeMap<String, TechniqueParameter>,
    pub pass: String,
    pub passes: TreeMap<String, Pass>,
}
