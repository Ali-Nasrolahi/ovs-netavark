use netavark::error::{NetavarkError, NetavarkResult};
use std::process::Command;

fn run_ovs(args: &[&str]) -> NetavarkResult<std::process::Output> {
    let out = Command::new("ovs-vsctl")
        .args(args)
        .output()
        .map_err(|e| NetavarkError::Message(format!("failed to execute ovs-vsctl: {e}")))?;

    if !out.status.success() {
        return Err(NetavarkError::Message(format!(
            "ovs-vsctl {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(out)
}

pub fn add_port(br: &str, port: &str) -> NetavarkResult<()> {
    run_ovs(&["add-port", br, port])?;
    Ok(())
}

pub fn del_port(br: &str, port: &str) -> NetavarkResult<()> {
    run_ovs(&["--if-exists", "del-port", br, port])?;
    Ok(())
}

pub fn ensure_br_exists(br: &str) -> NetavarkResult<()> {
    run_ovs(&["br-exists", br])?;
    Ok(())
}
