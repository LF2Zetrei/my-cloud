use common::container::launch_container;
use common::error::Result;
use common::models::ContainerBuilder;

fn main() -> Result<()> {
    let container = ContainerBuilder::new()
        .name("my-container")
        .image("/bin/sh")
        .build()?;

    let pid = launch_container(&container)?;
    println!("[RUNTIME] Container lancé avec PID {}", pid);
    Ok(())
}
