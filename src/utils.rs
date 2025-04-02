use bevy::{prelude::*, reflect::GetTypeRegistration};

#[macro_export]
macro_rules! join {
    ($sep:literal, [$head:literal $(, $tail:literal)*] ) => {
        concat!( $head $(, $sep, $tail)* )
    }
}

pub trait AppExt {
    fn register_scoped_type<T>(&mut self, state: impl States + Copy)
    where
        T: FromWorld + GetTypeRegistration + Reflect + Resource;
}

impl AppExt for App {
    fn register_scoped_type<T>(&mut self, state: impl States + Copy)
    where
        T: FromWorld + GetTypeRegistration + Reflect + Resource,
    {
        self.register_type::<T>();
        self.add_systems(OnEnter(state), init_resource::<T>);
        self.add_systems(OnExit(state), remove_resource::<T>);
    }
}

fn init_resource<T>(mut commands: Commands)
where
    T: FromWorld + Reflect + Resource,
{
    commands.init_resource::<T>();
}

fn remove_resource<T>(mut commands: Commands)
where
    T: FromWorld + Reflect + Resource,
{
    commands.remove_resource::<T>();
}
