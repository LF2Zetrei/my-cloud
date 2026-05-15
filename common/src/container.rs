use crate::{error::Result, models::Container};
use nix::mount::MsFlags;

pub fn setup_overlay(base: &str) -> Result<()> {
    std::fs::create_dir_all(format!("{}/lower", base))?;
    std::fs::create_dir_all(format!("{}/upper", base))?;
    std::fs::create_dir_all(format!("{}/work", base))?;
    std::fs::create_dir_all(format!("{}/merged", base))?;

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

    println!("[RUNTIME] OverlayFS monté sur /tmp/container/merged");
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
