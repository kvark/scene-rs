#![feature(phase)]
#![crate_name = "asteroids"]

extern crate native;
extern crate time;
#[phase(plugin)]
extern crate gfx_macros;
extern crate gfx;
extern crate gl_init_platform;
extern crate glinit = "gl-init-rs";
extern crate cgmath;
#[phase(plugin, link)]
extern crate scenegraph;

use std::{comm, slice};
use cgmath::angle::{Angle, Rad};
use cgmath::rotation::{Basis2, Rotation, Rotation2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vector, Vector2};
use scenegraph::{Array, Id};

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

#[shader_param(Program)]
struct ShaderParam {
    transform: [f32, ..4],
    screen_scale: [f32, ..4],
    color: [f32, ..4],
}

pub struct Drawable {
    program: Program,
    mesh_id: Id<gfx::Mesh>,
    state_id: Id<gfx::DrawState>,
    slice: gfx::Slice,
}

pub struct Spatial {
    pos: Point2<f32>,
    orient: Rad<f32>,
}

pub struct Inertial {
    velocity: Vector2<f32>,
    angular_velocity: Rad<f32>,
}

entity! { scenegraph
    draw: Drawable,
    space: Spatial,
    inertia: Inertial,
}

struct DrawSystem {
    frame: gfx::Frame,
    program: Program,
    pub meshes: Array<gfx::Mesh>,
    pub states: Array<gfx::DrawState>,
}

impl DrawSystem {
    fn new(frame: gfx::Frame, program: Program) -> DrawSystem {
        DrawSystem {
            frame: frame,
            program: program,
            meshes: Array::new(),
            states: Array::new(),
        }
    }

    fn render(&self, renderer: &mut gfx::Renderer, drawable: &Drawable) {
        let mesh = self.meshes.get(drawable.mesh_id);
        let state = self.states.get(drawable.state_id);
        renderer.draw(mesh, drawable.slice, &self.frame,
            &drawable.program, state).unwrap();
    }

    fn process<'a>(&self, renderer: &mut gfx::Renderer, hub: &mut DataHub,
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
                    drawable.program.data.transform = [s.pos.x, s.pos.y, s.orient.s, 0.0];
                });
                self.render(renderer, drawable)
            });
        }
    }
}

struct InertiaSystem;

impl InertiaSystem {
    fn process(&mut self, delta: f32, hub: &mut DataHub, mut en_iter: slice::Items<Entity>) {
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


struct Game {
    entities: Array<Entity>,
    hub: DataHub,
    draw: DrawSystem,
    inertia: InertiaSystem,
    ship_id: Id<Entity>,
    last_time: u64,
}

impl Game {
    fn new(frame: gfx::Frame, renderer: &mut gfx::Renderer) -> Game {
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
            ShaderParam {
                transform: [0.0, 0.0, 0.0, 0.0],
                screen_scale: [0.1, 0.1, 0.0, 0.0],
                color: [1.0, ..4],
            }
        ).unwrap();
        // populate entities
        let mut entities = Array::new();
        let mut hub = DataHub::new();
        let mut draw_system = DrawSystem::new(frame, program);
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
                .draw(Drawable {
                    program: program,
                    mesh_id: mesh_id,
                    state_id: state_id,
                    slice: slice,
                })
                .space(Spatial {
                    pos: Point2::new(0.0, 0.0),
                    orient: Rad{ s: 0.0 },
                })
                .inertia(Inertial {
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
            inertia: InertiaSystem,
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

    fn on_event(&mut self, event: glinit::Event) {
        match event {
            glinit::Pressed(glinit::A) => self.ship_impulse(1.0),
            glinit::Pressed(glinit::Left) => self.ship_orient(-1.0),
            glinit::Pressed(glinit::Right) => self.ship_orient(1.0),
            glinit::Released(glinit::Left) | glinit::Released(glinit::Right) =>
                self.ship_orient(0.0),
            _ => (),
        }
    }

    fn render(&mut self, renderer: &mut gfx::Renderer) {
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

fn main() {
    let builder = glinit::WindowBuilder::new()
        .with_title("Asteroids example for #scenegraph-rs".to_string())
        .with_gl_version((3,2));

    let window = gl_init_platform::Window::from_builder(builder)
        .unwrap();
    unsafe { window.make_current() };
    let (w, h) = window.get_inner_size().unwrap();
    let (ev_send, ev_recv) = channel();

    let mut device = gfx::build()
        .with_context(&window)
        .with_provider(&window)
        .with_queue_size(1)
        .spawn(proc(r) render(r, w as u16, h as u16, ev_recv))
        .unwrap();

    'main: loop {
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glinit::Pressed(glinit::Escape) => break 'main,
                glinit::Closed => break 'main,
                _ => ev_send.send(event),
            }
        }

        device.update();
    }

    device.update();
    device.close();
}

fn render(mut renderer: gfx::Renderer, width: u16, height: u16,
          ev_chan: Receiver<glinit::Event>) {
    let frame = gfx::Frame::new(width, height);
    let mut game = Game::new(frame, &mut renderer);
    while !renderer.should_finish() {
        loop {
            match ev_chan.try_recv() {
                Ok(event) => game.on_event(event),
                Err(comm::Empty) => break,
                Err(comm::Disconnected) => return,
            }
        }
        game.render(&mut renderer);
        renderer.end_frame();
        for err in renderer.errors() {
            println!("Renderer error: {}", err);
        }
    }
}
