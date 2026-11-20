# Owned-child terminal-state observation

## Original failed execution

[Compiler set run 44676352150, job 136089002371](https://github.com/glopyglerky/lustr/actions/runs/44676352150/job/136089002371)
failed on PR #33 head `33fffd4a04cc6f6f1c121e97de0282358dae5fb4`, tree
`0e3ce15c3ae3413ff97a450ad491372e0cd46f8d`. The
`profile-native-preflight` unit-test executable reported 30 passed, one failed
and one ignored. At `2026-11-20T04:47:35.818349Z`,
`owned_child::tests::exited_leader_cannot_leave_pipe_readers_blocked` failed
with `leaf survived leader cleanup` at `src/owned_child.rs:267`.

The test had already observed pipe EOF, leader exit code 17 and completion
within its three-second limit. Its next, immediate process-state read found
a non-zombie leaf. The record does not preserve that raw state or distinguish
the capture and streaming iterations. The earlier invocation in the main
unit-test executable passed, but cannot qualify the failed invocation.
The complete original records remain retained with SHA-256
`a18022b3321a4be3beea9b14f229261bbfdf71917062a3c227cc77911bd64f2b`.

## Diagnosis and bounded correction

[Linux `do_exit`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/stable/linux/+/refs/heads/linux-5.15.y/kernel/exit.c)
calls `exit_files` before `exit_notify` sets `EXIT_ZOMBIE`.
[Pipe EOF](https://man7.org/linux/man-pages/man7/pipe.7.html) observes closed
writers. A [group signal](https://man7.org/linux/man-pages/man2/kill.2.html)
and [waiting for the leader](https://man7.org/linux/man-pages/man2/wait.2.html)
do not wait for a grandchild's terminal process state. This ordering permits
the observed failure. It is an explanation inferred from source and the
retained failure, not a recorded state-transition trace of that execution.

The correction changes only `#[cfg(test)]` code. Both original capture and
streaming paths must still reach EOF and leader exit 17 within three seconds.
The test then observes the leaf until that same deadline, accepting only an
absent process or a valid zombie state. Other read, parse and `ps` errors fail.
A timeout includes the PID, mode, elapsed time and last raw state. Observation
does not send signals. A new real live-child control must time out while its
child remains alive; a separate cleanup guard kills and reaps that direct child
even if the control fails.

No production process ownership, signal handling, waiting or compiler path
changes. No compiler, build or test was executed locally for this correction.
Hosted qualification of the new source is still required. The original failed
run remains failed; this note supplies no full-compiler or performance claim.
