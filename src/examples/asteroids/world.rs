#[phase(plugin)]
extern crate gfx_macros;

use std::slice;
use cgmath::angle::{Angle, Rad};
use cgmath::rotation::{Basis2, Rotation, Rotation2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vector, Vector2};
use gfx;
use es = scenegraph;

#[shader_param(Program)]
pub struct ShaderParam {
	//TODO: hide these
	pub transform: [f32, ..4],
	pub screen_scale: [f32, ..4],
	pub color: [f32, ..4],
}

/// --- Components ---

pub struct Drawable {
	pub program: Program,
	pub mesh_id: es::Id<gfx::Mesh>,
	pub state_id: es::Id<gfx::DrawState>,
	pub slice: gfx::Slice,
}

pub struct Spatial {
	pub pos: Point2<f32>,
	pub orient: Rad<f32>,
	pub scale: f32,
}

impl Spatial {
	fn get_direction(&self) -> Vector2<f32> {
		let rot: Basis2<f32> = Rotation2::from_angle(self.orient);
		rot.rotate_vector(&Vector2::unit_y())
	}
}

pub struct Inertial {
	pub velocity: Vector2<f32>,
	pub angular_velocity: Rad<f32>,
}

pub struct Control {
	pub thrust_scale: f32,
	pub rotate_scale: f32,
}

pub struct Bullet {
	pub life_time: Option<f32>,
}

/// --- Entity ---

entity! { es
	draw: Drawable,
	space: Spatial,
	inertia: Inertial,
	control: Control,
	bullet: Bullet,
}

/// --- Systems ---

pub struct DrawSystem {
	program: Program,
	frame: gfx::Frame,
	pub meshes: es::Array<gfx::Mesh>,
	pub states: es::Array<gfx::DrawState>,
}

impl DrawSystem {
	pub fn new(frame: gfx::Frame, program: Program) -> DrawSystem {
		DrawSystem {
			program: program,
			frame: frame,
			meshes: es::Array::new(),
			states: es::Array::new(),
		}
	}

	fn render(&self, renderer: &mut gfx::Renderer, drawable: &Drawable) {
		let mesh = self.meshes.get(drawable.mesh_id);
		let state = self.states.get(drawable.state_id);
		renderer.draw(mesh, drawable.slice, &self.frame,
			&drawable.program, state).unwrap();
	}

	pub fn process<'a>(&self, renderer: &mut gfx::Renderer, hub: &mut DataHub,
				mut en_iter: slice::Items<'a, Entity>) {
		let clear_data = gfx::ClearData {
			color: Some(gfx::Color([0.1, 0.1, 0.1, 0.0])),
			depth: None,
			stencil: None,
		};
		renderer.clear(clear_data, self.frame);
		for ent in en_iter {
			ent.draw.map(|d_id| {
				let drawable = hub.draw.get_mut(d_id);
				ent.space.map(|s_id| {
					let s = hub.space.get(s_id);
					drawable.program.data.transform = [s.pos.x, s.pos.y, s.orient.s, s.scale];
				});
				self.render(renderer, drawable)
			});
		}
	}
}

pub struct InertiaSystem;
impl InertiaSystem {
	pub fn process(&mut self, delta: f32, hub: &mut DataHub, mut en_iter: slice::Items<Entity>) {
		for ent in en_iter {
			ent.space.map(|s_id| {
				let s = hub.space.get_mut(s_id);
				ent.inertia.map(|i_id| {
					let i = hub.inertia.get(i_id);
					let move = i.velocity.mul_s(delta);
					s.pos.add_self_v(&move);
					s.orient.add_self_a(i.angular_velocity.mul_s(delta));
				});
			});
		}
	}
}

pub struct ControlSystem {
	pub thrust: f32,
	pub rotate: f32,
}

impl ControlSystem {
	pub fn process(&mut self, delta: f32, hub: &mut DataHub, mut en_iter: slice::Items<Entity>) {
		for ent in en_iter {
			match (ent.control, ent.inertia) {
				(Some(c_id), Some(i_id)) => {
					let c = hub.control.get(c_id);
					let i = hub.inertia.get_mut(i_id);
					let rotate = delta * c.rotate_scale * self.rotate;
					i.angular_velocity = Rad{ s: rotate };
					match ent.space {
						Some(s_id) => {
							let s = hub.space.get_mut(s_id);
							let dir = s.get_direction();
							let thrust = delta * c.thrust_scale * self.thrust;
							i.velocity.add_self_v(&dir.mul_s(thrust));
						},
						None => (),
					}
				},
				(_, _) => (),
			}
		}
	}
}

pub struct BulletSystem {
	pub shoot: bool,
	ship_id: es::Id<Entity>,
	cool_time: f32,
	pool: Vec<es::Id<Entity>>,
}

impl BulletSystem {
	pub fn new(ship_id: es::Id<Entity>) -> BulletSystem {
		BulletSystem {
			shoot: false,
			ship_id: ship_id,
			cool_time: 1.0,
			pool: Vec::new(),
		}
	}

	pub fn process(&mut self, delta: f32, hub: &mut DataHub, entities: &mut es::Array<Entity>) {
		self.cool_time = if self.cool_time > delta {self.cool_time - delta} else {0.0};
		if self.shoot {
			let velocity = 5.0f32;
			let bullet = Bullet {
				life_time: Some(5.0f32),
			};
			let (space, inertia) = {
				let e_space = hub.space.get(entities.get(self.ship_id).space.unwrap());
				let e_inertia = hub.inertia.get(entities.get(self.ship_id).inertia.unwrap());
				(Spatial {
					pos: e_space.pos,
					orient: Rad{ s: 0.0 },
					scale: 0.1,
				},Inertial {
					velocity: e_inertia.velocity + e_space.get_direction().mul_s(velocity),
					angular_velocity: Rad{ s: 0.0 },
				})
			};
			let draw = *hub.draw.get(entities.get(self.ship_id).draw.unwrap());
			match self.pool.pop() {
				Some(e_id) => {
					let ent = entities.get_mut(e_id);
					*hub.bullet.get_mut(ent.bullet.unwrap()) = bullet;
					*hub.space.get_mut(ent.space.unwrap()) = space;
					*hub.inertia.get_mut(ent.inertia.unwrap()) = inertia;
				},
				None => {
					entities.add(hub.add()
						.space(space)
						.inertia(inertia)
						.draw(draw)
						.bullet(bullet)
						.entity
					);
				},
			}
		}
		for ent in entities.mut_iter() {
			match ent.bullet {
				Some(b_id) => {
					let bullet = hub.bullet.get_mut(b_id);
					bullet.life_time = match bullet.life_time {
						Some(t) if t>delta => Some(t-delta),
						Some(t) => {
							//self.pool.push();
							None
						},
						None => None,
					}
				},
				None => (),
			}
		}
	}
}
