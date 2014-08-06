#![feature(phase)]
#![crate_name = "asteroids"]

#[phase(plugin)]
extern crate gfx_macros;
extern crate gfx;
extern crate gl_init_platform;
extern crate glinit = "gl-init-rs";
extern crate cgmath;
extern crate native;
#[phase(plugin)]
extern crate scenegraph;

use cgmath::point::Point2;
use cgmath::vector::Vector2;

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
    offset_and_scale: [f32, ..4],
    color: [f32, ..4],
}

enum Resource<T> {
    New(T),
    Existing(uint),
}

struct Drawable {
    program: Program,
    mesh_id: uint,
    state_id: uint,
    slice: gfx::Slice,
}

struct Spatial {
    pos: Point2<f32>,
    speed: Vector2<f32>,
}

struct DrawSystem {
    data: Vec<Drawable>,
    frame: gfx::Frame,
    program: Program,
    meshes: Vec<gfx::Mesh>,
    states: Vec<gfx::DrawState>,
}

impl DrawSystem {
    fn new(frame: gfx::Frame, program: Program) -> DrawSystem {
        DrawSystem {
            data: Vec::new(),
            frame: frame,
            program: program,
            meshes: Vec::new(),
            states: Vec::new(),
        }
    }

    fn produce(&mut self, mesh: Resource<gfx::Mesh>,
               state: Resource<gfx::DrawState>, slice: gfx::Slice) -> Drawable {
        Drawable {
            program: self.program,
            mesh_id: match mesh {
                New(data) => {
                    self.meshes.push(data);
                    self.meshes.len() - 1
                },
                Existing(id) => id,
            },
            state_id: match state {
                New(data) => {
                    self.states.push(data);
                    self.states.len() - 1
                },
                Existing(id) => id,
            },
            slice: slice,
        }
    }

    fn render(&self, renderer: &mut gfx::Renderer) {
        let clear_data = gfx::ClearData {
            color: Some(gfx::Color([0.3, 0.3, 0.3, 1.0])),
            depth: None,
            stencil: None,
        };
        renderer.clear(clear_data, self.frame);
        for drawable in self.data.iter() {
            let mesh = &self.meshes[drawable.mesh_id];
            let state = &self.states[drawable.state_id];
            renderer.draw(mesh, drawable.slice, &self.frame,
                &drawable.program, state).unwrap();
        }
    }
}

mod sg {
    world! {
        draw: super::DrawSystem [ super::Drawable ],
        space: Vec<super::Spatial> [ super::Spatial ],
    }
    derive_system! { super::DrawSystem . data [ super::Drawable ]}
}

struct Game {
    world: sg::World<()>,
    ship: sg::EntityId,
}

impl Game {
    fn new(frame: gfx::Frame, renderer: &mut gfx::Renderer) -> Game {
        // create draw system
        let prog_handle = renderer.create_program(
            shaders! {
            GLSL_150: b"
                #version 150 core
                in vec2 pos;
                uniform vec4 offset_and_scale;
                void main() {
                    gl_Position = vec4((offset_and_scale.xy + pos) *
                        offset_and_scale.xy, 0.0, 1.0);
                }
            "},
            shaders! {
            GLSL_150: b"
                #version 150 core
                out vec4 o_Color;
                uniform vec4 color;
                void main() {
                    o_Color = color;
                }
            "}
        );
        let program = renderer.connect_program(
            prog_handle,
            ShaderParam {
                offset_and_scale: [0.0, 0.0, 0.01, 0.01],
                color: [1.0, ..4],
            }
        ).unwrap();
        // populate entities
        let mut world = sg::World::new(
            DrawSystem::new(frame, program),
            Vec::new()
        );
        let ship = {
            let mesh = renderer.create_mesh(vec![
                Vertex::new(-0.5, -0.5),
                Vertex::new(0.5, -0.5),
                Vertex::new(0.0, 0.5),
            ]);
            let slice = mesh.get_slice();
            let state = gfx::DrawState::new();
            let draw = world.systems.draw.produce(New(mesh), New(state), slice);
            let entity = world.extend(());
            world.add(entity).draw(draw);
            entity
        };
        // done
        Game {
            world: world,
            ship: ship,
        }
    }

    fn render(&mut self, renderer: &mut gfx::Renderer) {
        self.world.systems.draw.render(renderer);
    }
}

fn main() {
    let window = gl_init_platform::Window::new().unwrap();
    window.set_title("Asteroids example for #scenegraph-rs");
    unsafe { window.make_current() };

    let (w, h) = window.get_inner_size().unwrap();

    let mut device = gfx::build()
        .with_context(&window)
        .with_provider(&window)
        .with_queue_size(1)
        .spawn(proc(r) render(r, w as u16, h as u16))
        .unwrap();

    'main: loop {
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glinit::Pressed(glinit::Escape) => break 'main,
                glinit::Closed => break 'main,
                _ => {},
            }
        }

        device.update();
    }

    device.update();
    device.close();
}

fn render(mut renderer: gfx::Renderer, width: u16, height: u16) {
    let frame = gfx::Frame::new(width, height);
    let mut game = Game::new(frame, &mut renderer);
    while !renderer.should_finish() {
        game.render(&mut renderer);
        renderer.end_frame();
        for err in renderer.errors() {
            println!("Renderer error: {}", err);
        }
    }
}
