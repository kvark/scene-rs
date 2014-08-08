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
        pub struct Adder<'e, 'd> {
            entity: &'e mut Entity,
            hub: &'d mut DataHub,
        }
        impl<'e, 'd> Adder<'e, 'd> {
            $(
            pub fn $name(self, value: $component) -> Adder<'e, 'd> {
                debug_assert!(self.entity.$name.is_none());
                let id = self.hub.$name.add(value);
                self.entity.$name = Some(id);
                self
            }
            )*
        }
        /// Component get() wrapper
        pub struct Getter<'e, 'd> {
            entity: &'e Entity,
            hub: &'d DataHub,
        }
        impl<'e, 'd> Getter<'e, 'd> {
            $(
            pub fn $name(&'d self) -> Option<&'d $component> {
                self.entity.$name.map(|id| self.hub.$name.get(id))
            }
            )*
        }
        /// Component change() wrapper
        pub struct Changer<'e, 'd> {
            entity: &'e Entity,
            hub: &'d mut DataHub,
        }
        impl<'e, 'd> Changer<'e, 'd> {
            $(
            pub fn $name(&'d mut self) -> Option<&'d mut $component> {
                //Rust issue #16339
                //self.entity.$name.map(|id| self.hub.$name.change(id))
                match self.entity.$name {
                    Some(id) => Some(self.hub.$name.change(id)),
                    None => None,
                }
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
            pub fn add<'e, 'd>(&'d mut self, e: &'e mut Entity) -> Adder<'e, 'd> {
                Adder {entity: e, hub: self,}
            }
            pub fn get<'e, 'd>(&'d self, e: &'e Entity) -> Getter<'e, 'd> {
                Getter {entity: e, hub: self,}
            }
            pub fn change<'e, 'd>(&'d mut self, e: &'e Entity) -> Changer<'e, 'd> {
                Changer {entity: e, hub: self,}
            }
        }
    }
}
