#![feature(phase)]
#![crate_name = "asteroids"]

extern crate cgmath;
extern crate gfx;
extern crate gl_init;
#[phase(plugin, link)]
extern crate scenegraph;

use gfx::{Device, DeviceHelper};

mod game;
mod world;
mod sys {
    pub mod aster;
    pub mod bullet;
    pub mod control;
    pub mod draw;
    pub mod inertia;
}

fn main() {
    let window = gl_init::WindowBuilder::new()
        .with_title("Asteroids example for #scenegraph-rs".to_string())
        .with_gl_version((3,2))
        .build().unwrap();

    unsafe { window.make_current() };
    let mut device = gfx::GlDevice::new(|s| window.get_proc_address(s));

    let (w, h) = window.get_inner_size().unwrap();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let (ev_send, ev_recv) = game::EventSender::new();
    let game = game::Game::new(frame, ev_recv, &mut device);
    let (game_send, dev_recv) = channel();
    let (dev_send, game_recv) = channel();

    let list = device.create_draw_list();
    game_send.send(list.clone_empty()); // double-buffering draw lists
    game_send.send(list);

    spawn(proc() {
        let mut game = game;
        loop {
            let mut list: gfx::DrawList = match game_recv.recv_opt() {
                Ok(l) => l,
                Err(_) => break,
            };
            list.reset();
            game.render(&mut list);
            match game_send.send_opt(list) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
    });

    'main: loop {
        let list = dev_recv.recv();
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                gl_init::KeyboardInput(_, _, Some(gl_init::Escape), _) => break 'main,
                gl_init::Closed => break 'main,
                _ => ev_send.process(event),
            }
        }
        device.submit(list.as_slice());
        dev_send.send(list);
        window.swap_buffers();
    }
}
