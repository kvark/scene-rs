//! GLTF package loader for gfx-rs

#![crate_name = "gltf"]
#![feature(phase)]

#[phase(plugin, link)]
extern crate log;
extern crate serialize;
extern crate cgmath;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate gfx_gl;
extern crate serde;
#[phase(plugin)]
extern crate serde_macros;

use std::cmp;
use std::collections::HashMap;
use std::io::File;
use serde::{de, json};
use gfx_gl::types::GLenum;

mod types;

fn load_map<T: de::Deserializable<json::JsonDeserializer, json::ParserError>, R>(
            json: &json::Json, var: &str, fun: |T| -> R) -> HashMap<String, R> {
    match json.find(&var.to_string()) {
        Some(&json::Object(ref map)) => map.iter().map(|(id, sub)| {
            let v: T = json::from_json(sub.clone()).unwrap();
            (id.clone(), fun(v))
        }).collect(),
        _ => HashMap::new(),
    }
}

fn attrib_to_slice(attrib: &gfx::Attribute)
                   -> Result<gfx::Slice, ()> {
    use gfx::attrib as a;
    match (attrib.format.elem_count, attrib.format.elem_type) {
        (1, a::Int(a::IntRaw, a::U8, a::Unsigned)) => Ok(gfx::IndexSlice8(
            gfx::TriangleList,
            gfx::BufferHandle::from_raw(attrib.buffer),
            0,
            attrib.format.offset >> 0
        )),
        (1, a::Int(a::IntRaw, a::U16, a::Unsigned)) => Ok(gfx::IndexSlice16(
            gfx::TriangleList,
            gfx::BufferHandle::from_raw(attrib.buffer),
            0,
            attrib.format.offset >> 1
        )),
        (1, a::Int(a::IntRaw, a::U32, a::Unsigned)) => Ok(gfx::IndexSlice32(
            gfx::TriangleList,
            gfx::BufferHandle::from_raw(attrib.buffer),
            0,
            attrib.format.offset >> 2
        )),
        _ => Err(())
    }
}

#[deriving(Clone, PartialEq, Show)]
pub enum AccessorCountError {
    AccessorMatrix(u8),
    AccessorUnknown(String),
}

fn parse_accessor_count(ty: &str) -> Result<gfx::attrib::Count, AccessorCountError> {
    use gfx::attrib as a;
    match ty {
        "SCALAR" => Ok(1),
        "VEC2"   => Ok(2),
        "VEC3"   => Ok(3),
        "VEC4"   => Ok(4),
        "MAT2"   => Err(AccessorMatrix(2)),
        "MAT3"   => Err(AccessorMatrix(3)),
        "MAT4"   => Err(AccessorMatrix(4)),
        _        => Err(AccessorUnknown(ty.to_string())),
    }
}

#[deriving(Clone, PartialEq, Show)]
pub enum AccessorTypeError {
    AccessorType(GLenum),
    AccessorRange(f32, f32),
}

fn parse_accessor_type(ty: GLenum, range: f32) -> Result<gfx::attrib::Type, AccessorTypeError> {
    use gfx::attrib as a;
    let sub = if range <= 2.0 {a::IntNormalized} else {a::IntAsFloat};
    Ok(match ty {
        gfx_gl::BYTE => a::Int(sub, a::U8, a::Signed),
        gfx_gl::UNSIGNED_BYTE => a::Int(sub, a::U8, a::Unsigned),
        gfx_gl::SHORT => a::Int(sub, a::U16, a::Signed),
        gfx_gl::UNSIGNED_SHORT => a::Int(sub, a::U32, a::Unsigned),
        gfx_gl::INT => a::Int(sub, a::U16, a::Signed),
        gfx_gl::UNSIGNED_INT => a::Int(sub, a::U32, a::Unsigned),
        gfx_gl::FLOAT => a::Float(a::FloatDefault, a::F32),
        _ => return Err(AccessorType(ty)),
    })
}

#[deriving(Clone, PartialEq, Show)]
pub struct ShaderError(GLenum);

fn parse_shader_type(ty: GLenum) -> Result<gfx::shade::Stage, ShaderError> {
    match ty {
        gfx_gl::VERTEX_SHADER => Ok(gfx::shade::Vertex),
        gfx_gl::GEOMETRY_SHADER => Ok(gfx::shade::Geometry),
        gfx_gl::FRAGMENT_SHADER => Ok(gfx::shade::Fragment),
        _ => Err(ShaderError(ty)),
    }
}

