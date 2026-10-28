//! Deliver the SRoC side panel's keyboard keys to the GCS while the simulator
//! has focus. These are keyboard events, not extra SDL joystick buttons.
use anyhow::{anyhow, Result};
use std::time::{Duration, Instant};
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        Atom, AtomEnum, ConnectionExt, EventMask, KeyButMask, KeyPressEvent, Window,
        KEY_PRESS_EVENT, KEY_RELEASE_EVENT,
    },
    rust_connection::RustConnection,
    CURRENT_TIME, NONE,
};

pub struct FunctionKeys {
    connection: RustConnection,
    root: Window,
    clients_atom: Atom,
    active_atom: Atom,
    name_atom: Atom,
    target: Option<(Window, String)>,
    keycodes: [u8; 10],
    pressed: [bool; 10],
    pending_presses: [bool; 10],
    active_window: Option<Window>,
    next_scan: Instant,
    resume_at: Instant,
}

impl FunctionKeys {
    pub fn new() -> Result<Self> {
        let (connection, screen) = x11rb::connect(None)?;
        let setup = connection.setup();
        let root = setup.roots[screen].root;
        let first = setup.min_keycode;
        let count = setup.max_keycode - first + 1;
        let mapping = connection.get_keyboard_mapping(first, count)?.reply()?;
        let mut keycodes = [0; 10];
        for (index, symbols) in mapping
            .keysyms
            .chunks(mapping.keysyms_per_keycode as usize)
            .enumerate()
        {
            for (function, keycode) in keycodes.iter_mut().enumerate() {
                if symbols.contains(&(0xffbe + function as u32)) {
                    // XK_F1 through XK_F10
                    *keycode = first + index as u8;
                }
            }
        }
        if keycodes.contains(&0) {
            return Err(anyhow!("F1–F10 are missing from the keyboard layout"));
        }
        let atom = |name: &[u8]| -> Result<Atom> {
            Ok(connection.intern_atom(false, name)?.reply()?.atom)
        };
        let clients_atom = atom(b"_NET_CLIENT_LIST")?;
        let active_atom = atom(b"_NET_ACTIVE_WINDOW")?;
        let name_atom = atom(b"_NET_WM_NAME")?;
        Ok(Self {
            connection,
            root,
            clients_atom,
            active_atom,
            name_atom,
            target: None,
            keycodes,
            pressed: [false; 10],
            pending_presses: [false; 10],
            active_window: None,
            next_scan: Instant::now(),
            resume_at: Instant::now(),
        })
    }

    fn windows(&self, atom: Atom) -> Result<Vec<Window>> {
        let reply = self
            .connection
            .get_property(false, self.root, atom, AtomEnum::WINDOW, 0, 1024)?
            .reply()?;
        Ok(reply
            .value32()
            .map(|values| values.collect())
            .unwrap_or_default())
    }

    fn find_gcs(&self, active: Option<Window>) -> Result<Option<(Window, String)>> {
        let mut windows = self.windows(self.clients_atom)?;
        // Prefer the GCS the user was interacting with if several are open.
        windows.sort_by_key(|window| {
            if Some(*window) == active {
                0
            } else if self.target.as_ref().map(|target| target.0) == Some(*window) {
                1
            } else {
                2
            }
        });
        for window in windows {
            let class = self
                .connection
                .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256)?
                .reply();
            let Ok(class) = class else {
                continue;
            };
            let class = String::from_utf8_lossy(&class.value).to_lowercase();
            if !class
                .split('\0')
                .any(|class| class == "pdw-gcs" || class == "qgroundcontrol")
            {
                continue;
            }
            let name = self
                .connection
                .get_property(false, window, self.name_atom, AtomEnum::ANY, 0, 256)?
                .reply();
            let Ok(name) = name else {
                continue;
            };
            let name = String::from_utf8_lossy(&name.value).to_string();
            return Ok(Some((
                window,
                if name.is_empty() { "GCS".into() } else { name },
            )));
        }
        Ok(None)
    }

    fn send(&self, window: Window, index: usize, pressed: bool) -> Result<()> {
        let event = KeyPressEvent {
            response_type: if pressed {
                KEY_PRESS_EVENT
            } else {
                KEY_RELEASE_EVENT
            },
            detail: self.keycodes[index],
            sequence: 0,
            time: CURRENT_TIME,
            root: self.root,
            event: window,
            child: NONE,
            root_x: 0,
            root_y: 0,
            event_x: 0,
            event_y: 0,
            state: KeyButMask::default(),
            same_screen: true,
        };
        self.connection
            .send_event(
                false,
                window,
                if pressed {
                    EventMask::KEY_PRESS
                } else {
                    EventMask::KEY_RELEASE
                },
                event,
            )?
            .check()?;
        Ok(())
    }

    fn release(&mut self) {
        if let Some((window, _)) = &self.target {
            for (index, pressed) in self.pressed.iter().enumerate() {
                if *pressed {
                    let _ = self.send(*window, index, false);
                }
            }
        }
        self.pressed.fill(false);
        let _ = self.connection.flush();
    }

    pub fn sync(&mut self, desired: &[bool; 10]) -> Result<()> {
        let result = self.sync_inner(desired);
        if result.is_err() {
            // A GCS window can disappear between discovery and delivery.
            // Clear the stale target and discover its replacement on the next scan.
            self.release();
            self.target = None;
            self.pending_presses.fill(false);
            self.next_scan = Instant::now() + Duration::from_millis(200);
        }
        result
    }

    fn sync_inner(&mut self, desired: &[bool; 10]) -> Result<()> {
        if Instant::now() >= self.next_scan {
            self.next_scan = Instant::now() + Duration::from_millis(200);
            let active = self.windows(self.active_atom)?.first().copied();
            let target = self.find_gcs(active)?;
            if target.as_ref().map(|target| target.0) != self.target.as_ref().map(|target| target.0)
                || active != self.active_window
            {
                // The GCS releases its side-panel keys on focus loss. Replay
                // held checkboxes after focus changes so its state stays in sync.
                self.release();
                self.target = target;
                self.active_window = active;
                // Let Qt finish processing WindowDeactivate before replaying
                // held keys, otherwise that event can clear the new presses.
                self.resume_at = Instant::now() + Duration::from_millis(250);
            }
        }
        if Instant::now() < self.resume_at {
            if self.target.is_some() {
                for (index, pressed) in desired.iter().enumerate() {
                    self.pending_presses[index] |= *pressed;
                }
            }
            return Ok(());
        }
        if let Some((window, _)) = &self.target {
            for (index, pressed) in desired.iter().enumerate() {
                // Preserve a quick tap that began and ended while Qt was
                // settling the focus change, rather than losing that input.
                if self.pending_presses[index] && !*pressed && !self.pressed[index] {
                    self.send(*window, index, true)?;
                    self.send(*window, index, false)?;
                }
                if *pressed != self.pressed[index] {
                    self.send(*window, index, *pressed)?;
                    self.pressed[index] = *pressed;
                }
                self.pending_presses[index] = false;
            }
            self.connection.flush()?;
        }
        Ok(())
    }
}

impl Drop for FunctionKeys {
    fn drop(&mut self) {
        self.release();
    }
}
