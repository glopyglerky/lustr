//! Qualify the retained native object seam before composing a whole mixed crate.
#[path = "../compiler/native/src/protocol.rs"]
#[rustfmt::skip]
mod protocol;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
fn checked(c: &mut Command) {
    let o = c.output().unwrap();
    assert!(o.status.success(), "{c:?}: {}", String::from_utf8_lossy(&o.stderr));
}
fn main() {
    let mut a = std::env::args_os().skip(1);
    let work = PathBuf::from(a.next().unwrap());
    let core = PathBuf::from(a.next().unwrap());
    let rustc = PathBuf::from(a.next().unwrap());
    fs::create_dir(&work).unwrap();
    let work = work.canonicalize().unwrap();
    let retained = work.join("actual-compiler-bytes");
    fs::copy(&core, &retained).unwrap();
    fs::set_permissions(&retained, fs::Permissions::from_mode(0o644)).unwrap();
    let socket = work.join("daemon.sock");
    let log = fs::File::create(work.join("daemon.log")).unwrap();
    let mut child = Command::new(&core)
        .arg("daemon")
        .current_dir(&work)
        .env_remove("LUST_LIBRARY")
        .env("LUST_DAEMON_SOCKET", &socket)
        .env("LUST_CACHE_DIR", work.join("cache"))
        .env("LUST_DAEMON_IDLE", "2s")
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !socket.exists() {
        assert!(started.elapsed() < Duration::from_secs(10));
        assert!(child.try_wait().unwrap().is_none(), "daemon exited: {}", fs::read_to_string(work.join("daemon.log")).unwrap());
        std::thread::sleep(Duration::from_millis(10));
    }
    let source = work.join("kernel.rs");
    for (label, operand) in [("first", 17), ("body-edit", 19)] {
        fs::write(&source, format!("fn kernel(x:u64)->u64{{x ^ {operand}}}\nfn main(){{let _=kernel(0);}}\n")).unwrap();
        let args =
            vec!["build".into(), source.to_str().unwrap().into(), "-o".into(), work.join(format!("{label}.o")).to_str().unwrap().into()];
        let Some(protocol::Invocation::Compile { mut req, .. }) = protocol::parse_args(&args) else { panic!("native request") };
        req.session.krate.target_kind = "lustr-mixed-native".into();
        req.session.krate.target_name = "kernel".into();
        req.session.krate.package_id = "lustr-mixed-1111".into();
        req.client.env = None;
        let response = protocol::request_mixed(&socket, &req).expect("complete mixed exchange").expect("same-build native response");
        fs::write(work.join(format!("{label}.stdout")), &response.stdout).unwrap();
        fs::write(work.join(format!("{label}.stderr")), &response.stderr).unwrap();
        assert_eq!(response.code, 0, "{}", response.stderr);
        assert_eq!(response.stdout, "lustr_mixed_1111");
        let app = work.join(format!("{label}-app"));
        let test = work.join(format!("{label}-harness.rs"));
        fs::write(&test,format!("unsafe extern \"C\"{{fn lustr_mixed_1111(x:u64)->u64;}} fn main(){{for x in [0u64,1,42,u64::MAX]{{assert_eq!(unsafe{{lustr_mixed_1111(x)}},x ^ {operand});}}}}\n")).unwrap();
        checked(Command::new(&rustc).arg("--edition=2024").arg(&test).arg("-o").arg(&app).arg(format!("-Clink-arg={}", req.out.display())));
        let copy = work.join(format!("{label}-app-retained-bytes"));
        fs::copy(&app, &copy).unwrap();
        fs::set_permissions(copy, fs::Permissions::from_mode(0o644)).unwrap();
        checked(&mut Command::new(&app));
        println!("{label}: actual native object + Rust C caller runtime0; {}", response.stderr.trim());
    }
    let status = child.wait().unwrap();
    assert!(status.success(), "owned idle daemon: {status}");
    assert!(!Path::new(&socket).exists());
    // The nounwind policy is local to proved scalar units. Exercise ordinary
    // unwind generation through the same newly built compiler as a regression.
    for name in ["unwind_drops", "nested_resume_panic_any"] {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("compiler/native/tests/unwind").join(format!("{name}.rs"));
        let input = work.join(format!("{name}.rs"));
        fs::copy(fixture, &input).unwrap();
        let mut results = Vec::new();
        for engine in ["native", "oracle"] {
            let app = work.join(format!("{name}-{engine}"));
            let mut command = Command::new(if engine == "native" { &core } else { &rustc });
            if engine == "native" {
                command.arg("build");
            } else {
                command.arg("--edition=2024");
            }
            let output = command
                .arg(&input)
                .arg("-o")
                .arg(&app)
                .args(["-C", "panic=unwind"])
                .env_remove("LUST_LIBRARY")
                .env("LUST_CACHE_DIR", work.join("ordinary-cache"))
                .output()
                .unwrap();
            fs::write(work.join(format!("{name}-{engine}-build.stdout")), &output.stdout).unwrap();
            fs::write(work.join(format!("{name}-{engine}-build.stderr")), &output.stderr).unwrap();
            assert!(output.status.success(), "{name}/{engine}: {}", String::from_utf8_lossy(&output.stderr));
            let retained = app.with_extension("retained-bytes");
            fs::copy(&app, &retained).unwrap();
            fs::set_permissions(retained, fs::Permissions::from_mode(0o644)).unwrap();
            let result = Command::new(app).output().unwrap();
            fs::write(work.join(format!("{name}-{engine}-app.stdout")), &result.stdout).unwrap();
            fs::write(work.join(format!("{name}-{engine}-app.stderr")), &result.stderr).unwrap();
            assert!(result.status.success(), "{name}/{engine}: {}", result.status);
            results.push(result.stdout);
        }
        assert_eq!(results[0], results[1], "{name} destruction/recovery mismatch");
        println!("{name}: ordinary native unwind/drop + Rust oracle runtime0");
    }
}