fn parse_state(s: types::States) -> gfx::DrawState {
    let mut d = gfx::DrawState::new();
    for gl in s.enable.iter() {
        match *gl {
            gfx_gl::CULL_FACE => {
                let cull = match s.functions.cull_face {
                    (gfx_gl::FRONT, ) => gfx::state::CullFront,
                    (gfx_gl::BACK, ) => gfx::state::CullBack,
                    _ => {
                        error!("Unknown cull mode: {}", s.functions.cull_face);
                        gfx::state::CullNothing
                    },
                };
                d.primitive.method = gfx::state::Fill(cull);
            },
            gfx_gl::POLYGON_OFFSET_FILL => {
                let (f, u) = s.functions.polygon_offset;
                d.primitive.offset = gfx::state::Offset(f, u);
            },
            gfx_gl::SAMPLE_ALPHA_TO_COVERAGE => {
                //d.multisample.alpha_to_coverage = ;
            },
            gfx_gl::DEPTH_TEST => {
                //d.depth = Some();
            },
            gfx_gl::BLEND => {
                //d.blend = Some();
            },
            gfx_gl::SCISSOR_TEST => {
                let (x, y, w, h) = s.functions.scissor;
                d.scissor = Some(gfx::Rect {
                    x: x, y: y, w: w, h: h,
                });
            },
            _ => error!("Unknown GL state: {}", *gl),
        }
    }
    d
}

pub enum LoadError {
    ErrorString,
    ErrorJson,
}

pub struct SubMesh {
    pub mesh: gfx::Mesh,
    pub slice: gfx::Slice,
    pub material: String,
}

pub struct Package {
    pub buffers:    HashMap<String, gfx::RawBufferHandle>,
    pub attributes: HashMap<String, (gfx::Attribute, gfx::VertexCount)>,
    pub models:     HashMap<String, Vec<SubMesh>>,
    pub shaders:    HashMap<String, gfx::ShaderHandle>,
    pub programs:   HashMap<String, gfx::ProgramHandle>,
}

impl Package {
    pub fn load<C: gfx::CommandBuffer, D: gfx::Device<C>>(input: &str, device: &mut D)
            -> Result<Package, LoadError> {
        let json = match json::from_str(input) {
            Ok(j) => j,
            Err(_e) => return Err(ErrorString),
        };
        let buffers = load_map(&json, "buffers", |b: types::Buffer| {
            let data = File::open(&Path::new(b.uri)).read_to_end().unwrap();
            debug_assert_eq!(data.len(), b.byte_length);
            device.create_buffer_static(&data).raw()
        });
        let attributes = load_map(&json, "accessors", |a: types::Accessor| {
            let range = a.max.val0() - a.min.val0();
            let el_count = parse_accessor_count(a.ty.as_slice()).unwrap();
            let el_type  = parse_accessor_type(a.component_type, range).unwrap();
            (gfx::Attribute {
                name: a.name,
                buffer: *buffers.find(&a.buffer_view).unwrap(),
                format: gfx::attrib::Format {
                    elem_count: el_count,
                    elem_type: el_type,
                    offset: a.byte_offset,
                    stride: a.byte_stride,
                    instance_rate: 0,
                },
            }, a.count)
        });
        let models = load_map(&json, "meshes", |m: types::Mesh| {
            m.primitives.iter().map(|prim| SubMesh {
                mesh: gfx::Mesh {
                    num_vertices: prim.attributes.values().fold(0xFFFFFFFF, |min, id| {
                        let &(_, count) = attributes.find(id).unwrap();
                        cmp::min(min, count)
                    }),
                    attributes: prim.attributes.iter().map(|(name, id)| {
                        let (mut at, _) = attributes.find(id).unwrap().clone();
                        at.name = name.clone();
                        at
                    }).collect(),
                },
                slice: attrib_to_slice(attributes.find(&prim.indices)
                    .unwrap().ref0()).unwrap(),
                material: prim.material.clone(),
            }).collect()
        });
        let shaders = load_map(&json, "shaders", |s: types::Shader| {
            let stage = parse_shader_type(s.ty).unwrap();
            let data = File::open(&Path::new(s.uri)).read_to_end().unwrap();
            let source = gfx::ShaderSource {
                glsl_120: None,
                glsl_150: Some(gfx::OwnedBytes(data)),
            };
            device.create_shader(stage, source).unwrap()
        });
        let programs = load_map(&json, "programs", |p: types::Program| {
            let vs = shaders.find(&p.vertex_shader).unwrap().clone();
            assert_eq!(*vs.get_info(), gfx::shade::Vertex);
            let fs = shaders.find(&p.fragment_shader).unwrap().clone();
            assert_eq!(*fs.get_info(), gfx::shade::Fragment);
            device.create_program([vs, fs].as_slice()).unwrap()
        });
        Ok(Package {
            buffers: buffers,
            attributes: attributes,
            models: models,
            shaders: shaders,
            programs: programs,
        })
    }
}
