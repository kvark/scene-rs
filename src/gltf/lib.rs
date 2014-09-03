//! GLTF package loader for gfx-rs

#![crate_name = "gltf"]

extern crate serialize;
extern crate cgmath;
extern crate gfx;

use serialize::{json, Decoder, Decodable};
use std::collections::HashMap;
use std::from_str::FromStr;
use std::io::File;

mod types;

pub enum LoadError {
    ErrorString,
    ErrorJson,
}

pub struct Package {
    pub buffers: HashMap<String, gfx::RawBufferHandle>,
    pub attributes: HashMap<String, gfx::Attribute>,
    pub meshes: HashMap<String, (String, gfx::Mesh, gfx::Slice)>,
}

impl Package {
    fn load<C: gfx::CommandBuffer, D: gfx::Device<C>>(input: &str, device: &mut D)
            -> Result<Package, LoadError> {
        let json: json::Json = match FromStr::from_str(input) {
            Some(j) => j,
            None => return Err(ErrorString),
        };
        let buffers = match json.find(&"buffers".to_string()) {
            Some(buffers_json) => buffers_json.as_object().unwrap().iter().map(|(id, bj)| {
                let b: types::Buffer = Decodable::decode(&mut json::Decoder::new(bj.clone())).unwrap();
                let data = File::open(&Path::new(b.uri)).read_to_end().unwrap();
                debug_assert_eq!(data.len(), b.byteLength);
                let buf = device.create_buffer_static(&data);
                (id.clone(), buf.raw())
            }).collect(),
            None => HashMap::new(),
        };
        let attributes = match json.find(&"meshes".to_string()) {
            Some(accessors_json) => accessors_json.as_object().unwrap().iter().map(|(id, aj)| {
                let a: types::Accessor = Decodable::decode(&mut json::Decoder::new(aj.clone())).unwrap();
                let attrib = gfx::Attribute {
                    name: a.name,
                    buffer: *buffers.find(&a.bufferView).unwrap(),
                    format: gfx::attrib::Format {
                        elem_count: 1, //TODO
                        elem_type: gfx::attrib::Special,
                        offset: a.byteOffset as gfx::attrib::Offset,
                        stride: a.byteStride as gfx::attrib::Stride,
                        instance_rate: 0,
                    },
                };
                (id.clone(), attrib)
            }).collect(),
            None => HashMap::new(),
        };
        let meshes = match json.find(&"meshes".to_string()) {
            Some(meshes_json) => meshes_json.as_object().unwrap().iter().map(|(id, mj)| {
                let m: types::Mesh = Decodable::decode(&mut json::Decoder::new(mj.clone())).unwrap();
                let mesh_tuple = (m.name.clone(), gfx::Mesh::new(10), gfx::VertexSlice(gfx::Point, 0, 0)); //TODO
                (id.clone(), mesh_tuple)
            }).collect(),
            None => HashMap::new(),
        };
        Ok(Package {
            buffers: buffers,
            attributes: attributes,
            meshes: meshes,
        })
    }
}
