//! GLTF package loader for gfx-rs

#![crate_name = "gltf"]
#![feature(phase)]

extern crate serialize;
extern crate cgmath;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate serde;
#[phase(plugin)]
extern crate serde_macros;

use std::cmp;
use std::collections::HashMap;
use std::io::File;
use serde::{de, json};

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
pub enum AccessorError {
    AccessorMatrix(u8),
    AccessorUnknown(String),
}

fn parse_accessor_type(ty: &str) -> Result<(gfx::attrib::Count,
                       gfx::attrib::Type), AccessorError> {
    use gfx::attrib as a;
    match ty {
        "SCALAR" => Ok((1, a::Float(a::FloatDefault, a::F32))),
        "VEC2"   => Ok((2, a::Float(a::FloatDefault, a::F32))),
        "VEC3"   => Ok((3, a::Float(a::FloatDefault, a::F32))),
        "VEC4"   => Ok((4, a::Float(a::FloatDefault, a::F32))),
        "MAT2"   => Err(AccessorMatrix(2)),
        "MAT3"   => Err(AccessorMatrix(3)),
        "MAT4"   => Err(AccessorMatrix(4)),
        _        => Err(AccessorUnknown(ty.to_string())),
    }
}

pub enum LoadError {
    ErrorString,
    ErrorJson,
}

pub struct SubMesh {
    mesh: gfx::Mesh,
    slice: gfx::Slice,
    material: String,
}

pub struct Package {
    pub buffers: HashMap<String, gfx::RawBufferHandle>,
    pub attributes: HashMap<String, (gfx::Attribute, uint)>,
    pub models: HashMap<String, Vec<SubMesh>>,
    pub shaders: HashMap<String, gfx::ShaderHandle>,
}

impl Package {
    fn load<C: gfx::CommandBuffer, D: gfx::Device<C>>(input: &str, device: &mut D)
            -> Result<Package, LoadError> {
        let json = match json::from_str(input) {
            Ok(j) => j,
            Err(_e) => return Err(ErrorString),
        };
        let buffers = load_map(&json, "buffers", |b: types::Buffer| {
            let data = File::open(&Path::new(b.uri)).read_to_end().unwrap();
            debug_assert_eq!(data.len(), b.byteLength);
            device.create_buffer_static(&data).raw()
        });
        let attributes = load_map(&json, "accessors", |a: types::Accessor| {
            let (el_count, el_ty) = parse_accessor_type(a.ty.as_slice()).unwrap();
            (gfx::Attribute {
                name: a.name,
                buffer: *buffers.find(&a.bufferView).unwrap(),
                format: gfx::attrib::Format {
                    elem_count: el_count,
                    elem_type: el_ty,
                    offset: a.byteOffset as gfx::attrib::Offset,
                    stride: a.byteStride as gfx::attrib::Stride,
                    instance_rate: 0,
                },
            }, a.count)
        });
        let models = load_map(&json, "meshes", |m: types::Mesh| {
            //(m.name.clone(), gfx::Mesh::new(10), gfx::VertexSlice(gfx::Point, 0, 0))
            m.primitives.iter().map(|prim| SubMesh {
                mesh: gfx::Mesh {
                    num_vertices: prim.attributes.values().fold(-1u, |min, id| {
                        let &(_, count) = attributes.find(id).unwrap();
                        cmp::min(min, count)
                    }) as gfx::VertexCount,
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
            let kind = gfx::shade::Vertex; //TODO s.type
            let data = File::open(&Path::new(s.uri)).read_to_end().unwrap();
            let source = gfx::ShaderSource {
                glsl_120: None,
                glsl_150: Some(gfx::OwnedBytes(data)),
            };
            device.create_shader(kind, source).unwrap()
        });
        Ok(Package {
            buffers: buffers,
            attributes: attributes,
            models: models,
            shaders: shaders,
        })
    }
}
