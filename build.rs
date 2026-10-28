use std::{env, path::Path, process::Command};

fn repository_url(remote: &str) -> Option<String> {
    let remote = remote.trim();
    if remote.chars().any(char::is_whitespace) {
        return None;
    }
    let (scheme, authority, path) = if let Some((scheme, rest)) = remote.split_once("://") {
        if !matches!(scheme, "https" | "http" | "ssh" | "git") {
            return None;
        }
        let (authority, path) = rest.split_once('/')?;
        let authority = authority.rsplit('@').next()?;
        if matches!(scheme, "ssh" | "git") {
            ("https", authority.split(':').next()?, path)
        } else {
            (scheme, authority, path)
        }
    } else {
        let (authority, path) = remote.split_once(':')?;
        ("https", authority.rsplit('@').next()?, path)
    };
    if authority.is_empty() || authority.contains(['/', '\\']) {
        return None;
    }
    let path = path.split(['?', '#']).next()?.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|part| {
            part.is_empty()
                || matches!(*part, "." | "..")
                || !part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || "-_.".contains(ch))
        })
    {
        return None;
    }
    Some(format!("{scheme}://{authority}/{path}"))
}

fn git(directory: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=VIRTUAL_JOYSTICK_REPOSITORY_URL");
    let directory = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory");
    let directory = Path::new(&directory);
    // Watch the actual Git config, including linked worktrees, so changing
    // origin causes cargo run/build to refresh the embedded Website URL.
    for file in ["config", "config.worktree"] {
        if let Some(path) = git(directory, &["rev-parse", "--git-path", file]) {
            let path = directory.join(path);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    let remote = env::var("VIRTUAL_JOYSTICK_REPOSITORY_URL").ok()
        .or_else(|| git(directory, &["remote", "get-url", "origin"]))
        .expect("Cannot determine the repository URL. Set VIRTUAL_JOYSTICK_REPOSITORY_URL when building without a Git origin.");
    let repository = repository_url(&remote)
        .expect("The repository URL must identify an HTTP(S) or SSH host/owner/repository.");
    println!("cargo:rustc-env=VIRTUAL_JOYSTICK_RELEASES_URL={repository}/releases");
}

#[cfg(test)]
mod tests {
    use super::repository_url;

    #[test]
    fn normalizes_github_remotes() {
        for remote in [
            "https://github.com/example/fork.git",
            "git@github.com:example/fork.git",
            "ssh://git@github.com/example/fork.git",
            "ssh://git@github.com:22/example/fork.git",
            "git://github.com/example/fork.git",
            "https://github.com/example/fork/",
        ] {
            assert_eq!(
                repository_url(remote).as_deref(),
                Some("https://github.com/example/fork"),
                "{remote}"
            );
        }
    }

    #[test]
    fn keeps_enterprise_server_and_strips_credentials() {
        assert_eq!(
            repository_url("https://user:secret@git.example.com:8443/team/fork.git?token=secret")
                .as_deref(),
            Some("https://git.example.com:8443/team/fork")
        );
    }

    #[test]
    fn rejects_invalid_remotes() {
        for remote in [
            "",
            "/tmp/repo",
            "file:///tmp/repo",
            "https://github.com/owner",
            "https://github.com/../repo",
            "https://github.com/owner/repo\ninjected",
        ] {
            assert_eq!(repository_url(remote), None, "{remote}");
        }
    }
}
