# Linux Virtual Joystick
This is a simple program that creates a virtual joystick on your computer. I use it to develop software that needs a joystick, without actually having a joystick on-hand. it was inspired the basic functionality of [vJoy](http://vjoystick.sourceforge.net/site/). 

## Compilation
To compile from source you will need [cargo and rust](https://www.rust-lang.org/tools/install).To install from source clone this repository and `cargo build`. If the compilation fail's you may need `libudev-dev`, and `pkg-config` on your distribution.

## Usage
To start the program just `cargo run`. You may also need to enable uinput with `modprobe uinput`. Your user will also need `rw` access to /dev/uinput for this program to run.