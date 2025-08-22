use crate::ir::{Program, Service};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone)]
pub struct ExecOptions {
    pub plan_only: bool,
}

pub fn execute(program: &Program, opts: ExecOptions) -> Result<(), String> {
    for svc in &program.services {
        for _ in 0..svc.replicas {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| format!("system time error: {e}"))?
                .as_millis();

            let cname = format!("{}-{}", svc.name, ts);

            let args = docker_run_args(&cname, svc);

            if opts.plan_only {
                println!("docker {}", shell_join(&args));
            } else {
                let mut cmd = Command::new("docker");
                cmd.args(&args);
                cmd.stdout(Stdio::inherit())
                   .stderr(Stdio::inherit());
                let status = cmd.status()
                    .map_err(|e| format!("failed to spawn docker: {e}"))?;
                if !status.success() {
                    return Err(format!("docker exited with status {}", status));
                }
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
