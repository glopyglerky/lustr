#!/usr/bin/env python3
"""Construct the bounded artifact/data closure; never copy a compiler or driver."""
import hashlib, json, shutil, struct, sys
from pathlib import Path
SDK_RECEIPT="4e400b263bf9923616dea44ead2d77213703177d3bfaf6becd6a19a4c5a657a7"
STANDARD_RECEIPT="fa35ba66edb2400611692c9c64a7575e0658b9876a03b0be0531aa4ed6658e9d"

def digest(data):
    return hashlib.sha256(data).hexdigest()

def dynamic_libraries(data):
    if data[:7] != b"\x7fELF\x02\x01\x01":
        raise ValueError("artifact tool is not x64 little-endian ELF")
    start=struct.unpack_from("<Q",data,40)[0]
    stride,count=struct.unpack_from("<HH",data,58)
    if stride != 64 or not count:
        raise ValueError("unrepresented ELF section table")
    sections=[struct.unpack_from("<IIQQQQIIQQ",data,start+i*64) for i in range(count)]
    needed=[]
    for section in sections:
        if section[1] != 6:
            continue
        if section[6] >= count or section[9] != 16 or section[5]%16:
            raise ValueError("unrepresented ELF dynamic table")
        strings=sections[section[6]]
        names=data[strings[4]:strings[4]+strings[5]]
        for at in range(section[4],section[4]+section[5],16):
            tag,value=struct.unpack_from("<qQ",data,at)
            if tag==1:
                end=names.index(b"\0",value)
                needed.append(names[value:end].decode("ascii"))
    interpreter=None
    program_start=struct.unpack_from("<Q",data,32)[0]
    program_stride,program_count=struct.unpack_from("<HH",data,54)
    if program_count and program_stride!=56:
        raise ValueError("unrepresented ELF program table")
    for index in range(program_count):
        program=struct.unpack_from("<IIQQQQQQ",data,program_start+index*program_stride)
        if program[0]==3:
            value=data[program[2]:program[2]+program[5]]
            if not value.endswith(b"\0") or b"\0" in value[:-1]:
                raise ValueError("unrepresented ELF interpreter")
            interpreter=value[:-1].decode("ascii")
    return needed,interpreter

def install(repo,sdk,package):
    receipt=repo/"docs/compiler-set/evidence/metadata-sdk-receipt-20261120/receipt.json"
    raw=receipt.read_bytes()
    if digest(raw)!=SDK_RECEIPT:
        raise ValueError("original SDK receipt changed")
    facts=json.loads(raw)
    standard=(receipt.parent/"standard-receipt.json").read_bytes()
    if digest(standard)!=STANDARD_RECEIPT:
        raise ValueError("standard metadata SDK receipt changed")
    tool_path=repo/"upstream/metadata-artifact-tools.json"
    tool_raw=tool_path.read_bytes()
    tools=json.loads(tool_raw)
    if tools["rust_revision"]!=facts["rust_revision"] or tools["target"]!=facts["target"]:
        raise ValueError("artifact-tool source/target changed")
    copied=[]
    def read_record(record):
        relative=Path(record["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("closure path escapes package")
        source=(sdk/relative).resolve(strict=True)
        if not source.is_relative_to(sdk):
            raise ValueError("SDK source escapes selected root")
        data=source.read_bytes()
        if len(data)!=record["size"] or digest(data)!=record["sha256"]:
            raise ValueError("closure bytes differ: "+str(relative))
        return relative,source,data
    def copy_record(record):
        relative,source,data=read_record(record)
        output=package/relative
        output.parent.mkdir(parents=True,exist_ok=True)
        with output.open("xb") as file:
            file.write(data)
        shutil.copymode(source,output)
        copied.append(record)
        return data
    if [(d["name"],d["is_private"]) for d in facts["dependencies"]] != [("core",False),("compiler_builtins",True)]:
        raise ValueError("unrepresented SDK dependency pair")
    for dependency in facts["dependencies"]:
        for field in ("artifact","link_artifact"):
            copy_record(dependency[field])
    # Std-root scalar artifacts reference the actual pinned metadata IDs.
    # Their native bodies have no code from this standard dependency closure.
    for dep in json.loads(standard)["dependencies"]:
        _,_,metadata=read_record(dep["artifact"])
        _,_,archive=read_record(dep["link_artifact"])
        if (metadata[:8]!=b"rust\0\0\0\x0c" or metadata[24:40].hex()!=dep["svh_le_hex"]
                or not archive.startswith(b"!<arch>\n")):
            raise ValueError("standard metadata and link artifacts disagree: "+dep["name"])
    pending=[]
    dependency_edges=[]
    host_libraries=set()
    interpreters=[]
    def enqueue(record,data):
        needed,interpreter=dynamic_libraries(data)
        pending.extend((record["path"],name) for name in needed)
        if interpreter is not None:
            interpreters.append({"source":record["path"],"path":interpreter,"ownership":"host ABI"})
    for tool in tools["tools"]:
        enqueue(tool,copy_record(tool))
    allowed={Path(record["path"]).name:record for record in tools["allowed_sdk_libraries"]}
    included=set()
    while pending:
        source,name=pending.pop()
        if "/" in name or name in {".",".."}:
            raise ValueError("unrepresented dynamic dependency name")
        if name in allowed:
            dependency_edges.append({"source":source,"needed":name,"ownership":"installed SDK artifact tool","path":allowed[name]["path"]})
            if name not in included:
                included.add(name)
                enqueue(allowed[name],copy_record(allowed[name]))
        elif (sdk/"lib"/name).exists() or "rustc" in name or "LLVM" in name:
            raise ValueError("undeclared SDK library dependency: "+name)
        else:
            host_libraries.add(name)
            dependency_edges.append({"source":source,"needed":name,"ownership":"host ABI"})
    if included!=set(allowed):
        raise ValueError("pinned artifact-tool runtime closure differs from declaration")
    directory=package/"lib/lustr-metadata"
    directory.mkdir(parents=True,exist_ok=False)
    (directory/"sdk-receipt.json").write_bytes(raw)
    (directory/"standard-sdk-receipt.json").write_bytes(standard)
    (directory/"artifact-tools.json").write_bytes(tool_raw)
    notices=directory/"licenses"
    notices.mkdir()
    for name,source in [
        ("LICENSE-MIT",repo/"crates/lustr-metadata-encoder/LICENSE-MIT"),
        ("LICENSE-APACHE",repo/"crates/lustr-metadata-encoder/LICENSE-APACHE"),
        ("LICENSE-LLVM",repo/"crates/lustr-native-core/LICENSE-LLVM")]:
        shutil.copyfile(source,notices/name)
    closure={"schema":"lustr-metadata-installed-closure-v1","files":copied,
             "dependency_edges":sorted(dependency_edges,key=lambda edge:(edge["source"],edge["needed"])),
             "host_abi_libraries":sorted(host_libraries),"host_abi_interpreters":interpreters,
             "host_abi_note":"Names and interpreter paths are bound by the pinned copied ELF bytes; system library bytes remain host ABI inputs and are not compiler SDK payloads."}
    (directory/"installed-closure.json").write_text(json.dumps(closure,indent=2)+"\n")
    print(json.dumps({"sdk_receipt_sha256":digest(raw),"standard_sdk_receipt_sha256":digest(standard),
                      "tool_manifest_sha256":digest(tool_raw),"closure":closure},indent=2))

if __name__=="__main__":
    install(*(Path(arg).resolve(strict=True) for arg in sys.argv[1:]))
