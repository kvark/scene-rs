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

type Program = gfx::shade::CustomShell<_ShaderParamLink, ShaderParam>;

struct Drawable {
    program: Program,
    mesh_id: uint,
    state_id: uint,
    slice: gfx::Slice,
}

type DrawSystem = Vec<Drawable>;

mod sg {
    world! {
        draw: DrawSystem[Drawable],
    }
}

struct Game {
    program: Program,
    meshes: Vec<gfx::Mesh>,
    states: Vec<gfx::DrawState>,
}

impl Game {
    fn new(renderer: &mut gfx::Renderer) -> Game {
        let program = renderer.create_program(
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
        let ship_mesh = renderer.create_mesh(vec![
            Vertex::new(-0.5, -0.5),
            Vertex::new(0.5, -0.5),
            Vertex::new(0.0, 0.5),
        ]);
        let params = ShaderParam {
            offset_and_scale: [0.0, 0.0, 0.01, 0.01],
            color: [1.0, ..4],
        };
        Game {
            program: renderer.connect_program(program, params),
            meshes: Vec::new(),
            states: Vec::new(),
        }
    }

    fn render(renderer: &mut gfx::Renderer) {
        //empty
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
    let clear_data = gfx::ClearData {
        color: Some(gfx::Color([0.3, 0.3, 0.3, 1.0])),
        depth: None,
        stencil: None,
    };
    let mut game = Game::new(&mut renderer);
    while !renderer.should_finish() {
        renderer.clear(clear_data, frame);
        game.render(&mut renderer);
        renderer.end_frame();
        for err in renderer.errors() {
            println!("Renderer error: {}", err);
        }
    }
}
