#[macro_export]
macro_rules! entity {
    ($space:ident $($name:ident : $component:ty,)*) => {
        /// A collection of pointers to components
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

        /// DataHub implementation
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
    }
}
