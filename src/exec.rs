use std::process::{Command, Stdio};
use crate::ir::{Program, Service};

#[derive(Copy, Clone)]
pub struct ExecOptions {
    pub plan_only: bool,   // if true: print docker commands instead of executing
}

pub fn execute(program: &Program, opts: ExecOptions) -> Result<(), String> {
    for svc in &program.services {
        for i in 0..svc.replicas {
            let cname = format!("{}-{}", svc.name, i + 1);
            let args = docker_run_args(&cname, svc);
            if opts.plan_only {
                println!("docker {}", shell_join(&args));
            } else {
                run_docker(&args)?;
            }
        }
    }
    Ok(())
}

fn docker_run_args(name: &str, svc: &Service) -> Vec<String> {
    let mut args: Vec<String> = vec!["run".into(), "-d".into(), "--name".into(), name.into()];
    for (h, c) in &svc.ports {
        args.push("-p".into());
        args.push(format!("{}:{}", h, c));
    }
    for (k, v) in &svc.env {
        args.push("-e".into());
        args.push(format!("{}={}", k, v));
    }
    for v in &svc.volumes {
        args.push("-v".into());
        args.push(v.clone());
    }
    args.push(svc.image.as_ref().unwrap().clone());
    args
}

fn run_docker(args: &[String]) -> Result<(), String> {
    let mut cmd = Command::new("docker");
    cmd.args(args);
    // inherit stdout/stderr to surface docker errors
    let status = cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit())
        .status().map_err(|e| format!("failed to spawn docker: {e}"))?;
    if !status.success() {
        return Err(format!("docker exited with status {}", status));
    }
    Ok(())
}

// purely for readable --plan output; not for shell execution
fn shell_join(args: &[String]) -> String {
    args.iter().map(|a| {
        if a.chars().all(|c| c.is_alphanumeric() || "_-./:=@".contains(c)) {
            a.clone()
        } else {
            let escaped = a.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
        }
    }).collect::<Vec<_>>().join(" ")
}
