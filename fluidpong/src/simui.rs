use crate::lib::text_input;
use bevy::{prelude::*, utils::HashMap};

const BORDER_COLOR_ACTIVE: Color = Color::rgb(0.75, 0.52, 0.99);
const BORDER_COLOR_INACTIVE: Color = Color::rgb(0.25, 0.25, 0.25);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);

pub struct SimUIPlugin;

impl Plugin for SimUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(text_input::TextInputPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, focus);
    }
}

#[derive(Component, Clone)]
pub struct FluidSimVars {
    pub map: HashMap<String, f32>,
    pub initialized: bool,
    pub paused: bool,
    pub debug: bool,
    pub interact_mode: bool,
}

impl FluidSimVars {
    pub fn get(&self, key: &str) -> f32 {
        if let Some(value) = self.map.get(key) {
            return *value;
        }
        return 0.0;
    }
    pub fn set(&mut self, key: String, value: f32) {
        self.map.insert(key, value);
    }
}

#[derive(Component, Clone)]
pub struct SimVariable {
    pub name: String,
    pub value: f32,
    pub initial: f32,
}

impl SimVariable {
    fn new(name: &str, value: f32) -> Self {
        Self {
            name: String::from(name),
            value: value,
            initial: value,
        }
    }
}

fn setup(mut commands: Commands) {
    let simvars = [
        // SimVariable::new("sim_speed", 10.0),
        // SimVariable::new("gravity", 9.81),
        // SimVariable::new("restitution", 0.2),
        // SimVariable::new("friction", 0.8),
        // SimVariable::new("viscosity", 0.01),
        // SimVariable::new("pressure", 1.0),
        // SimVariable::new("interact_force", 10.0),
        // SimVariable::new("interact_radius", 50.0),
        // SimVariable::new("threshold_radius", 5.0),
        // SimVariable::new("smoothing_radius", 4.0),
        // SimVariable::new("wall_x", 200.0),
        // SimVariable::new("wall_y", 200.0),

        SimVariable::new("dt", 0.00001),
        SimVariable::new("viscosity", 0.0),
        SimVariable::new("diffusion", 0.2),
        SimVariable::new("iter", 4.),
        SimVariable::new("interact_force", 10.0),
        SimVariable::new("interact_velocity", 1000.0),
        SimVariable::new("dissipation", 0.001),
    ];
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                ..default()
            },
            Interaction::None,
        ))
        .with_children(|uiparent| {
            for simvar in simvars {
                uiparent
                    .spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(200.0),
                                // height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            ..default()
                        },
                        Interaction::None,
                    ))
                    .with_children(|parent| {
                        parent.spawn(
                            (TextBundle::from_section(
                                simvar.name.clone(),
                                TextStyle {
                                    font_size: 12.0,
                                    ..default()
                                },
                            )),
                        );
                        parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(200.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    padding: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                border_color: BORDER_COLOR_INACTIVE.into(),
                                background_color: BACKGROUND_COLOR.into(),
                                ..default()
                            },
                            text_input::TextInputBundle::default()
                                .with_text_style(TextStyle {
                                    font_size: 14.,
                                    color: TEXT_COLOR,
                                    ..default()
                                })
                                .with_value(simvar.value.to_string())
                                .with_settings(text_input::TextInputSettings {
                                    retain_on_submit: true,
                                })
                                .with_inactive(true),
                            simvar,
                        ));
                    });
            }
        });
}

fn focus(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut text_input_query: Query<(Entity, &mut text_input::TextInputInactive, &mut BorderColor)>,
) {
    for (interaction_entity, interaction) in &query {
        if *interaction == Interaction::Pressed {
            for (entity, mut inactive, mut border_color) in &mut text_input_query {
                if entity == interaction_entity {
                    inactive.0 = false;
                    *border_color = BORDER_COLOR_ACTIVE.into();
                } else {
                    inactive.0 = true;
                    *border_color = BORDER_COLOR_INACTIVE.into();
                }
            }
        }
    }
}
