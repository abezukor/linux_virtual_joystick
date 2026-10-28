# Linux Virtual Joystick
This is a simple program that creates a virtual joystick on your computer. I use it to develop software that needs a joystick, without actually having a joystick on-hand. it was inspired by the basic functionality of [vJoy](http://vjoystick.sourceforge.net/site/). 

## Joystick Options

#### Xbox Elite 2 (default)

![Xbox Elite 2 virtual controller with active inputs highlighted in blue](docs/xbox-screenshot.png)

<table>
  <tr>
    <th>SRoC Tab Active5</th>
    <th>SRoC DualGrip</th>
    <th>Linux Virtual Joystick</th>
  </tr>
  <tr>
    <td width="33%"><img src="docs/app-screenshot.png" width="300" alt="SRoC Tab Active5 virtual controller"></td>
    <td width="33%"><img src="docs/dualgrip-screenshot.png" width="300" alt="SRoC DualGrip virtual controller"></td>
    <td width="33%"><img src="docs/original-screenshot.png" width="300" alt="Original Linux Virtual Joystick interface"></td>
  </tr>
</table>

Interact with the joystick by dragging the image's joystick pads or clicking on the image's button. To set persistent inputs, use the controls surrounding the controller image.

## Run prebuilt appimage
Download the latest AppImage from this repository's [Releases page](../../releases).
```
sudo chmod +x linux_virtual_joystick.AppImage
sudo ./linux_virtual_joystick.AppImage
```

## Build from Source

Clone this repository using its **Code** button, then run the following commands
from the cloned directory. If you are viewing a fork, use that fork's clone URL.

```
# apt dependencies
sudo apt install git curl build-essential -y

# install rust/cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# restart terminal so cargo is added to PATH

# build and run
cargo build
sudo -E ~/.cargo/bin/cargo run
```

## Making a Release

Commit your changes first, then run this from the repository:

```bash
./scripts/release.sh
```

The script:

1. Shows the current version and asks for the new version, such as `2.0.1` or `2.1.0-rc.1`.
2. Updates `Cargo.toml` and the app's entry in `Cargo.lock`.
3. Makes a git commit with updated and tags the commit.
5. Pushes the branch and tag.

When gethub gets the tag push it triggers `.github/workflows/release.yml`, which builds an AppImage and publishes `linux_virtual_joystick.AppImage` to a GitHub release with automatically generated release notes. The release appears after the workflow succeeds.
