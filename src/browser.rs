use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

fn isolate_browser(command: &mut Command) -> &mut Command {
    // Browser launchers can signal their entire process group during cleanup.
    // Keep that group separate from the simulator and its terminal.
    command
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
}

/// A browser launched by a sudo-run app must belong to the desktop user.
/// Leave egui's normal browser handling in place for unprivileged launches.
pub fn redirect_sudo_link(ctx: &eframe::egui::Context, url: &str) {
    if !std::fs::metadata("/proc/self")
        .map(|metadata| metadata.uid() == 0)
        .unwrap_or(false)
    {
        return;
    }
    let user = match std::env::var("SUDO_USER") {
        Ok(user) if !user.is_empty() && user != "root" => user,
        _ => return,
    };
    let uid = match std::env::var("SUDO_UID")
        .ok()
        .and_then(|uid| uid.parse::<u32>().ok())
    {
        Some(uid) if uid != 0 => uid,
        _ => return,
    };
    let gid = match std::env::var("SUDO_GID")
        .ok()
        .and_then(|gid| gid.parse::<u32>().ok())
    {
        Some(gid) => gid,
        _ => return,
    };

    ctx.output_mut(|output| output.open_url = None);
    let url = url.to_owned();
    std::thread::spawn(move || {
        let runtime_dir = format!("/run/user/{uid}");
        // Drop privileges without runuser's PAM session setup/teardown, which
        // terminated the simulator when the browser launcher exited.
        let mut command = Command::new("/usr/bin/setpriv");
        command
            .args([
                "--reuid",
                &uid.to_string(),
                "--regid",
                &gid.to_string(),
                "--init-groups",
                "--",
                "xdg-open",
                &url,
            ])
            .env("XDG_RUNTIME_DIR", &runtime_dir)
            .env(
                "DBUS_SESSION_BUS_ADDRESS",
                format!("unix:path={runtime_dir}/bus"),
            );
        match isolate_browser(&mut command).status() {
            Ok(status) if status.success() => {}
            result => eprintln!("Could not open the website as {user}: {result:?}"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn browser_group_cleanup_does_not_kill_the_app() {
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            // Check isolation before signaling so a regression cannot kill
            // the test runner or its terminal.
            "group=$(ps -o pgid= -p $$ | tr -d ' '); \
             [ \"$group\" = \"$$\" ] || exit 99; \
             /bin/kill -KILL -- -$$",
        ]);
        let status = isolate_browser(&mut command).status().unwrap();
        assert_eq!(status.signal(), Some(9));
    }
}
