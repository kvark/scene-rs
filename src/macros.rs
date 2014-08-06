#[macro_export]
macro_rules! derive_system {
    ($system:ty . $field:ident [ $component:ty ]) => {
        impl System<$component> for $system {
            fn add_component(&mut self, c: $component) -> Id<$component> {
                self.$field.add_component(c)
            }
            fn get_component(&self, id: Id<$component>) -> &$component {
                self.$field.get_component(id)
            }
            fn mut_component(&mut self, id: Id<$component>) -> &mut $component {
                self.$field.mut_component(id)
            }
        }
    }
}

#[macro_export]
macro_rules! world {
    ($($name:ident : $system:ty [ $component:ty ],)*) => {
        #[deriving(Clone, PartialEq, Show)]
        pub struct Id<S>(uint);
        pub type EntityId = uint;

        pub trait System<T> {
            fn add_component(&mut self, T) -> Id<T>;
            fn get_component(&self, Id<T>) -> &T;
            fn mut_component(&mut self, Id<T>) -> &mut T;
        }

        impl<T> System<T> for Vec<T> {
            fn add_component(&mut self, t: T) -> Id<T> {
                self.push(t);
                Id(self.len() - 1)
            }
            fn get_component(&self, id: Id<T>) -> &T {
                let Id(h) = id;
                &self[h]
            }
            fn mut_component(&mut self, id: Id<T>) -> &mut T {
                let Id(h) = id;
                self.get_mut(h)
            }
        }

        /// A collection of pointers to components
        pub struct Entity<T> {
            #[allow(dead_code)]
            user_data: T,
            $(
                pub $name: Option<Id<$component>>,
            )*
        }
        /// A collection of systems
        pub struct SystemHub {
            $(
                pub $name: $system,
            )*
        }
        /// World has all the entities and systems
        pub struct World<T> {
            entities: Vec<Entity<T>>,
            pub systems: SystemHub,
        }
        /// Component add() wrapper
        #[allow(dead_code)]
        pub struct Adder<'a, T> {
            entity: &'a mut Entity<T>,
            hub: &'a mut SystemHub,
        }
        #[allow(dead_code)]
        impl<'a, T> Adder<'a, T> {
            $(
                pub fn $name(&mut self, value: $component) {
                    debug_assert!(self.entity.$name.is_none());
                    let id = self.hub.$name.add_component(value);
                    self.entity.$name = Some(id);
                }
            )*
        }
        /// Component get() wrapper
        #[allow(dead_code)]
        pub struct Getter<'a, T> {
            entity: &'a Entity<T>,
            hub: &'a SystemHub,
        }
        #[allow(dead_code)]
        impl<'a, T> Getter<'a, T> {
            pub fn user_data(&self) -> &T {
                &self.entity.user_data
            }
            $(
                pub fn $name(&self) -> &$component {
                    let id = self.entity.$name.unwrap();
                    self.hub.$name.get_component(id)
                }
            )*
        }
        /// Component change() wrapper
        #[allow(dead_code)]
        pub struct Changer<'a, T> {
            entity: &'a mut Entity<T>,
            hub: &'a mut SystemHub,
        }
        #[allow(dead_code)]
        impl <'a, T> Changer<'a, T> {
            pub fn user_data(&mut self) -> &mut T {
                &mut self.entity.user_data
            }
            $(
                pub fn $name(&mut self) -> &mut $component {
                    let id = self.entity.$name.unwrap();
                    self.hub.$name.mut_component(id)
                }
            )*
        }
        /// World implementation
        #[allow(dead_code)]
        impl<T> World<T> {
            pub fn new($($name : $system),*) -> World<T> {
                World {
                    entities: Vec::new(),
                    systems: SystemHub {
                        $($name : $name,)*
                    }
                }
            }
            pub fn extend(&mut self, data: T) -> EntityId {
                self.entities.push(Entity {
                    user_data: data,
                    $(
                        $name: None,
                    )*
                });
                self.entities.len() - 1
            }
            pub fn add<'a>(&'a mut self, eid: EntityId) -> Adder<'a, T> {
                Adder {
                    entity: self.entities.get_mut(eid),
                    hub: &mut self.systems,
                }
            }
            pub fn get<'a>(&'a self, eid: EntityId) -> Getter<'a, T> {
                Getter {
                    entity: &self.entities[eid],
                    hub: &self.systems,
                }
            }
            pub fn change<'a>(&'a mut self, eid: EntityId) -> Changer<'a, T> {
                Changer {
                    entity: self.entities.get_mut(eid),
                    hub: &mut self.systems,
                }
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    type DummyComponent = int;
    type DummySystem = Vec<DummyComponent>;

    world! {
        dummy : DummySystem[DummyComponent],
    }
}

#[test]
fn test() {
    let mut w = test::World::new(Vec::new());
    let eid = w.extend(());
    w.add(eid).dummy(4);
    println!("{}", w.get(eid).dummy());
}
