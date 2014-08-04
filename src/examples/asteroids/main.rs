#![feature(phase)]
#![crate_name = "asteroids"]

#[phase(plugin)]
extern crate gfx_macros;
extern crate gfx;
extern crate glinit = "gl-init-rs";
extern crate cgmath;
extern crate native;

//#[start]
//fn start(argc: int, argv: *const *const u8) -> int {
//     native::start(argc, argv, main)
//}

fn main() {
    let window = gfx::gl_init::Window::new().unwrap();
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
    let frame = gfx::Frame::new(width as u16, height as u16);
    let clear = gfx::ClearData {
        color: Some(gfx::Color([0.3, 0.3, 0.3, 1.0])),
        depth: None,
        stencil: None,
    };

    while !renderer.should_finish() {
        renderer.clear(clear, frame);
        renderer.end_frame();
        for err in renderer.errors() {
            println!("Renderer error: {}", err);
        }
    }
}
