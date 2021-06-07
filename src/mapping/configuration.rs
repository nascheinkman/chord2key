use super::actions::*;
use super::mapper::*;
use super::thresholds::*;
use crate::constants::*;
use crate::output::actions::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// The name of the evdev input device that this configuration should apply for
    pub device_name: String,

    /// The thresholds for axis input to be considered valid. Analogous to axis dead zones.
    pub axis_thresholds: Vec<(AbsAxisCode, AxisThreshold)>,

    /// All inputs that should be considered for chording.
    ///
    /// Be very careful when using the same input in both chords and modifiers, as the
    /// modifier+chord input can invalidate your expected chord inputs unexpectedly.
    pub chord_inputs: Vec<ChordInput>,

    /// Mapping of chords to actions
    pub chord_mapping: ChordMapInput,

    /// Mapping of modifiers to actions
    pub modifier_mapping: ModifierMapInput,

    /// Mapping of absolute axes inputs to mouse actions
    pub mouse_mapping: MouseMapInput,
}

impl Configuration {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer_pretty(&File::create(path).unwrap(), self)?;
        Ok(())
    }
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let config = serde_json::from_reader(reader)?;

        Ok(config)
    }

    #[allow(non_snake_case)]
    pub fn joycon_default() -> Self {
        let device_name = String::from("Nintendo Switch Combined Joy-Cons");
        let axis_thresholds: Vec<(AbsAxisCode, AxisThreshold)> = vec![
            (AbsAxisCode::ABS_X, (ThresholdType::Greater, 6000).into()),
            (AbsAxisCode::ABS_X, (ThresholdType::Lesser, -6000).into()),
            (AbsAxisCode::ABS_Y, (ThresholdType::Greater, 6000).into()),
            (AbsAxisCode::ABS_Y, (ThresholdType::Lesser, -6000).into()),
            (AbsAxisCode::ABS_RX, (ThresholdType::Greater, 16000).into()),
            (AbsAxisCode::ABS_RX, (ThresholdType::Lesser, -16000).into()),
            (AbsAxisCode::ABS_RY, (ThresholdType::Greater, 16000).into()),
            (AbsAxisCode::ABS_RY, (ThresholdType::Lesser, -16000).into()),
        ];
        const SENSITIVITY: f64 = 0.0006;
        let mouse_mapping: MouseMapInput = vec![
            (
                (AbsAxisCode::ABS_X, ThresholdType::Greater).into(),
                MouseProfile {
                    code: RelAxisCode::REL_X,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
            (
                (AbsAxisCode::ABS_X, ThresholdType::Lesser).into(),
                MouseProfile {
                    code: RelAxisCode::REL_X,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
            (
                (AbsAxisCode::ABS_Y, ThresholdType::Greater).into(),
                MouseProfile {
                    code: RelAxisCode::REL_Y,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
            (
                (AbsAxisCode::ABS_Y, ThresholdType::Lesser).into(),
                MouseProfile {
                    code: RelAxisCode::REL_Y,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
        ];
        let modifier_mapping: ModifierMapInput = vec![
            (
                KeyCode::BTN_TR2.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTSHIFT]), None).into(),
            ),
            (
                KeyCode::BTN_TL2.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTCTRL]), None).into(),
            ),
            (
                KeyCode::BTN_TL.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTMETA]), None).into(),
            ),
            (
                KeyCode::BTN_TR.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTALT]), None).into(),
            ),
            (
                KeyCode::BTN_START.into(),
                InnerAction::RepeatLastChord(OutputActionType::Toggle).into(),
            ),
        ];
        let B: ChordInput = KeyCode::BTN_SOUTH.into();
        let Y: ChordInput = KeyCode::BTN_WEST.into();
        let X: ChordInput = KeyCode::BTN_NORTH.into();
        let A: ChordInput = KeyCode::BTN_EAST.into();
        let Right: ChordInput = KeyCode::BTN_DPAD_RIGHT.into();
        let Left: ChordInput = KeyCode::BTN_DPAD_LEFT.into();
        let Up: ChordInput = KeyCode::BTN_DPAD_UP.into();
        let Down: ChordInput = KeyCode::BTN_DPAD_DOWN.into();
        let RSU: ChordInput = (AbsAxisCode::ABS_RY, ThresholdType::Lesser).into();
        let RSD: ChordInput = (AbsAxisCode::ABS_RY, ThresholdType::Greater).into();
        let RSR: ChordInput = (AbsAxisCode::ABS_RX, ThresholdType::Greater).into();
        let RSL: ChordInput = (AbsAxisCode::ABS_RX, ThresholdType::Lesser).into();
        let RSC: ChordInput = KeyCode::BTN_THUMBR.into();
        let Minus: ChordInput = KeyCode::BTN_SELECT.into();
        let Home: ChordInput = KeyCode::BTN_MODE.into();
        let Capture: ChordInput = KeyCode::BTN_Z.into();

        let chord_inputs: Vec<ChordInput> = vec![
            B,
            Y,
            X,
            A,
            Right,
            Left,
            Up,
            Down,
            RSU,
            RSD,
            RSR,
            RSL,
            RSC,
            KeyCode::BTN_THUMBL.into(),
            Minus,
            Home,
            Capture,
        ];
        let chord_mapping: ChordMapInput = vec![
            (
                vec![Capture, A, B, X, Y],
                InnerAction::SwitchConfig(Path::new("joycon_blank.json").to_path_buf()).into(),
            ),
            (
                vec![Up, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_APOSTROPHE]), None).into(),
            ),
            (
                vec![Right],
                Pulse::new(Some(vec![KeyCode::KEY_0]), None).into(),
            ),
            (
                vec![Right, RSU],
                Pulse::new(Some(vec![KeyCode::KEY_1]), None).into(),
            ),
            (
                vec![Right, RSU, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_2]), None).into(),
            ),
            (
                vec![Right, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_3]), None).into(),
            ),
            (
                vec![Right, RSR, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_4]), None).into(),
            ),
            (
                vec![Right, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_5]), None).into(),
            ),
            (
                vec![Right, RSD, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_6]), None).into(),
            ),
            (
                vec![Right, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_7]), None).into(),
            ),
            (
                vec![Right, RSL, RSU],
                Pulse::new(Some(vec![KeyCode::KEY_8]), None).into(),
            ),
            (
                vec![Right, RSC],
                Pulse::new(Some(vec![KeyCode::KEY_9]), None).into(),
            ),
            (
                vec![Down, B],
                Pulse::new(Some(vec![KeyCode::KEY_A]), None).into(),
            ),
            (
                vec![Up, B],
                Pulse::new(Some(vec![KeyCode::KEY_B]), None).into(),
            ),
            (
                vec![Right, A],
                Pulse::new(Some(vec![KeyCode::KEY_C]), None).into(),
            ),
            (
                vec![Down],
                Pulse::new(Some(vec![KeyCode::KEY_D]), None).into(),
            ),
            (
                vec![Up, X],
                Pulse::new(Some(vec![KeyCode::KEY_E]), None).into(),
            ),
            (
                vec![A, B],
                Pulse::new(Some(vec![KeyCode::KEY_F]), None).into(),
            ),
            (
                vec![X, Y],
                Pulse::new(Some(vec![KeyCode::KEY_G]), None).into(),
            ),
            (vec![Y], Pulse::new(Some(vec![KeyCode::KEY_H]), None).into()),
            (
                vec![Right, X],
                Pulse::new(Some(vec![KeyCode::KEY_I]), None).into(),
            ),
            (vec![B], Pulse::new(Some(vec![KeyCode::KEY_J]), None).into()),
            (vec![X], Pulse::new(Some(vec![KeyCode::KEY_K]), None).into()),
            (vec![A], Pulse::new(Some(vec![KeyCode::KEY_L]), None).into()),
            (
                vec![Right, A, B],
                Pulse::new(Some(vec![KeyCode::KEY_M]), None).into(),
            ),
            (
                vec![Y, B],
                Pulse::new(Some(vec![KeyCode::KEY_N]), None).into(),
            ),
            (
                vec![Right, B],
                Pulse::new(Some(vec![KeyCode::KEY_O]), None).into(),
            ),
            (
                vec![Down, A],
                Pulse::new(Some(vec![KeyCode::KEY_P]), None).into(),
            ),
            (
                vec![Right, X, Y],
                Pulse::new(Some(vec![KeyCode::KEY_Q]), None).into(),
            ),
            (
                vec![Down, Y, B],
                Pulse::new(Some(vec![KeyCode::KEY_R]), None).into(),
            ),
            (
                vec![Right, Y],
                Pulse::new(Some(vec![KeyCode::KEY_S]), None).into(),
            ),
            (
                vec![X, A],
                Pulse::new(Some(vec![KeyCode::KEY_T]), None).into(),
            ),
            (
                vec![Up],
                Pulse::new(Some(vec![KeyCode::KEY_U]), None).into(),
            ),
            (
                vec![Up, Y],
                Pulse::new(Some(vec![KeyCode::KEY_V]), None).into(),
            ),
            (
                vec![Up, A],
                Pulse::new(Some(vec![KeyCode::KEY_W]), None).into(),
            ),
            (
                vec![Down, X],
                Pulse::new(Some(vec![KeyCode::KEY_X]), None).into(),
            ),
            (
                vec![Down, Y],
                Pulse::new(Some(vec![KeyCode::KEY_Y]), None).into(),
            ),
            (
                vec![Right, A, X],
                Pulse::new(Some(vec![KeyCode::KEY_Z]), None).into(),
            ),
            (
                vec![Minus, A],
                Pulse::new(Some(vec![KeyCode::KEY_LEFTBRACE]), None).into(),
            ),
            (
                vec![Minus, Y],
                Pulse::new(Some(vec![KeyCode::KEY_RIGHTBRACE]), None).into(),
            ),
            (
                vec![Minus, X, A],
                Pulse::new(Some(vec![KeyCode::KEY_SEMICOLON]), None).into(),
            ),
            (
                vec![Minus, B],
                Pulse::new(Some(vec![KeyCode::KEY_EQUAL]), None).into(),
            ),
            (
                vec![Minus, A, B],
                Pulse::new(Some(vec![KeyCode::KEY_COMMA]), None).into(),
            ),
            (
                vec![Minus, B, Y],
                Pulse::new(Some(vec![KeyCode::KEY_DOT]), None).into(),
            ),
            (
                vec![Minus, X],
                Pulse::new(Some(vec![KeyCode::KEY_MINUS]), None).into(),
            ),
            (
                vec![Minus, X, Y],
                Pulse::new(Some(vec![KeyCode::KEY_SLASH]), None).into(),
            ),
            (
                vec![Up, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_BACKSLASH]), None).into(),
            ),
            (
                vec![Up, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_SPACE]), None).into(),
            ),
            (
                vec![Left, RSU],
                Pulse::new(Some(vec![KeyCode::KEY_UP]), None).into(),
            ),
            (
                vec![Left, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_DOWN]), None).into(),
            ),
            (
                vec![Left, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_LEFT]), None).into(),
            ),
            (
                vec![Left, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_RIGHT]), None).into(),
            ),
            (
                vec![Minus],
                Pulse::new(Some(vec![KeyCode::KEY_BACKSPACE]), None).into(),
            ),
            (
                vec![Home],
                Pulse::new(Some(vec![KeyCode::KEY_ENTER]), None).into(),
            ),
            (
                vec![Capture],
                Pulse::new(Some(vec![KeyCode::KEY_ESC]), None).into(),
            ),
            (
                vec![Capture, Home],
                Pulse::new(Some(vec![KeyCode::KEY_TAB]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into()],
                Pulse::new(Some(vec![KeyCode::BTN_LEFT]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into(), B],
                Toggle::new(Some(vec![KeyCode::BTN_LEFT]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into(), Y],
                Pulse::new(Some(vec![KeyCode::BTN_RIGHT]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into(), B, Y],
                Toggle::new(Some(vec![KeyCode::BTN_RIGHT]), None).into(),
            ),
        ];

        Self {
            device_name: device_name,
            axis_thresholds: axis_thresholds,
            chord_inputs: chord_inputs,
            chord_mapping: chord_mapping,
            modifier_mapping: modifier_mapping,
            mouse_mapping: mouse_mapping,
        }
    }

    #[allow(non_snake_case)]
    pub fn pro_default() -> Self {
        let device_name = String::from("Nintendo Switch Pro Controller");
        let axis_thresholds: Vec<(AbsAxisCode, AxisThreshold)> = vec![
            (AbsAxisCode::ABS_X, (ThresholdType::Greater, 2000).into()),
            (AbsAxisCode::ABS_X, (ThresholdType::Lesser, -2000).into()),
            (AbsAxisCode::ABS_Y, (ThresholdType::Greater, 2000).into()),
            (AbsAxisCode::ABS_Y, (ThresholdType::Lesser, -2000).into()),
            (AbsAxisCode::ABS_RX, (ThresholdType::Greater, 16000).into()),
            (AbsAxisCode::ABS_RX, (ThresholdType::Lesser, -16000).into()),
            (AbsAxisCode::ABS_RY, (ThresholdType::Greater, 16000).into()),
            (AbsAxisCode::ABS_RY, (ThresholdType::Lesser, -16000).into()),
            (AbsAxisCode::ABS_HAT0X, (ThresholdType::Greater, 1).into()),
            (AbsAxisCode::ABS_HAT0X, (ThresholdType::Lesser, -1).into()),
            (AbsAxisCode::ABS_HAT0Y, (ThresholdType::Greater, 1).into()),
            (AbsAxisCode::ABS_HAT0Y, (ThresholdType::Lesser, -1).into()),
        ];
        const SENSITIVITY: f64 = 0.0006;
        let mouse_mapping: MouseMapInput = vec![
            (
                (AbsAxisCode::ABS_X, ThresholdType::Greater).into(),
                MouseProfile {
                    code: RelAxisCode::REL_X,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
            (
                (AbsAxisCode::ABS_X, ThresholdType::Lesser).into(),
                MouseProfile {
                    code: RelAxisCode::REL_X,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
            (
                (AbsAxisCode::ABS_Y, ThresholdType::Greater).into(),
                MouseProfile {
                    code: RelAxisCode::REL_Y,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
            (
                (AbsAxisCode::ABS_Y, ThresholdType::Lesser).into(),
                MouseProfile {
                    code: RelAxisCode::REL_Y,
                    slope: SENSITIVITY,
                    offset: 0.0,
                },
            ),
        ];
        let modifier_mapping: ModifierMapInput = vec![
            (
                KeyCode::BTN_TR2.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTSHIFT]), None).into(),
            ),
            (
                KeyCode::BTN_TL2.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTCTRL]), None).into(),
            ),
            (
                KeyCode::BTN_TL.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTMETA]), None).into(),
            ),
            (
                KeyCode::BTN_TR.into(),
                Toggle::new(Some(vec![KeyCode::KEY_LEFTALT]), None).into(),
            ),
            (
                KeyCode::BTN_START.into(),
                InnerAction::RepeatLastChord(OutputActionType::Toggle).into(),
            ),
        ];
        let B: ChordInput = KeyCode::BTN_SOUTH.into();
        let Y: ChordInput = KeyCode::BTN_WEST.into();
        let X: ChordInput = KeyCode::BTN_NORTH.into();
        let A: ChordInput = KeyCode::BTN_EAST.into();
        let Right: ChordInput = (AbsAxisCode::ABS_HAT0X, ThresholdType::Greater).into();
        let Left: ChordInput = (AbsAxisCode::ABS_HAT0X, ThresholdType::Lesser).into();
        let Up: ChordInput = (AbsAxisCode::ABS_HAT0Y, ThresholdType::Lesser).into();
        let Down: ChordInput = (AbsAxisCode::ABS_HAT0Y, ThresholdType::Greater).into();
        let RSU: ChordInput = (AbsAxisCode::ABS_RY, ThresholdType::Lesser).into();
        let RSD: ChordInput = (AbsAxisCode::ABS_RY, ThresholdType::Greater).into();
        let RSR: ChordInput = (AbsAxisCode::ABS_RX, ThresholdType::Greater).into();
        let RSL: ChordInput = (AbsAxisCode::ABS_RX, ThresholdType::Lesser).into();
        let RSC: ChordInput = KeyCode::BTN_THUMBR.into();
        let Minus: ChordInput = KeyCode::BTN_SELECT.into();
        let Home: ChordInput = KeyCode::BTN_MODE.into();
        let Capture: ChordInput = KeyCode::BTN_Z.into();

        let chord_inputs: Vec<ChordInput> = vec![
            B,
            Y,
            X,
            A,
            Right,
            Left,
            Up,
            Down,
            RSU,
            RSD,
            RSR,
            RSL,
            RSC,
            KeyCode::BTN_THUMBL.into(),
            Minus,
            Home,
            Capture,
        ];
        let chord_mapping: ChordMapInput = vec![
            (
                vec![Capture, A, B, X, Y],
                InnerAction::SwitchConfig(Path::new("pro_blank.json").to_path_buf()).into(),
            ),
            (
                vec![Up, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_APOSTROPHE]), None).into(),
            ),
            (
                vec![Right],
                Pulse::new(Some(vec![KeyCode::KEY_0]), None).into(),
            ),
            (
                vec![Right, RSU],
                Pulse::new(Some(vec![KeyCode::KEY_1]), None).into(),
            ),
            (
                vec![Right, RSU, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_2]), None).into(),
            ),
            (
                vec![Right, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_3]), None).into(),
            ),
            (
                vec![Right, RSR, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_4]), None).into(),
            ),
            (
                vec![Right, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_5]), None).into(),
            ),
            (
                vec![Right, RSD, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_6]), None).into(),
            ),
            (
                vec![Right, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_7]), None).into(),
            ),
            (
                vec![Right, RSL, RSU],
                Pulse::new(Some(vec![KeyCode::KEY_8]), None).into(),
            ),
            (
                vec![Right, RSC],
                Pulse::new(Some(vec![KeyCode::KEY_9]), None).into(),
            ),
            (
                vec![Down, B],
                Pulse::new(Some(vec![KeyCode::KEY_A]), None).into(),
            ),
            (
                vec![Up, B],
                Pulse::new(Some(vec![KeyCode::KEY_B]), None).into(),
            ),
            (
                vec![Right, A],
                Pulse::new(Some(vec![KeyCode::KEY_C]), None).into(),
            ),
            (
                vec![Down],
                Pulse::new(Some(vec![KeyCode::KEY_D]), None).into(),
            ),
            (
                vec![Up, X],
                Pulse::new(Some(vec![KeyCode::KEY_E]), None).into(),
            ),
            (
                vec![A, B],
                Pulse::new(Some(vec![KeyCode::KEY_F]), None).into(),
            ),
            (
                vec![X, Y],
                Pulse::new(Some(vec![KeyCode::KEY_G]), None).into(),
            ),
            (vec![Y], Pulse::new(Some(vec![KeyCode::KEY_H]), None).into()),
            (
                vec![Right, X],
                Pulse::new(Some(vec![KeyCode::KEY_I]), None).into(),
            ),
            (vec![B], Pulse::new(Some(vec![KeyCode::KEY_J]), None).into()),
            (vec![X], Pulse::new(Some(vec![KeyCode::KEY_K]), None).into()),
            (vec![A], Pulse::new(Some(vec![KeyCode::KEY_L]), None).into()),
            (
                vec![Right, A, B],
                Pulse::new(Some(vec![KeyCode::KEY_M]), None).into(),
            ),
            (
                vec![Y, B],
                Pulse::new(Some(vec![KeyCode::KEY_N]), None).into(),
            ),
            (
                vec![Right, B],
                Pulse::new(Some(vec![KeyCode::KEY_O]), None).into(),
            ),
            (
                vec![Down, A],
                Pulse::new(Some(vec![KeyCode::KEY_P]), None).into(),
            ),
            (
                vec![Right, X, Y],
                Pulse::new(Some(vec![KeyCode::KEY_Q]), None).into(),
            ),
            (
                vec![Down, Y, B],
                Pulse::new(Some(vec![KeyCode::KEY_R]), None).into(),
            ),
            (
                vec![Right, Y],
                Pulse::new(Some(vec![KeyCode::KEY_S]), None).into(),
            ),
            (
                vec![X, A],
                Pulse::new(Some(vec![KeyCode::KEY_T]), None).into(),
            ),
            (
                vec![Up],
                Pulse::new(Some(vec![KeyCode::KEY_U]), None).into(),
            ),
            (
                vec![Up, Y],
                Pulse::new(Some(vec![KeyCode::KEY_V]), None).into(),
            ),
            (
                vec![Up, A],
                Pulse::new(Some(vec![KeyCode::KEY_W]), None).into(),
            ),
            (
                vec![Down, X],
                Pulse::new(Some(vec![KeyCode::KEY_X]), None).into(),
            ),
            (
                vec![Down, Y],
                Pulse::new(Some(vec![KeyCode::KEY_Y]), None).into(),
            ),
            (
                vec![Right, A, X],
                Pulse::new(Some(vec![KeyCode::KEY_Z]), None).into(),
            ),
            (
                vec![Minus, A],
                Pulse::new(Some(vec![KeyCode::KEY_LEFTBRACE]), None).into(),
            ),
            (
                vec![Minus, Y],
                Pulse::new(Some(vec![KeyCode::KEY_RIGHTBRACE]), None).into(),
            ),
            (
                vec![Minus, X, A],
                Pulse::new(Some(vec![KeyCode::KEY_SEMICOLON]), None).into(),
            ),
            (
                vec![Minus, B],
                Pulse::new(Some(vec![KeyCode::KEY_EQUAL]), None).into(),
            ),
            (
                vec![Minus, A, B],
                Pulse::new(Some(vec![KeyCode::KEY_COMMA]), None).into(),
            ),
            (
                vec![Minus, B, Y],
                Pulse::new(Some(vec![KeyCode::KEY_DOT]), None).into(),
            ),
            (
                vec![Minus, X],
                Pulse::new(Some(vec![KeyCode::KEY_MINUS]), None).into(),
            ),
            (
                vec![Minus, X, Y],
                Pulse::new(Some(vec![KeyCode::KEY_SLASH]), None).into(),
            ),
            (
                vec![Up, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_BACKSLASH]), None).into(),
            ),
            (
                vec![Up, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_SPACE]), None).into(),
            ),
            (
                vec![Left, RSU],
                Pulse::new(Some(vec![KeyCode::KEY_UP]), None).into(),
            ),
            (
                vec![Left, RSD],
                Pulse::new(Some(vec![KeyCode::KEY_DOWN]), None).into(),
            ),
            (
                vec![Left, RSL],
                Pulse::new(Some(vec![KeyCode::KEY_LEFT]), None).into(),
            ),
            (
                vec![Left, RSR],
                Pulse::new(Some(vec![KeyCode::KEY_RIGHT]), None).into(),
            ),
            (
                vec![Minus],
                Pulse::new(Some(vec![KeyCode::KEY_BACKSPACE]), None).into(),
            ),
            (
                vec![Home],
                Pulse::new(Some(vec![KeyCode::KEY_ENTER]), None).into(),
            ),
            (
                vec![Capture],
                Pulse::new(Some(vec![KeyCode::KEY_ESC]), None).into(),
            ),
            (
                vec![Capture, Home],
                Pulse::new(Some(vec![KeyCode::KEY_TAB]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into()],
                Pulse::new(Some(vec![KeyCode::BTN_LEFT]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into(), B],
                Toggle::new(Some(vec![KeyCode::BTN_LEFT]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into(), Y],
                Pulse::new(Some(vec![KeyCode::BTN_RIGHT]), None).into(),
            ),
            (
                vec![KeyCode::BTN_THUMBL.into(), B, Y],
                Toggle::new(Some(vec![KeyCode::BTN_RIGHT]), None).into(),
            ),
        ];

        Self {
            device_name: device_name,
            axis_thresholds: axis_thresholds,
            chord_inputs: chord_inputs,
            chord_mapping: chord_mapping,
            modifier_mapping: modifier_mapping,
            mouse_mapping: mouse_mapping,
        }
    }
}