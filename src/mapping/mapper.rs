use super::actions::*;
use super::configuration::*;
use super::maps::chord_map::ChordMap;
use super::maps::modifier_map::ModifierMap;
use super::maps::mouse_map::MouseMap;
use super::thresholds::*;
use crate::constants::*;
use crate::input::events::InputEvent;
use crate::output::actions::*;
use crate::output::device::*;
use crate::strum::IntoEnumIterator;
use std::collections::HashMap;
use std::error::Error;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

pub use super::maps::chord_map::{Chord, ChordInput, ChordMapInput};
pub use super::maps::modifier_map::{ModifierInput, ModifierMapInput};
pub use super::maps::mouse_map::{MouseInput, MouseMapInput, MouseProfile};

struct Maps {
    pub chords: ChordMap,
    pub modifiers: ModifierMap,
    pub mouse: MouseMap,
}
pub struct Mapper {
    output_device: OutputDevice,
    input_device_name: String,
    current_config_index: usize,
    mappings_vec: Vec<Maps>,
    paths_to_indices: HashMap<Option<PathBuf>, usize>,
}

impl Mapper {
    fn get_mappings(config: Configuration) -> Maps {
        let thresholds = AllAxisThresholds::init(config.axis_thresholds);
        let chord_mapping = ChordMap::init(config.chord_mapping, thresholds.clone());
        let modifier_mapping = ModifierMap::init(config.modifier_mapping, thresholds.clone());
        let mouse_mapping = MouseMap::init(config.mouse_mapping, thresholds.clone());

        Maps {
            chords: chord_mapping,
            modifiers: modifier_mapping,
            mouse: mouse_mapping,
        }
    }
    pub fn init_from_file<P: AsRef<Path>>(
        device: OutputDevice,
        path: P,
    ) -> Result<Self, Box<dyn Error>> {
        let pathbuf = path.as_ref().to_path_buf().canonicalize()?;
        let mut device_name: Option<String> = None;
        let mut paths_to_indices = HashMap::<Option<PathBuf>, usize>::new();
        let mut mappings_vec = Vec::<Maps>::new();
        let mut config_paths: Vec<PathBuf> = vec![pathbuf.clone()];

        let mut i = 0;

        // Populate mappings with all the unique linked configurations
        while i < config_paths.len() {
            // Prevent duplicate mappings
            if paths_to_indices.contains_key(&Some(config_paths[i].to_path_buf())) {
                i += 1;
                continue;
            }

            // Load the configuration file
            let config = Configuration::load_from_file(&config_paths[i])?;

            // Check for same device name
            match &mut device_name {
                Some(name) => {
                    if name != &config.device_name {
                        println!("{} != {}", name, &config.device_name);
                        return Err(
                            Box::new(
                                std::io::Error::new(
                                    std::io::ErrorKind::Other, 
                                    format!("Device in configuration file {:?} does not match \
                                            device in source file", &config_paths[i])
                                )
                            )
                        );
                    }
                }
                None => {
                    device_name = Some(config.device_name.clone());
                }
            }

            // Populate mappings
            let mut maps = Self::get_mappings(config);

            // Look at all the actions in the chord_mapping
            maps.chords.peek_actions_mut()
                // Look at all the actions in the modifier_mapping
                .chain(maps.modifiers.peek_actions_mut())
                // Find all configuration-related actions
                .try_for_each(|a| -> Result<_, Box<dyn Error>> {
                    if let Action::InnerAction(InnerAction::SwitchConfig(p)) = a {
                        // Get the config file path
                        let mut p = p.to_path_buf();

                        // Ensure the config file path is an absolute one
                        if p.is_relative() {
                            p = config_paths[i]
                                .parent()
                                .ok_or("A configuration file is somehow the root directory")?
                                .join(p)
                                .canonicalize()?;
                            *a = Action::InnerAction(InnerAction::SwitchConfig(p.clone()));
                        }

                        // Push new config files to the stack
                        if !paths_to_indices.contains_key(&Some(p.to_path_buf())) {
                            config_paths.push(p);
                        }
                    }
                    Ok(())
                })?;

            // Save mappings
            paths_to_indices.insert(Some(config_paths[i].clone()), i);
            mappings_vec.push(maps);
            i += 1;
        }

        let device_name = device_name.unwrap();

        Ok(Self {
            output_device: device,
            input_device_name: device_name,
            current_config_index: 0,
            paths_to_indices: paths_to_indices,
            mappings_vec: mappings_vec,
        })
    }

    pub fn get_input_name(&self) -> &str {
        &self.input_device_name
    }

    fn clear_all(&mut self) {
        self.get_chord_mapping_mut().clear_state();
        let handsoff: StateChange = StateChange::new(
            Some(KeyStateChange {
                keys: KeyCode::iter().collect(),
                state: PressState::Up,
            }),
            Some(AxisList::from_iter(
                RelAxisCode::iter().map(|code| RelAxisEvent::new(code, 0)),
            )),
        );
        self.handle_action(handsoff.into());
    }

    fn switch_config(&mut self, path: std::path::PathBuf) {
        self.current_config_index = *self.paths_to_indices.get(&Some(path)).unwrap();
        self.clear_all();
    }

    fn get_chord_mapping(&self) -> &ChordMap {
        &self.mappings_vec[self.current_config_index].chords
    }

    fn get_chord_mapping_mut(&mut self) -> &mut ChordMap {
        &mut self.mappings_vec[self.current_config_index].chords
    }

    fn get_modifier_mapping_mut(&mut self) -> &mut ModifierMap {
        &mut self.mappings_vec[self.current_config_index].modifiers
    }

    fn get_mouse_mapping_mut(&mut self) -> &mut MouseMap {
        &mut self.mappings_vec[self.current_config_index].mouse
    }

    fn repeat_last_chord(&mut self, act_type: OutputActionType) {
        let act_opt = self.get_chord_mapping().get_prev_action();
        if let Some(Action::OutputAction(act)) = act_opt {
            match act_type {
                OutputActionType::Pulse => {
                    let pulse: Pulse = act.into();
                    self.handle_action(pulse.into());
                }
                OutputActionType::StateChange => {
                    let sc: StateChange = act.into();
                    self.handle_action(sc.into());
                }
                OutputActionType::Toggle => {
                    let t: Toggle = act.into();
                    self.handle_action(t.into());
                }
            }
        }
    }

    fn handle_inner_action(&mut self, ev: InnerAction) {
        match ev {
            InnerAction::RepeatLastChord(act_type) => {
                self.repeat_last_chord(act_type);
            }
            InnerAction::SwitchConfig(path) => {
                self.switch_config(path);
            }
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::OutputAction(oe) => {
                self.output_device.execute_event(oe).ok();
            }
            Action::InnerAction(ie) => {
                self.handle_inner_action(ie);
            }
        }
    }

    pub fn handle_event(&mut self, ev: &InputEvent) {
        let chord_act_opt = self.get_chord_mapping_mut().handle_event(ev);
        if let Some(action) = chord_act_opt {
            self.handle_action(action);
        }

        let modifier_act_dbl_opt = self.get_modifier_mapping_mut().handle_event(ev);
        if let Some((act, next_act_opt)) = modifier_act_dbl_opt {
            self.handle_action(act);
            if let Some(next_act) = next_act_opt {
                self.handle_action(next_act);
            }
        }

        let mouse_act_dbl_opt = self.get_mouse_mapping_mut().handle_event(ev);
        if let Some((act, next_act_opt)) = mouse_act_dbl_opt {
            self.handle_action(act);
            if let Some(next_act) = next_act_opt {
                self.handle_action(next_act);
            }
        }
    }
}
