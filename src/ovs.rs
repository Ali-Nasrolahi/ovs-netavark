use netavark::error::{NetavarkError, NetavarkResult};
use std::{collections::HashMap, process::Command};

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

pub fn add_port(br: &str, port: &str, external_ids: HashMap<&str, &str>) -> NetavarkResult<()> {
    let ext_ids: Vec<String> = external_ids
        .iter()
        .map(|(k, v)| format!("external_ids:{}={}", k, v))
        .collect();

    let mut args: Vec<&str> = vec!["add-port", br, port, "--", "set", "Interface", port];
    for e in &ext_ids {
        args.push(e.as_str());
    }

    run_ovs(&args)?;
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
