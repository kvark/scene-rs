extern crate time;

use cgmath::{Rad, Point2, Vector2};
use gfx;
use gfx::DeviceHelper;
use event;
use sys;
use world;

static SCREEN_EXTENTS: [f32, ..2] = [10.0, 10.0];

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

pub struct Game {
    world: world::World,
    last_time: u64,
}

impl Game {
    fn create_program<D: gfx::Device>(device: &mut D) -> world::Program {
        device.link_program(
            shaders! {
            GLSL_150: b"
                #version 150 core
                in vec2 pos;
                in vec4 color;
                uniform vec4 transform, screen_scale;
                out vec4 v_color;
                void main() {
                    v_color = color;
                    vec2 sc = vec2(sin(transform.z), cos(transform.z));
                    vec2 p = vec2(pos.x*sc.y - pos.y*sc.x, pos.x*sc.x + pos.y*sc.y);
                    p = (p * transform.w + transform.xy) * screen_scale.xy;
                    gl_Position = vec4(p, 0.0, 1.0);
                }
            "},
            shaders! {
            GLSL_150: b"
                #version 150 core
                in vec4 v_color;
                void main() {
                    gl_FragColor = v_color;
                }
            "}
        ).unwrap()
    }

    fn create_ship<D: gfx::Device>(device: &mut D, data: &mut world::Components,
                   draw: &mut sys::draw::System, program: world::Program)
                   -> world::Entity {
        let mesh = device.create_mesh(vec![
            Vertex::new(-0.3, -0.5, 0x20C02000),
            Vertex::new(0.3, -0.5,  0x20C02000),
            Vertex::new(0.0, 0.5,   0xC0404000),
        ]);
        let slice = mesh.get_slice();
        let mut state = gfx::DrawState::new();
        state.primitive.method = gfx::state::Fill(gfx::state::CullNothing);
        data.add()
            .draw(world::Drawable {
                program: program,
                mesh_id: draw.meshes.add(mesh),
                state_id: draw.states.add(state),
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
                thrust_speed: 4.0,
                turn_speed: -90.0,
            })
            .collision(world::Collision {
                radius: 0.2,
                health: 3,
                damage: 2,
            })
            .entity
    }

    pub fn new<D: gfx::Device>(frame: gfx::Frame,
               (ev_control, ev_bullet): event::ReceiverHub, device: &mut D) -> Game {
        let mut w = world::World::new();
        // prepare systems
        let program = Game::create_program(device);
        let mut draw_system = sys::draw::System::new(SCREEN_EXTENTS, frame);
        let bullet_draw = {
            let mut mesh = device.create_mesh(vec![
                Vertex::new(0.0, 0.0, 0xFF808000),
            ]);
            mesh.prim_type = gfx::Point;
            let slice = mesh.get_slice();
            let mut state = gfx::DrawState::new();
            state.primitive.method = gfx::state::Point;
            world::Drawable {
                program: program.clone(),
                mesh_id: draw_system.meshes.add(mesh),
                state_id: draw_system.states.add(state),
                slice: slice,
            }
        };
        let aster_draw = {
            let mut mesh = device.create_mesh(vec![
                Vertex::new(-0.5, -0.5, 0xFFFFFF00),
                Vertex::new(0.5, -0.5,  0xFFFFFF00),
                Vertex::new(-0.5, 0.5,  0xFFFFFF00),
                Vertex::new(0.5, 0.5,   0xFFFFFF00),
            ]);
            mesh.prim_type = gfx::TriangleStrip;
            let slice = mesh.get_slice();
            let mut state = gfx::DrawState::new();
            state.primitive.method = gfx::state::Fill(gfx::state::CullNothing);
            world::Drawable {
                program: program.clone(),
                mesh_id: draw_system.meshes.add(mesh),
                state_id: draw_system.states.add(state),
                slice: slice,
            }
        };
        let ship = Game::create_ship(device, &mut w.data, &mut draw_system, program);
        let (space_id, inertia_id) = (ship.space.unwrap(), ship.inertia.unwrap());
        // populate world and return
        w.entities.push(ship);
        w.systems.push_all_move(vec![
            box draw_system as Box<world::System + Send>,
            box sys::inertia::System,
            box sys::control::System::new(ev_control),
            box sys::bullet::System::new(ev_bullet,
                space_id, inertia_id, bullet_draw),
            box sys::aster::System::new(SCREEN_EXTENTS, aster_draw),
            box sys::physics::System::new(),
        ]);
        Game {
            world: w,
            last_time: time::precise_time_ns(),
        }
    }

    pub fn render(&mut self, renderer: &mut gfx::Renderer) {
        let new_time = time::precise_time_ns();
        let delta = (new_time - self.last_time) as f32 / 1e9;
        self.last_time = new_time;
        self.world.update(&mut (delta, renderer));
    }

    pub fn is_alive(&self) -> bool {
        self.world.entities.iter().find(|e| {
            match (e.control, e.collision) {
                (Some(_), Some(o_id)) =>
                    self.world.data.collision.get(o_id).health != 0,
                _ => false,
            }
        }).is_some()
    }
}
