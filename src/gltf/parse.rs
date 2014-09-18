use gfx;
use gfx_gl as gl;
use gfx_gl::types::GLenum;
use serde::json;

#[deriving(Clone, PartialEq, Show)]
pub enum AccessorCountError {
    AccessorMatrix(u8),
    AccessorUnknown(String),
}

pub fn parse_accessor_count(ty: &str) -> Result<gfx::attrib::Count, AccessorCountError> {
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

pub fn parse_accessor_type(ty: GLenum, range: f32) -> Result<gfx::attrib::Type, AccessorTypeError> {
    use gfx::attrib as a;
    let sub = if range <= 2.0 {a::IntNormalized} else {a::IntAsFloat};
    Ok(match ty {
        gl::BYTE => a::Int(sub, a::U8, a::Signed),
        gl::UNSIGNED_BYTE => a::Int(sub, a::U8, a::Unsigned),
        gl::SHORT => a::Int(sub, a::U16, a::Signed),
        gl::UNSIGNED_SHORT => a::Int(sub, a::U32, a::Unsigned),
        gl::INT => a::Int(sub, a::U16, a::Signed),
        gl::UNSIGNED_INT => a::Int(sub, a::U32, a::Unsigned),
        gl::FLOAT => a::Float(a::FloatDefault, a::F32),
        _ => return Err(AccessorType(ty)),
    })
}

#[deriving(Clone, PartialEq, Show)]
pub struct ShaderError(GLenum);

pub fn parse_shader_type(ty: GLenum) -> Result<gfx::shade::Stage, ShaderError> {
    match ty {
        gl::VERTEX_SHADER => Ok(gfx::shade::Vertex),
        gl::GEOMETRY_SHADER => Ok(gfx::shade::Geometry),
        gl::FRAGMENT_SHADER => Ok(gfx::shade::Fragment),
        _ => Err(ShaderError(ty)),
    }
}

pub fn parse_comparison(fun: GLenum) -> Result<gfx::state::Comparison, ()> {
    use gfx::state as s;
    Ok(match fun {
        gl::NEVER    => s::Never,
        gl::LESS     => s::Less,
        gl::LEQUAL   => s::LessEqual,
        gl::EQUAL    => s::Equal,
        gl::GEQUAL   => s::GreaterEqual,
        gl::GREATER  => s::Greater,
        gl::NOTEQUAL => s::NotEqual,
        gl::ALWAYS   => s::Always,
        _ => return Err(()),
    })
}

pub fn parse_blend_factor(factor: GLenum) -> Result<gfx::state::Factor, ()> {
    use gfx::state as s;
    Ok(match factor {
        gl::ZERO => s::Factor(s::Normal, s::Zero),
        gl::SRC_COLOR => s::Factor(s::Normal, s::SourceColor),
        gl::SRC_ALPHA => s::Factor(s::Normal, s::SourceAlpha),
        gl::SRC_ALPHA_SATURATE => s::Factor(s::Normal, s::SourceAlphaSaturated),
        gl::DST_COLOR => s::Factor(s::Normal, s::DestColor),
        gl::DST_ALPHA => s::Factor(s::Normal, s::DestAlpha),
        gl::CONSTANT_COLOR => s::Factor(s::Normal, s::ConstColor),
        gl::CONSTANT_ALPHA => s::Factor(s::Normal, s::ConstAlpha),
        gl::ONE => s::Factor(s::Inverse, s::Zero),
        gl::ONE_MINUS_SRC_COLOR => s::Factor(s::Inverse, s::SourceColor),
        gl::ONE_MINUS_SRC_ALPHA => s::Factor(s::Inverse, s::SourceAlpha),
        gl::ONE_MINUS_DST_COLOR => s::Factor(s::Inverse, s::DestColor),
        gl::ONE_MINUS_DST_ALPHA => s::Factor(s::Inverse, s::DestAlpha),
        gl::ONE_MINUS_CONSTANT_COLOR => s::Factor(s::Inverse, s::ConstColor),
        gl::ONE_MINUS_CONSTANT_ALPHA => s::Factor(s::Inverse, s::ConstAlpha),
        _ => return Err(()),
    })
}

#[deriving(Clone, PartialEq, Show)]
pub enum BlendChannelError {
    BlendEquation(GLenum),
    BlendSource(GLenum),
    BlendDestination(GLenum),
}

pub fn parse_blend_channel(eq: GLenum, src: GLenum, dst: GLenum)
                       -> Result<gfx::state::BlendChannel, BlendChannelError> {
    use gfx::state as s;
    Ok(s::BlendChannel {
        equation: match eq {
            gl::FUNC_ADD => s::FuncAdd,
            gl::FUNC_SUBTRACT => s::FuncSub,
            gl::FUNC_REVERSE_SUBTRACT => s::FuncRevSub,
            gl::MIN => s::FuncMin,
            gl::MAX => s::FuncMax,
            _ => return Err(BlendEquation(eq)),
        },
        source: match parse_blend_factor(src) {
            Ok(f) => f,
            Err(_) => return Err(BlendSource(src)),
        },
        destination: match parse_blend_factor(dst) {
            Ok(f) => f,
            Err(_) => return Err(BlendDestination(src)),
        },
    })
}

#[deriving(Clone, PartialEq, Show)]
pub enum StateError {
    StateUnknownFrontFace(GLenum),
}

pub fn parse_state(s: &::types::States) -> Result<gfx::DrawState, StateError> {
    let mut d = gfx::DrawState::new();
    d.primitive.front_face = match s.functions.front_face {
        (gl::CW, ) => gfx::state::Clockwise,
        (gl::CCW, ) => gfx::state::CounterClockwise,
        (face, ) => return Err(StateUnknownFrontFace(face)),
    };
    for gl in s.enable.iter() {
        match *gl {
            gl::CULL_FACE => {
                let cull = match s.functions.cull_face {
                    (gl::FRONT, ) => gfx::state::CullFront,
                    (gl::BACK, ) => gfx::state::CullBack,
                    _ => {
                        error!("Unknown cull mode: {}", s.functions.cull_face);
                        gfx::state::CullNothing
                    },
                };
                d.primitive.method = gfx::state::Fill(cull);
            },
            gl::POLYGON_OFFSET_FILL => {
                let (f, u) = s.functions.polygon_offset;
                d.primitive.offset = gfx::state::Offset(f, u);
            },
            gl::SAMPLE_ALPHA_TO_COVERAGE => {
                //TODO
                //d.multisample.alpha_to_coverage = ;
            },
            gl::STENCIL_TEST => {
                //TODO
            },
            gl::DEPTH_TEST => {
                let f = &s.functions;
                d.depth = Some(gfx::state::Depth {
                    fun: parse_comparison(f.depth_func.val0()).unwrap(),
                    write: f.depth_mask.val0(),
                });
            },
            gl::BLEND => {
                let (eq_c, eq_a) = s.functions.blend_equation_separate;
                let (src_c, dst_c, src_a, dst_a) = s.functions.blend_func_separate;
                let (r, g, b, a) = s.functions.blend_color;
                d.blend = Some(gfx::state::Blend {
                    color: parse_blend_channel(eq_c, src_c, dst_c).unwrap(),
                    alpha: parse_blend_channel(eq_a, src_a, dst_a).unwrap(),
                    value: [r, g, b, a],
                });
            },
            gl::SCISSOR_TEST => {
                let (x, y, w, h) = s.functions.scissor;
                d.scissor = Some(gfx::Rect {
                    x: x, y: y, w: w, h: h,
                });
            },
            _ => error!("Unknown GL state: {}", *gl),
        }
    }
    Ok(d)
}

pub enum Parameter {
    ParamUniform(gfx::UniformValue),
    ParamTexture(String),
}

impl Parameter {
    pub fn from_json(p: &json::Json) -> Result<Parameter, ()> {
        match *p {
            json::Integer(v) => Ok(ParamUniform(gfx::ValueI32(v as i32))),
            json::Floating(v) => Ok(ParamUniform(gfx::ValueF32(v as f32))),
            json::List(ref list) => match list.as_slice() {
                [json::Integer(v0), json::Integer(v1), json::Integer(v2), json::Integer(v3)] =>
                    Ok(ParamUniform(gfx::ValueI32Vector4([v0 as i32, v1 as i32, v2 as i32, v3 as i32]))),
                [json::Floating(v0), json::Floating(v1), json::Floating(v2), json::Floating(v3)] =>
                    Ok(ParamUniform(gfx::ValueF32Vector4([v0 as f32, v1 as f32, v2 as f32, v3 as f32]))),
                _ => Err(()),
            },
            json::String(ref s) => Ok(ParamTexture(s.clone())),
            _ => Err(()),
        }
    }
}
