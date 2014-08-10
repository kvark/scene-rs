#![macro_escape]

#[macro_export]
macro_rules! entity {
    ($space:ident $($name:ident : $component:ty,)*) => {
        /// A collection of pointers to components
        #[deriving(Clone)]
        pub struct Entity {
            $(
            pub $name: Option<$space::Id<$component>>,
            )*
        }

        impl Entity {
            pub fn new() -> Entity {
                Entity {
                    $(
                    $name: None,
                    )*
                }
            }
        }

        /// A collection of component arrays
        pub struct DataHub {
            $(
            pub $name: $space::Array<$component>,
            )*
        }

        /// Component add_to() wrapper
        pub struct Adder<'d> {
            pub entity: Entity,
            hub: &'d mut DataHub,
        }
        impl<'d> Adder<'d> {
            $(
            pub fn $name(mut self, value: $component) -> Adder<'d> {
                debug_assert!(self.entity.$name.is_none());
                let id = self.hub.$name.add(value);
                self.entity.$name = Some(id);
                self
            }
            )*
        }

        impl DataHub {
            pub fn new() -> DataHub {
                DataHub {
                $(
                    $name: $space::Array::new(),
                )*
                }
            }
            pub fn add<'d>(&'d mut self) -> Adder<'d> {
                Adder {entity: Entity::new(), hub: self,}
            }
        }

        /// A system responsible for some aspect (physics, rendering, etc)
        pub trait System {
            fn process(&mut self, &mut DataHub, &mut Vec<Entity>, delta: f32);
        }

        /// A top level union of entities, their data, and systems
        pub struct World {
            pub data: DataHub,
            pub entities: Vec<Entity>,
            pub systems: Vec<Box<System>>,
        }

        impl World {
            pub fn new() -> World {
                World {
                    data: DataHub::new(),
                    entities: Vec::new(),
                    systems: Vec::new(),
                }
            }
            pub fn update(&mut self, delta: f32) {
                for sys in self.systems.mut_iter() {
                    sys.process(&mut self.data, &mut self.entities, delta);
                }
            }
        }
    }
}
