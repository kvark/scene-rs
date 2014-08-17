use gl_init;
use glfw;
use sys;

pub type ReceiverHub = (
    Receiver<sys::control::Event>,
    Receiver<sys::bullet::Event>
);

pub struct SenderHub {
    control: Sender<sys::control::Event>,
    bullet: Sender<sys::bullet::Event>,
}

impl SenderHub {
    pub fn new() -> (SenderHub, ReceiverHub) {
        let (sc, rc) = channel();
        let (sb, rb) = channel();
        (SenderHub {
            control: sc,
            bullet: sb,
        }, (rc, rb))
    }

    pub fn process_gl_init(&self, event: gl_init::Event) {
        use sys::control::{EvThrust, EvTurn};
        use sys::bullet::{EvShoot};
        match event {
            gl_init::KeyboardInput(state, _, Some(gl_init::A), _) =>
                self.control.send(EvThrust(match state {
                    gl_init::Pressed => 1.0,
                    gl_init::Released => 0.0,
                })),
            gl_init::KeyboardInput(gl_init::Pressed, _, Some(gl_init::Left), _) =>
                self.control.send(EvTurn(-1.0)),
            gl_init::KeyboardInput(gl_init::Pressed, _, Some(gl_init::Right), _) =>
                self.control.send(EvTurn(1.0)),
            gl_init::KeyboardInput(gl_init::Released, _, Some(k), _)
                if k == gl_init::Left || k == gl_init::Right =>
                self.control.send(EvTurn(0.0)),
            gl_init::KeyboardInput(state, _, Some(gl_init::S), _) =>
                self.bullet.send(EvShoot(match state {
                    gl_init::Pressed => true,
                    gl_init::Released => false,
                })),
            _ => (),
        }
    }

    pub fn process_glfw(&self, event: glfw::WindowEvent) {
        use sys::control::{EvThrust, EvTurn};
        use sys::bullet::{EvShoot};
        match event {
            glfw::KeyEvent(glfw::KeyA, _, state, _) =>
                self.control.send(EvThrust(match state {
                    glfw::Press | glfw::Repeat => 1.0,
                    glfw::Release => 0.0,
                })),
            glfw::KeyEvent(glfw::KeyLeft, _, glfw::Press, _) =>
                self.control.send(EvTurn(-1.0)),
            glfw::KeyEvent(glfw::KeyRight, _, glfw::Press, _) =>
                self.control.send(EvTurn(1.0)),
            glfw::KeyEvent(k, _, glfw::Release, _)
                if k == glfw::KeyLeft || k == glfw::KeyRight =>
                self.control.send(EvTurn(0.0)),
            glfw::KeyEvent(glfw::KeyS, _, state, _) =>
                self.bullet.send(EvShoot(match state {
                    glfw::Press | glfw::Repeat => true,
                    glfw::Release => false,
                })),
            _ => (),
        }
    }
}
