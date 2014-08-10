#[phase(plugin)]
extern crate gfx_macros;

use cgmath::angle::Rad;
use cgmath::rotation::{Basis2, Rotation, Rotation2};
use cgmath::point::Point2;
use cgmath::vector::Vector2;
use gfx;
use es = scenegraph;


#[shader_param(Program)]
pub struct ShaderParam {
	//TODO: hide these
	pub transform: [f32, ..4],
	pub screen_scale: [f32, ..4],
}

/// --- Components ---

pub struct Drawable {
	pub program: Program,
	pub mesh_id: es::Id<gfx::Mesh>,
	pub state_id: es::Id<gfx::DrawState>,
	pub slice: gfx::Slice,
}

impl Clone for Drawable {
	fn clone(&self) -> Drawable {
		*self
	}
}

#[deriving(Clone)]
pub struct Spatial {
	pub pos: Point2<f32>,
	pub orient: Rad<f32>,
	pub scale: f32,
}

impl Spatial {
	pub fn get_direction(&self) -> Vector2<f32> {
		let rot: Basis2<f32> = Rotation2::from_angle(self.orient);
		rot.rotate_vector(&Vector2::unit_y())
	}
}

#[deriving(Clone)]
pub struct Inertial {
	pub velocity: Vector2<f32>,
	pub angular_velocity: Rad<f32>,
}

#[deriving(Clone)]
pub struct Control {
	pub thrust_scale: f32,
	pub rotate_scale: f32,
}

#[deriving(Clone)]
pub struct Bullet {
	pub life_time: Option<f32>,
}


entity! { es
	draw: Drawable,
	space: Spatial,
	inertia: Inertial,
	control: Control,
	bullet: Bullet,
}
