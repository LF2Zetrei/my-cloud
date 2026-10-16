use crate::{error::Result, models::Container};
use nix::mount::MsFlags;
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::{WaitPidFlag, waitpid};
use nix::unistd::Pid;

fn is_mounted(path: &str) -> bool {
    std::fs::read_to_string("/proc/mounts")
        .map(|m| m.lines().any(|l| l.split_whitespace().nth(1) == Some(path)))
        .unwrap_or(false)
}

pub fn setup_overlay(base: &str) -> Result<()> {
    std::fs::create_dir_all(format!("{}/lower", base))?;
    std::fs::create_dir_all(format!("{}/upper", base))?;
    std::fs::create_dir_all(format!("{}/work", base))?;
    std::fs::create_dir_all(format!("{}/merged", base))?;

    // Un redémarrage du container ne doit pas empiler un second overlay
    let merged = format!("{}/merged", base);
    if is_mounted(&merged) {
        return Ok(());
    }

    nix::mount::mount(
        Some("overlay"),
        format!("{}/merged", base).as_str(),
        Some("overlay"),
        MsFlags::empty(),
        Some(
            format!(
                "lowerdir={}/lower,upperdir={}/upper,workdir={}/work",
                base, base, base
            )
            .as_str(),
        ),
    )?;

    println!("[RUNTIME] OverlayFS monté sur {}", merged);
    Ok(())
}

pub fn setup_cgroup(container_id: &str, pid: u32) -> Result<()> {
    let cgroup_path_memory = format!("/sys/fs/cgroup/memory/my-container-{}", container_id);
    let cgroup_path_cpu = format!("/sys/fs/cgroup/cpu/my-container-{}", container_id);
    let cgroup_path_pids = format!("/sys/fs/cgroup/pids/my-container-{}", container_id);

    std::fs::create_dir_all(cgroup_path_cpu)?;
    std::fs::create_dir_all(cgroup_path_memory)?;
    std::fs::create_dir_all(cgroup_path_pids)?;

    std::fs::write(
        format!(
            "/sys/fs/cgroup/memory/my-container-{}/memory.limit_in_bytes",
            container_id
        ),
        "536870912",
    )?;

    std::fs::write(
        format!(
            "/sys/fs/cgroup/cpu/my-container-{}/cpu.cfs_quota_us",
            container_id
        ),
        "50000",
    )?;
    std::fs::write(
        format!(
            "/sys/fs/cgroup/cpu/my-container-{}/cpu.cfs_period_us",
            container_id
        ),
        "100000",
    )?;

    std::fs::write(
        format!(
            "/sys/fs/cgroup/memory/my-container-{}/cgroup.procs",
            container_id
        ),
        pid.to_string(),
    )?;
    std::fs::write(
        format!(
            "/sys/fs/cgroup/cpu/my-container-{}/cgroup.procs",
            container_id
        ),
        pid.to_string(),
    )?;

    println!("[RUNTIME] Cgroup configuré pour le PID {}", pid);
    Ok(())
}

pub fn launch_container(container: &Container) -> Result<u32> {
    let base = format!("/var/my-cloud/containers/{}", container.id);
    setup_overlay(&base)?;

    let child = std::process::Command::new(&container.image)
        .args(&container.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let pid = child.id();
    setup_cgroup(&container.id, pid)?;
    Ok(pid)
}

/// Vrai si le processus existe et n'est pas un zombie.
/// Un enfant terminé mais jamais attendu reste dans /proc à l'état "Z" :
/// on le récupère ici (waitpid) pour libérer son entrée dans la table des processus.
pub fn is_alive(pid: u32) -> bool {
    let stat = match std::fs::read_to_string(format!("/proc/{}/stat", pid)) {
        Ok(s) => s,
        Err(_) => return false,
    };
    // Format : "pid (comm) state ..." — comm peut contenir des espaces, on coupe après ')'
    let state = stat
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().next());
    if state == Some("Z") {
        let _ = waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG));
        return false;
    }
    true
}

pub fn stop_container(pid: u32) -> Result<()> {
    match kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
        Ok(()) | Err(nix::errno::Errno::ESRCH) => {}
        Err(e) => return Err(e.into()),
    }
    let _ = waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zombie_is_not_alive() {
        let child = std::process::Command::new("/bin/true").spawn().unwrap();
        let pid = child.id();
        // Sans wait(), l'enfant terminé reste zombie : /proc/<pid> existe toujours
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(std::path::Path::new(&format!("/proc/{}", pid)).exists());
        assert!(!is_alive(pid));
    }

    #[test]
    fn running_process_is_alive() {
        let mut child = std::process::Command::new("/bin/sleep").arg("5").spawn().unwrap();
        assert!(is_alive(child.id()));
        stop_container(child.id()).unwrap();
        let _ = child.wait();
    }
}
