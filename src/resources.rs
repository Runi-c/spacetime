use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Resources>();
}

#[derive(Reflect, Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    Health,
    Mineral,
    Gas,
    Time,
    Ammo,
    Rockets,
}

impl ResourceType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Health => "Health".to_string(),
            Self::Mineral => "Minerals".to_string(),
            Self::Gas => "Gas".to_string(),
            Self::Time => "Time".to_string(),
            Self::Ammo => "Ammo".to_string(),
            Self::Rockets => "Rockets".to_string(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct Resources {
    pub health: f32,
    pub minerals: f32,
    pub gas: f32,
    pub time: f32,
    pub ammo: f32,
    pub rockets: f32,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            health: 100.0,
            minerals: 10.0,
            gas: 0.0,
            time: 30.0,
            ammo: 20.0,
            rockets: 0.0,
        }
    }
}

impl Resources {
    pub fn get(&self, resource: ResourceType) -> f32 {
        match resource {
            ResourceType::Health => self.health,
            ResourceType::Mineral => self.minerals,
            ResourceType::Gas => self.gas,
            ResourceType::Time => self.time,
            ResourceType::Ammo => self.ammo,
            ResourceType::Rockets => self.rockets,
        }
    }

    pub fn add(&mut self, resource: ResourceType, amount: f32) {
        match resource {
            ResourceType::Health => self.health += amount,
            ResourceType::Mineral => self.minerals += amount,
            ResourceType::Gas => self.gas += amount,
            ResourceType::Time => self.time += amount,
            ResourceType::Ammo => self.ammo += amount,
            ResourceType::Rockets => self.rockets += amount,
        }
    }
}
