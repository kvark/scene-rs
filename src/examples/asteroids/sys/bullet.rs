use cgmath::{Angle, Rad, Point, Vector};
use scenegraph::ces;
use w = world;

pub enum Event {
    EvShoot(bool),
}

pub struct System {
    input: Receiver<Event>,
    shoot: bool,
    ship_space_id: ces::Id<w::Spatial>,
    ship_inertia_id: ces::Id<w::Inertial>,
    draw: w::Drawable,
    cool_time: f32,
    pool: Vec<w::Entity>,
}

impl System {
    pub fn new(chan: Receiver<Event>, space_id: ces::Id<w::Spatial>,
               inertia_id: ces::Id<w::Inertial>, draw: w::Drawable) -> System {
        System {
            input: chan,
            shoot: false,
            ship_space_id: space_id,
            ship_inertia_id: inertia_id,
            draw: draw,
            cool_time: 1.0,
            pool: Vec::new(),
        }
    }

    fn check_input(&mut self) {
        loop {
            match self.input.try_recv() {
                Ok(EvShoot(value)) => self.shoot = value,
                Err(_) => return,
            }
        }
    }
}

impl w::System for System {
    fn process(&mut self, &(time, _): w::Params, data: &mut w::Components, entities: &mut Vec<w::Entity>) {
        self.check_input();
        self.cool_time = if self.cool_time > time {self.cool_time - time} else {0.0};
        if self.shoot && self.cool_time <= 0.0 {
            self.cool_time = 0.2;
            let velocity = 5.0f32;
            let bullet = w::Bullet {
                life_time: Some(1.0f32),
            };
            let (space, inertia) = {
                let e_space = data.space.get(self.ship_space_id);
                let e_inertia = data.inertia.get(self.ship_inertia_id);
                (w::Spatial {
                    pos: e_space.pos,
                    orient: Rad{ s: 0.0 },
                    scale: 0.1,
                }, w::Inertial {
                    velocity: e_inertia.velocity + e_space.get_direction().mul_s(velocity),
                    angular_velocity: Rad{ s: 0.0 },
                })
            };
            let ent = match self.pool.pop() {
                Some(ent) => {
                    *data.bullet.get_mut(ent.bullet.unwrap()) = bullet;
                    *data.space.get_mut(ent.space.unwrap()) = space;
                    *data.inertia.get_mut(ent.inertia.unwrap()) = inertia;
                    ent
                },
                None => {
                    data.add()
                        .space(space)
                        .inertia(inertia)
                        .draw(self.draw.clone())
                        .bullet(bullet)
                        .entity
                },
            };
            entities.push(ent);
        }
        let (new_entities, reserve) = entities.partitioned(|ent| {
            match ent.bullet {
                Some(b_id) => {
                    let bullet = data.bullet.get_mut(b_id);
                    match bullet.life_time {
                        Some(ref mut t) if *t>time => {
                            *t -= time;
                            true
                        },
                        Some(_) => {
                            bullet.life_time = None;
                            false
                        },
                        None => true,
                    }
                },
                None => true,
            }
        });
        *entities = new_entities;
        self.pool.push_all_move(reserve);
    }
}
