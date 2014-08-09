#![feature(phase)]
#![crate_name = "asteroids"]

extern crate native;
extern crate cgmath;
extern crate gfx;
extern crate gl_init_platform;
extern crate glinit = "gl-init-rs";
#[phase(plugin, link)]
extern crate scenegraph;

use std::comm;

mod game;
mod world;

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
    let mut game = game::Game::new(frame, &mut renderer);
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
