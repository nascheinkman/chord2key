# chord2key Chorded gamepad input on Linux

## About:

chord2key is a chorded input converter for any linux evdev device. It allows for the control of a
virtual keyboard and mouse using your gamepad. Other gamepad input converters can only emulate a
subset of the keyboard -- each gamepad button is mapped to a single keyboard button. chord2key is
different because it allows for the mapping of combinations of gamepad buttons to a single
keyboard+mouse action. This allows for a single gamepad to have user inputs that map to the entire
keyboard while also controlling the mouse.

## Table of contents:

- [About](#about)
- [Installation](#installation)
    - [Install the Rust compiler](#install-the-rust-compiler)
    - [Download this repository and compile](#download-this-repository-and-compile)
    - [Run](#run)
- [Configuration](#configuration)
- [Input](#input)
    - [Chords](#chords)
    - [Modifiers](#modifiers)
    - [Axis Mouse](#axis-mouse)
    - [Note: Thresholded Axis](#note-thresholded-axis)
- [Actions](#actions)
    - [OutputActions](#outputactions)
        - [Pulse](#pulse)
        - [StateChange](#statechange)
        - [Toggle](#toggle)
    - [InnerActions](#inneractions)
        - [RepeatLastChord](#repeatlastchord)
        - [SwitchConfig](#switchconfig)

## Installation:

The current installation method is to compile the source code.

### Install the Rust compiler:

See https://www.rust-lang.org/tools/install

### Download this repository and compile:

In your preferred terminal: 
```
cd YOUR_INSTALL_DIR
git clone https://github.com/nascheinkman/chord2key.git && cd chord2key
cargo build --release
```

### Run

The program can now be run in the terminal through the following command:
```
sudo YOUR_INSTALL_DIR/chord2key/target/release/chord2key YOUR_CONFIG_FILE
```

## Configuration
The repo comes with configurations for two devices: the Nintendo Switch Pro Controller and Nintendo
Switch Combined Joy-Cons. Both require the
[dkms-hid-nintendo](https://github.com/nicman23/dkms-hid-nintendo) kernel module, and the joysticks
require the [joycond](https://github.com/DanielOgorchock/joycond) daemon. You could then run the
program using the provided configuration file. For instance, if you have the Pro controller, you
would run 
```
cd YOUR_INSTALL_DIR/chord2key
sudo target/release/chord2key configs/nintendo_pro_controller/pro_keyboard.json
```

It's currently difficult to make your own configuration file. If you're tech-savvy, you can look at
the source code and see how the Pro controller configuration was generated
[here](https://github.com/nascheinkman/chord2key/blob/main/src/mapping/configuration.rs#L398), and
edit it to your own needs. You can then save the configuration to a config file through the
`config.save_to_file(FILE_PATH)` function. See the commented lines
[here](https://github.com/nascheinkman/chord2key/blob/7ee11513a4ee7f8cee05ba6ec39c2a7f78d72b1c/src/mapping/configuration.rs#L398)
to see how the Pro controller configuration was saved. 

## Input

Currently, input is separated into three different mappings.

* [Chords](#chords) -> [Action(s)](#actions)
* [Modifiers](#modifiers) -> [Action(s)](#actions)
* [Axis Mouse](#axis-mouse) -> MouseMovement(s)

It's recommended to keep the input sets for each input mapping unique. For instance, if you use the
buttons A,B,X,Y to created chorded input, you should avoid using the button X, or any other one of
those buttons, in modifier input.

### Chords

Chords are a multiple to multiple mapping of button(s) and/or (multiple) thresholded axes to an
output Action(s). Chords are "primed" when a chorded input is added to the current chord, "unprimed"
when a chord action is emitted, and "reprimed" with any subsequent chord input. This means that if
your chord takes 3 buttons, you can hold 2 down and spam the 3rd one to repeatedly emit that Action, 
without unintended side-effects. 

### Modifiers

Modifiers are a one to multiple mapping of a button or thresholded axis to Action(s). The associated
action of a modifier is emitted both when the modifier is pressed down, and when it is released.
These are typically used with Toggle actions so that modifier keys such as SHIFT can be combined
with chorded letter input. 

### Axis Mouse

The mouse mapping maps a single thresholded axis to a linear mouse profile. When the axis passes the
threshold, the relative difference is used to set the mouse velocity. The linear profile allows for
smooth, intuitive mouse output from the axis position. 

### Note: Thresholded Axis

A thresholded axis can be thought of as an axis input with a dead-zone and a threshold direction. If
the axis passes the dead-zone in the indicated direction, it can be thought of as "pressed", and
when it recedes back into the dead-zone it can be thought to be "unpressed". 

## Actions

Actions can be split into two major categories:
* [OutputActions](#outputactions)
* [InnerActions](#inneractions)

### OutputActions

These are standard actions that can be performed on a keyboard.
* [Pulse](#pulse)
* [StateChange](#statechange)
* [Toggle](#toggle)

#### Pulse

You can pulse key(s) and/or mouse. This can be thought of as tapping a keyboard button, or setting a
mouse axis to a certain speed then back to where it was.

#### StateChange

You can set the state of keys/axes directly. This can be thought of as holding a key down/up, or
setting the mouse to a certain speed. Note that it doesn't automatically do the reverse like Toggle.
If a chord is mapped to set a key to down, inputting the chord again won't change anything. 

#### Toggle

You can toggle the state of keys/axes. For keys, this means that if the key is down it's set to up,
and if the key is up it's set to down. This action works really well with Modifier input. For axes,
this means that if the mouse is currently set to the specified speed, it'll be set to 0, otherwise
it'll be set to the specified speed.

### InnerActions

These are special actions that act/depend on the internal state of the program. 
* [RepeatLastChord](#repeatlastchord)
* [SwitchConfig](#switchconfig)

#### RepeatLastChord

This takes the last chord input that resulted in an OutputAction, converts it to the specified type,
then executes that action. This is useful for repeating the last chord a lot. For instance, you can
map A+B as a chord to Pulse the backspace key, and Y as a modifier to toggle RepeatLastChord. This
means you can press A+B to press Backspace once, then press and hold Y to press and hold Backspace.

#### SwitchConfig

This allows you to hot swap the loaded configuration file to another one that operates on the same
input device. This can be used in simple setups to create a blank configuration when you don't want
there to be accidental inputs, that requires a complicated chord to switch to and from a fully
mapped but sensitive keyboard configuration. More complicated setups are theoretically possible, but
untested. 
