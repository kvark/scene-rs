extern crate time;

use cgmath::angle::{Angle, Rad};
use cgmath::rotation::{Basis2, Rotation, Rotation2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vector, Vector2};
use glinit;
use gfx;
use scenegraph::{Array, Id};
use w = world;

#[vertex_format]
struct Vertex {
	pos: [f32, ..2],
}

impl Vertex {
	fn new(x: f32, y: f32) -> Vertex {
		Vertex {
			pos: [x, y]
		}
	}
}

pub struct Game {
	entities: Array<w::Entity>,
	hub: w::DataHub,
	draw: w::DrawSystem,
	inertia: w::InertiaSystem,
	ship_id: Id<w::Entity>,
	last_time: u64,
}

impl Game {
	pub fn new(frame: gfx::Frame, renderer: &mut gfx::Renderer) -> Game {
		// create draw system
		let prog_handle = renderer.create_program(
			shaders! {
			GLSL_120: b"
				#version 120
				attribute vec2 pos;
				uniform vec4 transform, screen_scale;
				void main() {
					vec2 sc = vec2(sin(transform.z), cos(transform.z));
					vec2 p = vec2(pos.x*sc.y - pos.y*sc.x, pos.x*sc.x + pos.y*sc.y);
					p = (p + transform.xy) * screen_scale.xy;
					gl_Position = vec4(p, 0.0, 1.0);
				}
			"},
			shaders! {
			GLSL_120: b"
				#version 120
				uniform vec4 color;
				void main() {
					gl_FragColor = color;
				}
			"}
		);
		let program = renderer.connect_program(
			prog_handle,
			w::ShaderParam {
				transform: [0.0, 0.0, 0.0, 0.0],
				screen_scale: [0.1, 0.1, 0.0, 0.0],
				color: [1.0, ..4],
			}
		).unwrap();
		// populate entities
		let mut entities = Array::new();
		let mut hub = w::DataHub::new();
		let mut draw_system = w::DrawSystem::new(frame);
		let ship = {
			let mesh = renderer.create_mesh(vec![
				Vertex::new(-0.3, -0.5),
				Vertex::new(0.3, -0.5),
				Vertex::new(0.0, 0.5),
			]);
			let slice = mesh.get_slice();
			let mut state = gfx::DrawState::new();
			state.primitive.method = gfx::state::Fill(gfx::state::CullNothing);
			let mesh_id = draw_system.meshes.add(mesh);
			let state_id = draw_system.states.add(state);
			hub.add()
				.draw(w::Drawable {
					program: program,
					mesh_id: mesh_id,
					state_id: state_id,
					slice: slice,
				})
				.space(w::Spatial {
					pos: Point2::new(0.0, 0.0),
					orient: Rad{ s: 0.0 },
				})
				.inertia(w::Inertial {
					velocity: Vector2::zero(),
					angular_velocity: Rad{ s:0.0 },
				})
				.entity
		};
		let ship_id = entities.add(ship);
		// done
		Game {
			entities: entities,
			hub: hub,
			draw: draw_system,
			inertia: w::InertiaSystem,
			ship_id: ship_id,
			last_time: time::precise_time_ns(),
		}
	}

	fn ship_impulse(&mut self, value: f32) {
		let speed_scale = 0.2f32;
		let ent = self.entities.get(self.ship_id);
		match (ent.space, ent.inertia) {
			(Some(s_id), Some(i_id)) => {
				let s = self.hub.space.get(s_id);
				let i = self.hub.inertia.get_mut(i_id);
				let rot: Basis2<f32> = Rotation2::from_angle(s.orient);
				let dir = rot.rotate_vector(&Vector2::unit_y());
				i.velocity.add_self_v(&dir.mul_s(speed_scale * value));
			},
			(_, _) => (),
		}
	}

	fn ship_orient(&mut self, dir: f32) {
		let orient_scale = -1.5f32;
		self.entities.get(self.ship_id).inertia.map(|i_id| {
			let al = Rad{ s: orient_scale * dir };
			self.hub.inertia.get_mut(i_id).angular_velocity = al;
		});
	}

	pub fn on_event(&mut self, event: glinit::Event) {
		match event {
			glinit::Pressed(glinit::A) => self.ship_impulse(1.0),
			glinit::Pressed(glinit::Left) => self.ship_orient(-1.0),
			glinit::Pressed(glinit::Right) => self.ship_orient(1.0),
			glinit::Released(glinit::Left) | glinit::Released(glinit::Right) =>
				self.ship_orient(0.0),
			_ => (),
		}
	}

	pub fn render(&mut self, renderer: &mut gfx::Renderer) {
		for err in renderer.errors() {
			println!("Device error: {}", err);
		}

		let new_time = time::precise_time_ns();
		let delta = (new_time - self.last_time) as f32 / 1e9;
		self.last_time = new_time;

		self.inertia.process(delta, &mut self.hub, self.entities.iter());
		self.draw.process(renderer, &mut self.hub, self.entities.iter());
	}
}
