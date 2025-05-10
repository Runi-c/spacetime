use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<Resources>();
}

#[derive(Component, Clone, Copy, Debug)]
pub enum ResourceType {
    Mineral,
    Gas,
    Time,
    Ammo,
}

impl ResourceType {
    pub fn to_string(&self) -> String {
        match self {
            ResourceType::Mineral => "Minerals".to_string(),
            ResourceType::Gas => "Gas".to_string(),
            ResourceType::Time => "Time".to_string(),
            ResourceType::Ammo => "Ammo".to_string(),
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct Resources {
    pub minerals: f32,
    pub gas: f32,
    pub time: f32,
    pub ammo: f32,
}

impl Resources {
    pub fn get(&self, resource: ResourceType) -> f32 {
        match resource {
            ResourceType::Mineral => self.minerals,
            ResourceType::Gas => self.gas,
            ResourceType::Time => self.time,
            _ => 0.0,
        }
    }
}
