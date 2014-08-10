extern crate time;

use cgmath::angle::Rad;
use cgmath::point::{Point2};
use cgmath::vector::{Vector2};
use glinit;
use gfx;
use sys;
use world;

#[vertex_format]
struct Vertex {
	pos: [f32, ..2],
	#[normalized]
	color: [u8, ..4],
}

impl Vertex {
	fn new(x: f32, y: f32, col: uint) -> Vertex {
		Vertex {
			pos: [x, y],
			color: [(col>>24) as u8, (col>>16) as u8, (col>>8) as u8, col as u8],
		}
	}
}

struct SystemHub {
	draw: sys::draw::System,
	inertia: sys::inertia::System,
	control: sys::control::System,
	bullet: sys::bullet::System,
}

pub struct Game {
	entities: Vec<world::Entity>,
	data: world::DataHub,
	systems: SystemHub,
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
				attribute vec4 color;
				uniform vec4 transform, screen_scale;
				varying vec4 v_color;
				void main() {
					v_color = color;
					vec2 sc = vec2(sin(transform.z), cos(transform.z));
					vec2 p = vec2(pos.x*sc.y - pos.y*sc.x, pos.x*sc.x + pos.y*sc.y);
					p = (p * transform.w + transform.xy) * screen_scale.xy;
					gl_Position = vec4(p, 0.0, 1.0);
				}
			"},
			shaders! {
			GLSL_120: b"
				#version 120
				varying vec4 v_color;
				void main() {
					gl_FragColor = v_color;
				}
			"}
		);
		let program = renderer.connect_program(
			prog_handle,
			world::ShaderParam {
				transform: [0.0, 0.0, 0.0, 1.0],
				screen_scale: [0.1, 0.1, 0.0, 0.0],
			}
		).unwrap();
		// populate entities
		let mut entities = Vec::new();
		let mut data = world::DataHub::new();
		let mut draw_system = sys::draw::System::new(frame);
		let bullet_draw = {
			let mut mesh = renderer.create_mesh(vec![
				Vertex::new(0.0, 0.0, 0xFF404000),
			]);
			mesh.prim_type = gfx::Point;
			let slice = mesh.get_slice();
			let mut state = gfx::DrawState::new();
			state.primitive.method = gfx::state::Point;
			world::Drawable {
				program: program,
				mesh_id: draw_system.meshes.add(mesh),
				state_id: draw_system.states.add(state),
				slice: slice,
			}
		};
		let ship = {
			let mesh = renderer.create_mesh(vec![
				Vertex::new(-0.3, -0.5, 0x20C02000),
				Vertex::new(0.3, -0.5,  0x20C02000),
				Vertex::new(0.0, 0.5,   0x20C02000),
			]);
			let slice = mesh.get_slice();
			let mut state = gfx::DrawState::new();
			state.primitive.method = gfx::state::Fill(gfx::state::CullNothing);
			data.add()
				.draw(world::Drawable {
					program: program,
					mesh_id: draw_system.meshes.add(mesh),
					state_id: draw_system.states.add(state),
					slice: slice,
				})
				.space(world::Spatial {
					pos: Point2::new(0.0, 0.0),
					orient: Rad{ s: 0.0 },
					scale: 1.0,
				})
				.inertia(world::Inertial {
					velocity: Vector2::zero(),
					angular_velocity: Rad{ s:0.0 },
				})
				.control(world::Control {
					thrust_scale: 4.0,
					rotate_scale: -90.0,
				})
				.entity
		};
		let (space_id, inertia_id) = (ship.space.unwrap(), ship.inertia.unwrap());
		entities.push(ship);
		// done
		Game {
			entities: entities,
			data: data,
			systems: SystemHub {
				draw: draw_system,
				inertia: sys::inertia::System,
				control: sys::control::System::new(),
				bullet: sys::bullet::System::new(space_id, inertia_id, bullet_draw),
			},
			last_time: time::precise_time_ns(),
		}
	}

	pub fn on_event(&mut self, event: glinit::Event) {
		match event {
			glinit::Pressed(glinit::A) => self.systems.control.thrust = 1.0,
			glinit::Released(glinit::A) => self.systems.control.thrust = 0.0,
			glinit::Pressed(glinit::Left) => self.systems.control.rotate = -1.0,
			glinit::Pressed(glinit::Right) => self.systems.control.rotate = 1.0,
			glinit::Released(glinit::Left) | glinit::Released(glinit::Right) =>
				self.systems.control.rotate = 0.0,
			glinit::Pressed(glinit::S) => self.systems.bullet.shoot = true,
			glinit::Released(glinit::S) => self.systems.bullet.shoot = false,
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

		self.systems.control.process(delta, &mut self.data, self.entities.as_slice());
		self.systems.inertia.process(delta, &mut self.data, self.entities.as_slice());
		self.systems.bullet .process(delta, &mut self.data, &mut self.entities);
		self.systems.draw.process(renderer, &mut self.data, self.entities.as_slice());
	}
}
