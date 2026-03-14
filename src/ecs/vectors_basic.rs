use crate::ecs::*;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use paste::paste;

macro_rules! struct_with_vector {
    ($struct_name:ident, $t:ident, $($name:ident),+) => {
        #[derive(Component, Debug, Clone, Default)]
        pub struct $struct_name {
            $(
                $name: Reversible<$t>,
            )+
        }

        impl $struct_name{
            pub fn new<T>($($name: T,)+) -> Self
            where T: Into<$t> {
                Self{
                    $($name:$name.into().into(),)+
                }
            }

            pub fn values_clone(&self) -> Self{
                Self {
                    $(
                        $name: Reversible::new(self.$name.value()),
                    )+
                }
            }

            paste! {
                $(
                    pub fn [<with_ $name>](mut self, value: $t) -> Self {
                        self.$name.set_value(value);
                        self
                    }

                    pub fn $name(&self) -> $t {
                        self.$name.value()
                    }

                    pub fn [<$name _ref>](&self) -> &Reversible<$t> {
                        &self.$name
                    }

                    pub fn [<$name _ref_mut>](&mut self) -> &mut Reversible<$t> {
                        &mut self.$name
                    }

                    pub fn [<set_ $name>](&mut self, value : $t) {
                        self.$name.set_value(value);
                    }
                )+
            }
        }

        // From tuple
        impl From<($t, $t, $t)> for $struct_name {
            fn from(v: ($t, $t, $t)) -> Self {
                Self::new(v.0, v.1, v.2)
            }
        }

        impl From<&($t, $t, $t)> for $struct_name {
            fn from(v: &($t, $t, $t)) -> Self {
                Self::new(v.0, v.1, v.2)
            }
        }

        // From to Vector3
        impl From<$struct_name> for Vector3<$t> {
            fn from(val: $struct_name) -> Self {
                Vector3::new($(val.$name(),)+)
            }
        }

        impl From<&$struct_name> for Vector3<$t> {
            fn from(val: &$struct_name) -> Self {
                Vector3::new($(val.$name(),)+)
            }
        }

        // From to [T; 3]
        impl From<$struct_name> for [$t; 3] {
            fn from(val: $struct_name) -> Self {
                [$(val.$name(),)+]
            }
        }

        impl From<&$struct_name> for [$t; 3] {
            fn from(val: &$struct_name) -> Self {
                [$(val.$name(),)+]
            }
        }
    };
}

struct_with_vector!(Destination, f32, x, y, z);
struct_with_vector!(Position, f32, x, y, z);
struct_with_vector!(Rotation, f32, yaw, pitch, roll);

impl Position {
    pub fn distance_calculate(&self, target: &Position) -> f32 {
        let dx = self.x() - target.x();
        let dy = self.y() - target.y();
        let dz = self.z() - target.z();
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}