#![feature(phase)]
#![crate_name = "asteroids"]

extern crate cgmath;
extern crate gfx;
extern crate gl_init;
extern crate glfw;
#[phase(plugin, link)]
extern crate scenegraph;

use glfw::Context;
use gfx::{Device, DeviceHelper};

mod event;
mod game;
mod world;
mod sys {
    pub mod aster;
    pub mod bullet;
    pub mod control;
    pub mod draw;
    pub mod inertia;
}

fn game_loop(mut game: game::Game, list_recv: Receiver<gfx::DrawList>, list_end: Sender<gfx::DrawList>) {
    while game.is_alive() {
        let mut list = match list_recv.recv_opt() {
            Ok(l) => l,
            Err(_) => break,
        };
        list.reset();
        game.render(&mut list);
        match list_end.send_opt(list) {
            Ok(_) => (),
            Err(_) => break,
        }
    }
}

fn main() {
    let use_glfw = false;
    let title = "Asteroids example for #scenegraph-rs";
    let (ev_send, ev_recv) = event::SenderHub::new();
    let (game_send, dev_recv) = channel();
    let (dev_send, game_recv) = channel();

    if use_glfw {
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::ContextVersion(3, 2));
        glfw.window_hint(glfw::OpenglForwardCompat(true));
        glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));
        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);

        let (window, events) = glfw
            .create_window(640, 480, title, glfw::Windowed)
            .expect("Failed to create GLFW window.");

        window.make_current();
        window.set_key_polling(true); // so we can quit when Esc is pressed
        let mut device = gfx::GlDevice::new(|s| glfw.get_proc_address(s));

        let (w, h) = window.get_framebuffer_size();
        let frame = gfx::Frame::new(w as u16, h as u16);
        let game = game::Game::new(frame, ev_recv, &mut device);

        let list = device.create_draw_list();
        game_send.send(list.clone_empty()); // double-buffering draw lists
        game_send.send(list);

        spawn(proc() game_loop(game, game_recv, game_send));

        while !window.should_close() {
            let list = dev_recv.recv();
            glfw.poll_events();
            // quit when Esc is pressed.
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) =>
                        window.set_should_close(true),
                    _ => ev_send.process_glfw(event),
                }
            }
            device.submit(list.as_slice());
            dev_send.send(list);
            window.swap_buffers();
        }
    }else {
        let window = gl_init::WindowBuilder::new()
            .with_title(title.to_string())
            .with_gl_version((3,2))
            .build().unwrap();

        unsafe { window.make_current() };
        let mut device = gfx::GlDevice::new(|s| window.get_proc_address(s));

        let (w, h) = window.get_inner_size().unwrap();
        let frame = gfx::Frame::new(w as u16, h as u16);
        let game = game::Game::new(frame, ev_recv, &mut device);

        let list = device.create_draw_list();
        game_send.send(list.clone_empty()); // double-buffering draw lists
        game_send.send(list);

        spawn(proc() game_loop(game, game_recv, game_send));

        'main: loop {
            let list = dev_recv.recv();
            // quit when Esc is pressed.
            for event in window.poll_events() {
                match event {
                    gl_init::KeyboardInput(_, _, Some(gl_init::Escape), _) => break 'main,
                    gl_init::Closed => break 'main,
                    _ => ev_send.process_gl_init(event),
                }
            }
            device.submit(list.as_slice());
            dev_send.send(list);
            window.swap_buffers();
        }
    };
}
