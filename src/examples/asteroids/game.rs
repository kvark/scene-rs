extern crate time;

use cgmath::angle::Rad;
use cgmath::point::{Point2};
use cgmath::vector::{Vector2};
use glinit;
use gfx;
use scenegraph::Array;
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
	control: w::ControlSystem,
	bullet: w::BulletSystem,
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
					p = (p * transform.w + transform.xy) * screen_scale.xy;
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
				transform: [0.0, 0.0, 0.0, 1.0],
				screen_scale: [0.1, 0.1, 0.0, 0.0],
				color: [1.0, ..4],
			}
		).unwrap();
		// populate entities
		let mut entities = Array::new();
		let mut hub = w::DataHub::new();
		let mut draw_system = w::DrawSystem::new(frame, program);
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
					scale: 1.0,
				})
				.inertia(w::Inertial {
					velocity: Vector2::zero(),
					angular_velocity: Rad{ s:0.0 },
				})
				.control(w::Control {
					thrust_scale: 4.0,
					rotate_scale: -90.0,
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
			control: w::ControlSystem {
				thrust: 0.0,
				rotate: 0.0,
			},
			bullet: w::BulletSystem::new(ship_id),
			last_time: time::precise_time_ns(),
		}
	}

	pub fn on_event(&mut self, event: glinit::Event) {
		match event {
			glinit::Pressed(glinit::A) => self.control.thrust = 1.0,
			glinit::Released(glinit::A) => self.control.thrust = 0.0,
			glinit::Pressed(glinit::Left) => self.control.rotate = -1.0,
			glinit::Pressed(glinit::Right) => self.control.rotate = 1.0,
			glinit::Released(glinit::Left) | glinit::Released(glinit::Right) =>
				self.control.rotate = 0.0,
			glinit::Pressed(glinit::S) => self.bullet.shoot = true,
			glinit::Released(glinit::S) => self.bullet.shoot = false,
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

		self.control.process(delta, &mut self.hub, self.entities.iter());
		self.inertia.process(delta, &mut self.hub, self.entities.iter());
		self.bullet.process(delta, &mut self.hub, &mut self.entities);
		self.draw.process(renderer, &mut self.hub, self.entities.iter());
	}
}
