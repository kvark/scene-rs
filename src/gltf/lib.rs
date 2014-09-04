//! GLTF package loader for gfx-rs

#![crate_name = "gltf"]

extern crate serialize;
extern crate cgmath;
extern crate gfx;

use serialize::{json, Decoder, Decodable};
use std::cmp;
use std::collections::HashMap;
use std::from_str::FromStr;
use std::io::File;

mod types;

fn load_map<T: Decodable<json::Decoder, json::DecoderError>, R>(json: &json::Json,
            var: &str, fun: |T| -> R) -> HashMap<String, R> {
    match json.find(&var.to_string()) {
        Some(&json::Object(ref map)) => map.iter().map(|(id, sub)| {
            let b: T = Decodable::decode(&mut json::Decoder::new(sub.clone())).unwrap();
            (id.clone(), fun(b))
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
    pub models: HashMap<String, (String, Vec<SubMesh>)>,
}

impl Package {
    fn load<C: gfx::CommandBuffer, D: gfx::Device<C>>(input: &str, device: &mut D)
            -> Result<Package, LoadError> {
        let json: json::Json = match FromStr::from_str(input) {
            Some(j) => j,
            None => return Err(ErrorString),
        };
        let buffers = load_map(&json, "buffers", |b: types::Buffer| {
            let data = File::open(&Path::new(b.uri)).read_to_end().unwrap();
            debug_assert_eq!(data.len(), b.byteLength);
            device.create_buffer_static(&data).raw()
        });
        let attributes = load_map(&json, "accessors", |a: types::Accessor| {
            (gfx::Attribute {
                name: a.name,
                buffer: *buffers.find(&a.bufferView).unwrap(),
                format: gfx::attrib::Format {
                    elem_count: 1, //TODO
                    elem_type: gfx::attrib::Special, //TODO
                    offset: a.byteOffset as gfx::attrib::Offset,
                    stride: a.byteStride as gfx::attrib::Stride,
                    instance_rate: 0,
                },
            }, a.count)
        });
        let models = load_map(&json, "meshes", |m: types::Mesh| {
            //(m.name.clone(), gfx::Mesh::new(10), gfx::VertexSlice(gfx::Point, 0, 0))
            let sub_meshes = m.primitives.iter().map(|prim| SubMesh {
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
            }).collect();
            (m.name.clone(), sub_meshes)
        });
        Ok(Package {
            buffers: buffers,
            attributes: attributes,
            models: models,
        })
    }
}
