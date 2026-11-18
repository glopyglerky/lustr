#!/usr/bin/env python3
"""Capture selected benchmark host facts without usernames, hostnames or secrets.

Unsupported or unreadable facts remain explicitly unknown. This does not pin
cores, change a governor, flush caches, or measure process CPU/RSS consumption.
"""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import platform

SCHEMA = "lustr-benchmark-host-v1"


def observation(value, source, reason=None):
    result = {"value": value, "source": source}
    if value is None:
        result["unavailable_reason"] = reason or "not reported on this host"
    return result


def read(path):
    try:
        return path.read_text(encoding="utf-8").strip()
    except (OSError, UnicodeError):
        return None


def field(text, name):
    for line in (text or "").splitlines():
        key, separator, value = line.partition(":")
        if separator and key.strip() == name:
            return value.strip()
    return None


def collect(proc=Path("/proc"), sysfs=Path("/sys"), environ=None):
    environ = os.environ if environ is None else environ
    cpu = read(proc / "cpuinfo")
    model = field(cpu, "model name") or field(cpu, "Processor")
    memory = field(read(proc / "meminfo"), "MemTotal")
    memory_bytes = None
    if memory:
        parts = memory.split()
        if len(parts) == 2 and parts[0].isdigit() and parts[1] == "kB":
            memory_bytes = int(parts[0]) * 1024
    try:
        affinity = sorted(os.sched_getaffinity(0))
    except (AttributeError, OSError):
        affinity = None
    governors = sorted({value for path in (sysfs / "devices/system/cpu/cpufreq").glob("policy*/scaling_governor")
                        if (value := read(path))})
    runner_keys = ("RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion",
                   "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_SHA")
    return {
        "schema": SCHEMA,
        "recorded_utc": datetime.now(timezone.utc).isoformat(),
        "platform": {"system": platform.system(), "release": platform.release(), "machine": platform.machine()},
        "cpu_model": observation(model, "/proc/cpuinfo: model name or Processor"),
        "logical_cpus": observation(os.cpu_count(), "os.cpu_count"),
        "process_affinity": observation(affinity, "os.sched_getaffinity(0)"),
        "physical_memory_bytes": observation(memory_bytes, "/proc/meminfo: MemTotal"),
        "frequency_governors": observation(governors or None, "/sys/devices/system/cpu/cpufreq/policy*/scaling_governor"),
        "cgroup_v2_cpu_max": observation(read(sysfs / "fs/cgroup/cpu.max"), "/sys/fs/cgroup/cpu.max"),
        "cgroup_v2_memory_max": observation(read(sysfs / "fs/cgroup/memory.max"), "/sys/fs/cgroup/memory.max"),
        "cgroup_v2_cpuset_effective": observation(read(sysfs / "fs/cgroup/cpuset.cpus.effective"), "/sys/fs/cgroup/cpuset.cpus.effective"),
        "runner": {key: observation(environ.get(key), f"environment:{key}") for key in runner_keys},
        "limits": ["Container-visible host facts; physical memory is not a process budget.",
                   "Cgroup files describe the visible cgroup root; nested constraints may differ.",
                   "Affinity is observed, not pinned. CPU frequency and host load can change.",
                   "No process CPU time, RSS, energy or isolated application runtime is measured."],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="new JSON file, never overwrite earlier evidence")
    args = parser.parse_args()
    record = collect()
    with args.output.open("x", encoding="utf-8") as output:
        json.dump(record, output, indent=2, allow_nan=False)
        output.write("\n")
    print(f"Recorded selected host facts in {args.output}")


if __name__ == "__main__":
    main()
