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

mod parse;
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

pub enum LoadError {
    ErrorString,
    ErrorJson,
}

pub struct Pass {
    pub name: String,
    pub program: gfx::ProgramHandle,
    pub draw_state: gfx::DrawState,
}

pub struct Technique {
    pub name: String,
    pub passes: Vec<Pass>,
    pub default_pass: uint,
    pub parameters: gfx::shade::ParamDictionary,
}

pub struct Id<T>(String);

impl<T> Id<T> {
    pub fn as_ref(&self) -> &String {
        let Id(ref s) = *self;
        s
    }
}

pub struct Material {
    pub name: String,
    pub technique: Id<Technique>,
    pub parameters: gfx::shade::ParamDictionary,
}

pub struct SubMesh {
    pub mesh: gfx::Mesh,
    pub slice: gfx::Slice,
    pub material: Id<Material>,
}

pub struct Package {
    pub techniques: HashMap<String, Technique>,
    pub materials: HashMap<String, Material>,
    pub models: HashMap<String, Vec<SubMesh>>,
}

pub type Batch = gfx::batch::RefBatch<gfx::shade::ParamDictionaryLink, gfx::shade::ParamDictionary>;

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
            device.create_buffer_static(data.as_slice()).raw()
        });
        let attributes = load_map(&json, "accessors", |a: types::Accessor| {
            let range = a.max.val0() - a.min.val0();
            let el_count = parse::parse_accessor_count(a.ty.as_slice()).unwrap();
            let el_type  = parse::parse_accessor_type(a.component_type, range).unwrap();
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
        let shaders = load_map(&json, "shaders", |s: types::Shader| {
            let stage = parse::parse_shader_type(s.ty).unwrap();
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
        let techniques = load_map(&json, "techniques", |t: types::Technique| {
            let passes: Vec<Pass> = t.passes.iter().map(|(s, p)| {
                Pass {
                    name: s.clone(),
                    program: programs.find(&p.instance_program.program)
                                     .unwrap().clone(),
                    draw_state: parse::parse_state(&p.states).unwrap(),
                }
            }).collect();
            let default = passes.iter().position(|p|
                p.name.as_slice() == t.pass.as_slice()
            ).unwrap();
            Technique {
                name: t.name.clone(),
                passes: passes,
                default_pass: default,
                parameters: unimplemented!(),
            }
        });
        let materials = load_map(&json, "materials", |m: types::Material| {
            let tech = techniques.find(&m.instance_technique.technique).unwrap();
            Material {
                name: m.name.clone(),
                technique: Id(m.instance_technique.technique.clone()),
                parameters: unimplemented!(),
            }
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
                material: Id(prim.material.clone()),
            }).collect()
        });
        Ok(Package {
            techniques: techniques,
            materials: materials,
            models: models,
        })
    }

    pub fn to_batches(&self, context: &mut gfx::batch::Context) ->
        (Vec<Batch>, Vec<gfx::batch::BatchError>) {
        let mut batches = Vec::new();
        let mut errors = Vec::new();
        for model_vec in self.models.values() {
            for model in model_vec.iter() {
                let material = self.materials.find(model.material.as_ref()).unwrap();
                let technique = self.techniques.find(material.technique.as_ref()).unwrap();
                let pass = &technique.passes[technique.default_pass];
                match context.batch(&model.mesh, model.slice, &pass.program, &pass.draw_state) {
                    Ok(b) => batches.push(b),
                    Err(e) => errors.push(e),
                }
            }
        }
        (batches, errors)
    }
}
