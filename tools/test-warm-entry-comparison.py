#!/usr/bin/env python3
"""Negative controls for paired warm-entry measurements; no compiler speed claim."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("warm_entry", Path(__file__).with_name("run-warm-entry-comparison.py"))
B = importlib.util.module_from_spec(spec)
spec.loader.exec_module(B)


def synthetic_samples(candidate=0.001):
    return [{"sample":i, "lane":lane, "phase":phase, "seconds":candidate if lane == "entry" else 0.01, "commands":[i]}
            for i in range(12) for lane in B.LANES for phase in B.PHASES]


class ComparisonControls(unittest.TestCase):
    def test_baseline_gap_is_red(self):
        samples = synthetic_samples(0.03)
        result = B.parity(B.summaries(samples,12))
        self.assertFalse(result["meets_target"])
        self.assertEqual(result["modes"]["no-op-check"]["slowdown_median"],3.0)

    def test_faster_candidate_can_pass_the_exact_target(self):
        self.assertTrue(B.parity(B.summaries(synthetic_samples(),12))["meets_target"])

    def test_missing_sample_rejected(self):
        with self.assertRaises(ValueError): B.summaries(synthetic_samples()[:-1],12)

    def test_duplicate_sample_rejected(self):
        rows=synthetic_samples(); rows[-1]=copy.deepcopy(rows[0])
        with self.assertRaises(ValueError): B.summaries(rows,12)

    def test_nonfinite_sample_rejected(self):
        rows=synthetic_samples(); rows[0]["seconds"]=float("nan")
        with self.assertRaises(ValueError): B.summaries(rows,12)

    def test_order_counterbalances_all_four_lanes(self):
        schedule=B.orders(24)
        for position in range(4):
            self.assertEqual({lane:sum(row[position]==lane for row in schedule) for lane in B.LANES},dict.fromkeys(B.LANES,6))

    def test_a_single_failing_warm_mode_fails_the_target(self):
        rows=synthetic_samples()
        for row in rows:
            if row["lane"]=="entry" and row["phase"]=="no-op-build": row["seconds"]=0.02
        result=B.parity(B.summaries(rows,12))
        self.assertTrue(result["modes"]["no-op-check"]["meets_target"])
        self.assertFalse(result["modes"]["no-op-build"]["meets_target"])
        self.assertFalse(result["meets_target"])

    def test_incomplete_measurement_cannot_replay(self):
        with tempfile.TemporaryDirectory() as directory:
            output=Path(directory)
            B.S.write_json(output/"report.json",{"schema":B.SCHEMA,"status":"failed"})
            with self.assertRaises(ValueError): B.verify(output)


def synthetic_campaign(output):
    """Execute real campaign orchestration with labeled synthetic child results."""
    args=SimpleNamespace(output=output,timeout=10,pairs=12,built=output/"synthetic-tools",pristine_lust=output/"synthetic-pristine/lust",
                         pristine_source=output/"synthetic-pristine-source",repository=output/"synthetic-candidate-source")
    campaign=B.Campaign(args)
    campaign.fixture.mkdir(); (campaign.fixture/"src").mkdir()
    (campaign.fixture/"src/slice_main.rs").write_text("fn main() {}\n")
    (campaign.fixture/"Cargo.toml").write_text("# synthetic fixture\n")
    (campaign.fixture/"Cargo.lock").write_text("# synthetic lock\n")
    (output/"rejected.rs").write_text(B.S.REJECTION)
    recorder=campaign.recorder
    def run(argv,env,cwd,lane,phase,expected=0):
        argv=list(map(str,argv)); data=error=b""
        if phase=="gate-reject": error=b"error[E0308]: synthetic expected mismatch\n"
        elif phase in ("gate-retained-check","gate-retained-build"):
            error=(b"lust-stats: mode=reuse fns=4 rechecked=0 invalidated=0 c_bytes=0 dirty_units=0 units_compiled=0 reemitted=0 discovered=0\n"
                   b"lust-visits: path=reuse why=- checked=0\n")
        elif len(argv)==1: data=b"synthetic runtime result\n"
        if len(argv)>1 and argv[1] in ("check","build") and phase!="gate-reject":
            binary=None
            if lane=="cargo":
                target=Path(env["CARGO_TARGET_DIR"])
                binary=target/B.S.TARGET/"debug/slice" if argv[1]=="build" else None
                data=(json.dumps({"reason":"compiler-artifact","target":{"name":"slice","kind":["bin"]},
                      "profile":{"opt_level":"0","debuginfo":0,"debug_assertions":True,"overflow_checks":True},
                      "fresh":phase not in ("cold-check","cold-build","gate-build"),"executable":str(binary) if binary else None})+"\n"+
                      json.dumps({"reason":"build-finished","success":True})+"\n").encode()
            elif "-o" in argv: binary=Path(argv[argv.index("-o")+1])
            if binary:
                binary.parent.mkdir(parents=True,exist_ok=True); binary.write_bytes(b"synthetic nonexecutable bytes\n")
            if "LUSTR_TRACE_PATH" in env:
                Path(env["LUSTR_TRACE_PATH"]).write_text(json.dumps({"engine":"lust","entry":"upstream-process" if "LUST_NO_DAEMON" in env else "daemon"})+"\n")
            if "LUSTR_SELECTION_DIAGNOSTICS" in env:
                with Path(env["LUSTR_SELECTION_DIAGNOSTICS"]).open("a") as stream:
                    stream.write(json.dumps({"selection_cache":"hit" if phase=="gate-selection-hit" else "miss"})+"\n")
        row={"kind":"command","id":len(recorder.records),"argv":argv,"environment":env,"cwd":str(cwd),"lane":lane,"phase":phase,
             "expected_exit":expected,"exit":expected,"signal":None,"timed_out":False,"output_limit":False,"seconds":.01}
        for name,value in (("stdout",data),("stderr",error)):
            path=output/"commands"/f"{row['id']:05}.{name}"; path.write_bytes(value)
            row[name]={"file":str(path.relative_to(output)),"bytes":len(value),"sha256":B.S.digest(path)}
        recorder.event(row); return row,data,error
    class Daemon:
        def __init__(self,recorder,executable,env,cwd,lane,scope):
            self.row={"kind":"daemon-start","lane":lane,"pid":len(recorder.records)+100,"scope":scope,"executable":str(executable),
                      "environment":env,"cwd":str(cwd),"socket":env["LUST_DAEMON_SOCKET"]}
        def __enter__(self): recorder.event(self.row); return self
        def alive(self): pass
        def __exit__(self,*args): recorder.event({"kind":"daemon-stop",**{k:self.row[k] for k in ("lane","pid","scope")},"exit":0,"error":None,"streams":[]})
    tools={key:str(value) for key,value in campaign.tools.items()}
    identity={"revision":"c"*40,"tree":"d"*40}
    receipts=output/"build-receipts"; receipts.mkdir()
    (receipts/"lustr-selection-tools.txt").write_text("synthetic trusted-tool receipt\n")
    receipt={"status":"completed","baseline":{"revision":B.BASELINE},"candidate":identity,"baseline_binary":{"path":tools["baseline"],"sha256":"a"*64},
             "shared_package_sha256":{str(Path(tools["entry"]).parent.parent/"lib/lustr-selection-tools.txt"):B.S.digest(receipts/"lustr-selection-tools.txt")}}
    B.S.write_json(receipts/"receipt.json",receipt)
    campaign.report.update(tool_paths=tools,compiler_and_runner_sha256={tools["baseline"]:"a"*64},fixture_files=B.S.inventory(campaign.fixture),
        candidate_identity=identity,baseline_composition=receipt,build_receipts={name:{"file":"build-receipts/"+name,"sha256":B.S.digest(receipts/name)} for name in ("receipt.json","lustr-selection-tools.txt")},
        rustc_version=f"commit-hash: {B.S.RUST}\n")
    recorder.event({"kind":"identities",**{key:campaign.report[key] for key in
        ("tool_paths","compiler_and_runner_sha256","fixture_files","candidate_identity","baseline_composition","build_receipts","rustc_version")}})
    sockets=output/"sockets"; sockets.mkdir()
    with recorder.raw,patch.object(recorder,"run",side_effect=run),patch.object(B.S,"Daemon",Daemon):
        campaign.qualification(sockets,"initial")
        campaign.report["orders"]=B.orders(12)
        for sample,order in enumerate(campaign.report["orders"]):
            for operation in ("check","build"): campaign.measure_group(sample,order,operation,sockets)
        campaign.qualification(sockets,"final")
    campaign.report.update(status="completed",samples=campaign.samples,summaries=B.summaries(campaign.samples,12),raw_sha256=B.S.digest(output/"raw.jsonl"))
    campaign.report["warm_target"]=B.parity(campaign.report["summaries"])
    campaign.report["priming_summary"]=B.priming_summary(recorder.records,12)
    B.S.write_json(output/"report.json",campaign.report); (output/"README.md").write_text(B.render(campaign.report))


class ReplayControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary=tempfile.TemporaryDirectory(); cls.original=Path(cls.temporary.name)/"synthetic-original"; cls.original.mkdir()
        synthetic_campaign(cls.original)
    @classmethod
    def tearDownClass(cls): cls.temporary.cleanup()
    def corruption(self,mutate):
        output=Path(self.temporary.name)/self.id().split(".")[-1]; shutil.copytree(self.original,output)
        rows=[json.loads(line) for line in (output/"raw.jsonl").read_text().splitlines()]
        mutate(rows)
        (output/"raw.jsonl").write_text("".join(json.dumps(row)+"\n" for row in rows))
        report=json.loads((output/"report.json").read_text()); report["raw_sha256"]=B.S.digest(output/"raw.jsonl")
        B.S.write_json(output/"report.json",report)
        return output
    def test_complete_orchestration_replays(self): self.assertEqual(len(B.verify(self.original)["samples"]),192)
    def test_changed_timed_executable_is_rejected(self):
        def mutate(rows): next(r for r in rows if r["kind"]=="command" and r["phase"]=="no-op-check")["argv"][0]="/unrelated/compiler"
        with self.assertRaises(ValueError): B.verify(self.corruption(mutate))
    def test_missing_retained_build_proof_is_rejected(self):
        def mutate(rows): next(r for r in rows if r["kind"]=="retained-route" and r["operation"]=="build")["kind"]="unrelated"
        with self.assertRaises(ValueError): B.verify(self.corruption(mutate))
    def test_missing_selection_reuse_proof_is_rejected(self):
        def mutate(rows): next(r for r in rows if r["kind"]=="selection-cache")["kind"]="unrelated"
        with self.assertRaises(ValueError): B.verify(self.corruption(mutate))
    def test_freshly_restarted_warm_command_is_rejected(self):
        def mutate(rows): next(r for r in rows if r["kind"]=="command" and r["phase"]=="no-op-check")["environment"]["LUST_NO_DAEMON"]="1"
        with self.assertRaises(ValueError): B.verify(self.corruption(mutate))
    def test_instrumented_timed_command_is_rejected(self):
        def mutate(rows): next(r for r in rows if r["kind"]=="command" and r["phase"]=="no-op-check")["environment"]["LUST_STATS"]="1"
        with self.assertRaises(ValueError): B.verify(self.corruption(mutate))
    def test_omitted_runtime_identity_is_rejected(self):
        def mutate(rows): next(r for r in rows if r["kind"]=="runtime-identity")["kind"]="unrelated"
        with self.assertRaises(ValueError): B.verify(self.corruption(mutate))
    def test_misbound_group_end_is_rejected(self):
        def mutate(rows): next(r for r in rows if r["kind"]=="group-end")["sample"]+=1
        with self.assertRaises(ValueError): B.verify(self.corruption(mutate))


if __name__ == "__main__":
    unittest.main()
