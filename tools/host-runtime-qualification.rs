//! Standalone host contracts: actual IPC and process behavior, without a compiler SDK.
//! Run on every admitted Unix host with LUST_BUILD_HASH set at compilation.
#[path = "../src/owned_child.rs"]
mod owned_child;
#[path = "../compiler/native/src/protocol.rs"]
mod protocol;

#[cfg(test)]
mod tests {
    use crate::protocol;
    use std::io::{BufReader, Write};
    #[cfg(target_os = "linux")]
    use std::os::unix::io::AsRawFd;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    struct Temporary(PathBuf);
    impl Temporary {
        fn new(name: &str) -> Self {
            let root = std::path::Path::new("/tmp").join(format!("lustr-host-{name}-{}", std::process::id()));
            std::fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn request_reaches_a_socket_in_a_long_directory() {
        let root = Temporary::new("long-request");
        let directory = root.0.join("a".repeat(90));
        std::fs::create_dir(&directory).unwrap();
        let source = root.0.join("main.rs");
        let socket = directory.join("daemon.sock");
        #[cfg(target_os = "linux")]
        let handle = std::fs::File::open(&directory).unwrap();
        #[cfg(target_os = "linux")]
        let short = PathBuf::from(format!("/proc/self/fd/{}/daemon.sock", handle.as_raw_fd()));
        #[cfg(target_os = "macos")]
        let short = protocol::host_runtime::DaemonSocket::resolve(&socket).unwrap().path().to_owned();
        let listener = UnixListener::bind(&short).unwrap();
        listener.set_nonblocking(true).unwrap();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        protocol::read_request(&mut BufReader::new(&stream)).expect("complete protocol request");
                        stream.write_all(format!("0 0 0 {}\n", protocol::build_id()).as_bytes()).unwrap();
                        return true;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => return false,
                }
            }
        });
        let args = vec!["check".to_owned(), source.to_str().unwrap().to_owned()];
        let Some(protocol::Invocation::Compile { req, .. }) = protocol::parse_args(&args) else {
            panic!("check request");
        };
        let answer = protocol::request(&socket, &req);
        let reached = server.join().unwrap();
        std::fs::remove_file(&short).unwrap();
        assert!(reached, "the request never reached the bound socket");
        assert_eq!(answer.map(|response| response.code), Some(0));
    }

    #[test]
    fn host_abi_matches_native_headers() {
        use protocol::host_runtime as host;
        let root = Temporary::new("abi");
        let source = root.0.join("abi.c");
        let binary = root.0.join("abi");
        std::fs::write(
            &source,
            r#"
#include <stdio.h>
#include <signal.h>
#include <sys/wait.h>
#include <sys/un.h>
#include <poll.h>
#include <dlfcn.h>
int main(void) {
    printf("%zu %zu %zu %d %d %d %d %zu %zu %d %d\n",
        sizeof(sigset_t), sizeof(siginfo_t), _Alignof(siginfo_t),
        SIG_BLOCK, SIG_SETMASK, WNOWAIT, RTLD_NOW | RTLD_LOCAL,
        sizeof(((struct sockaddr_un *)0)->sun_path), sizeof(nfds_t), P_PID, WEXITED);
    return 0;
}
"#,
        )
        .unwrap();
        let compiled = std::process::Command::new("cc").arg(&source).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let output = std::process::Command::new(binary).output().unwrap();
        assert!(output.status.success());
        let values: Vec<usize> = std::str::from_utf8(&output.stdout).unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(values.len(), 11);
        assert_eq!(values[0], std::mem::size_of::<host::SigSet>());
        assert!(values[1] <= std::mem::size_of::<[u64; 16]>());
        assert!(values[2] <= std::mem::align_of::<[u64; 16]>());
        assert_eq!(
            &values[3..],
            &[
                host::SIG_BLOCK as usize,
                host::SIG_SETMASK as usize,
                host::WNOWAIT as usize,
                host::RTLD_NOW_LOCAL as usize,
                host::SOCKET_PATH_CAPACITY,
                std::mem::size_of::<host::PollCount>(),
                1,
                4,
            ]
        );
    }
}
