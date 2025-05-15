use std::time::Duration;

use bevy::prelude::*;

use crate::{resources::Resources, scheduling::Sets};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<FactoryTick>()
        .init_resource::<TimeScale>()
        .insert_resource(FactoryTimer(Timer::new(
            Duration::from_secs(1),
            TimerMode::Repeating,
        )))
        .add_systems(Update, time_consume.in_set(Sets::PostUpdate));
}

#[derive(Resource)]
pub struct FactoryTimer(Timer);

#[derive(Event)]
pub struct FactoryTick;

fn time_consume(
    mut timer: ResMut<FactoryTimer>,
    time: Res<Time>,
    mut resources: ResMut<Resources>,
    mut time_scale: ResMut<TimeScale>,
    mut writer: EventWriter<FactoryTick>,
) {
    let time_to_consume = resources.time.min(time.delta_secs() * time_scale.0);
    resources.time -= time_to_consume;
    timer.0.tick(Duration::from_secs_f32(time_to_consume));
    for _ in 0..timer.0.times_finished_this_tick() {
        writer.write(FactoryTick);
    }

    time_scale.0 = if resources.time > 0.0 { 1.0 } else { 0.0 };
}

#[derive(Resource)]
pub struct TimeScale(pub f32);
impl Default for TimeScale {
    fn default() -> Self {
        Self(1.0)
    }
}
