#![feature(phase)]
#![crate_name = "asteroids"]

#[phase(plugin)]
extern crate gfx_macros;
extern crate gfx;
extern crate gl_init_platform;
extern crate glinit = "gl-init-rs";
extern crate cgmath;
extern crate native;
#[phase(plugin, link)]
extern crate scenegraph;

use cgmath::point::Point2;
use cgmath::vector::Vector2;
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
    offset_and_scale: [f32, ..4],
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
    speed: Vector2<f32>,
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

    fn render(&self, renderer: &mut gfx::Renderer, hub: &DataHub) {
        let clear_data = gfx::ClearData {
            color: Some(gfx::Color([0.1, 0.1, 0.1, 0.0])),
            depth: None,
            stencil: None,
        };
        renderer.clear(clear_data, self.frame);
        for drawable in hub.draw.iter() {
            let mesh = self.meshes.get(drawable.mesh_id);
            let state = self.states.get(drawable.state_id);
            renderer.draw(mesh, drawable.slice, &self.frame,
                &drawable.program, state).unwrap();
        }
    }
}

entity! { scenegraph
    draw: Drawable,
    space: Spatial,
}

struct Game {
    entities: Array<Entity>,
    hub: DataHub,
    draw: DrawSystem,
    ship_id: Id<Entity>,
}

impl Game {
    fn new(frame: gfx::Frame, renderer: &mut gfx::Renderer) -> Game {
        // create draw system
        let prog_handle = renderer.create_program(
            shaders! {
            GLSL_120: b"
                #version 120
                attribute vec2 pos;
                uniform vec4 offset_and_scale;
                void main() {
                    gl_Position = vec4((offset_and_scale.xy + pos) *
                        offset_and_scale.xy, 0.0, 1.0);
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
                offset_and_scale: [0.0, 0.0, 0.1, 0.1],
                color: [1.0, ..4],
            }
        ).unwrap();
        // populate entities
        let mut entities = Array::new();
        let mut hub = DataHub::new();
        let mut draw_system = DrawSystem::new(frame, program);
        let ship = {
            let mesh = renderer.create_mesh(vec![
                Vertex::new(-0.5, -0.5),
                Vertex::new(0.5, -0.5),
                Vertex::new(0.0, 0.5),
            ]);
            let slice = mesh.get_slice();
            let mut state = gfx::DrawState::new();
            state.primitive.method = gfx::state::Fill(gfx::state::CullNothing);
            //let draw = draw_system.produce(New(mesh), New(state), slice);
            let mesh_id = draw_system.meshes.add(mesh);
            let state_id = draw_system.states.add(state);
            Entity {
                draw: Some(hub.draw.add(Drawable {
                    program: program,
                    mesh_id: mesh_id,
                    state_id: state_id,
                    slice: slice,
                })),
                space: None,
            }
        };
        let ship_id = entities.add(ship);
        // done
        Game {
            entities: entities,
            hub: hub,
            draw: draw_system,
            ship_id: ship_id,
        }
    }

    fn render(&mut self, renderer: &mut gfx::Renderer) {
        self.draw.render(renderer, &self.hub);
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
