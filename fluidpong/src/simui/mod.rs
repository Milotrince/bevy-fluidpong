pub mod text_input;

use bevy::{input::{keyboard::KeyboardInput, ButtonState}, prelude::*, utils::HashMap};

const BORDER_COLOR_ACTIVE: Color = Color::rgb(0.75, 0.52, 0.99);
const BORDER_COLOR_INACTIVE: Color = Color::rgb(0.25, 0.25, 0.25);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);

pub struct SimUIPlugin {
    pub fluid_type: String,
}

impl Plugin for SimUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(text_input::TextInputPlugin)
            .add_systems(Startup, if self.fluid_type == "sph" { sph_setup } else { ns_setup })
            .add_systems(Update, (update_simvars, focus));
    }
}

#[derive(Component, Clone)]
pub struct FluidSimVars {
    pub map: HashMap<String, f32>,
    pub initialized: bool,
    pub paused: bool,
    pub debug: bool,
    pub interact_mode: bool,
    pub do_reset: bool,
}

impl FluidSimVars {
    pub fn new(map: HashMap<String, f32>) -> Self {
        FluidSimVars {
            map: map,
            initialized: false,
            paused: false,
            debug: false,
            interact_mode: false,
            do_reset: false,
        }
    }

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
        Self { name: String::from(name), value: value, initial: value }
    }
}

fn sph_setup(commands: Commands) {
    let simvars = vec![
        SimVariable::new("kernel_radius", 8.0),
        SimVariable::new("particle_mass", 100.0),
        SimVariable::new("rest_dens", 0.0),
        SimVariable::new("gas_const", 1000.0),
        SimVariable::new("visc_const", 300.0),
        SimVariable::new("bound_damping", 0.5),
        SimVariable::new("gravity", 1.0),
        SimVariable::new("interact_force", 3000.0),
    ];
    setup(commands, simvars);
}

fn ns_setup(commands: Commands) {
    let simvars = vec![
        SimVariable::new("dt", 0.00001),
        SimVariable::new("iter", 4.),
        SimVariable::new("viscosity", 0.0),
        SimVariable::new("diffusion", 0.2),
        SimVariable::new("interact_force", 10.0),
        SimVariable::new("interact_velocity", 1000.0),
        SimVariable::new("dissipation", 0.1),
    ];
    setup(commands, simvars);
}

fn setup(mut commands: Commands, simvars: Vec<SimVariable>) {
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
                        parent.spawn(TextBundle::from_section(
                            simvar.name.clone(),
                            TextStyle { font_size: 12.0, ..default() },
                        ));
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


fn update_simvars(
    mut key_evr: EventReader<KeyboardInput>,
    mut simvars_query: Query<&mut FluidSimVars>,
    query: Query<(&SimVariable, &text_input::TextInputValue)>,
) {
    let mut simvars = simvars_query.single_mut();
    let mut do_update = false;
    for ev in key_evr.read() {
        if ev.state == ButtonState::Released {
            if ev.key_code == KeyCode::Enter {
                do_update = true;
            }
            if ev.key_code == KeyCode::KeyD {
                simvars.debug = !simvars.debug;
                println!("debug: {}", simvars.debug);
            }
            if ev.key_code == KeyCode::KeyP {
                simvars.paused = !simvars.paused;
                println!("paused: {}", simvars.paused);
            }
            if ev.key_code == KeyCode::KeyI {
                simvars.interact_mode = !simvars.interact_mode;
                println!("interact mode: {}", simvars.interact_mode);
            }
            if ev.key_code == KeyCode::KeyR {
                simvars.do_reset = !simvars.do_reset;
                println!("resetting")
            }
        }
    }
    if !simvars.initialized {
        do_update = true;
        simvars.initialized = true;
    }
    if do_update {
        for (simvar, input) in query.iter() {
            let value = input.0.parse::<f32>().unwrap_or(0.0);
            simvars.set(simvar.name.clone(), value);
            println!("updating {} to {}", simvar.name.clone(), value);
        }
    }
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
