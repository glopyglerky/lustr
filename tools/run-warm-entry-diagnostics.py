#!/usr/bin/env python3
"""Same-host residual diagnostics after the unchanged four-lane campaign.

These groups do not alter the primary measurements or its performance target.
The selector helper times full::select, excluding parsing and process startup.
Its executable identity differs from the public entry; components are not an
additive decomposition of independently measured whole-command durations.
"""
import argparse
from contextlib import ExitStack
import importlib.util
import itertools
import json
from pathlib import Path
import random
import shutil
import statistics
import tempfile
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('warm_comparison', HERE / 'run-warm-entry-comparison.py')
B = importlib.util.module_from_spec(spec)
spec.loader.exec_module(B)
S, require = B.S, B.require
SCHEMA = 'lustr-warm-residual-diagnostics-v1'
LANES = ('entry', 'integrated', 'pristine')
OPT_OUT = '__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT'


def orders(pairs):
    require(pairs >= 12 and pairs % 6 == 0, 'diagnostic pairs must be a multiple of six')
    rng, result = random.Random(B.SEED + 1), []
    while len(result) < pairs:
        block = list(itertools.permutations(LANES))
        rng.shuffle(block)
        result.extend(list(order) for order in block)
    return result


def selector_result(data, fixture):
    value = S.STATS.json_load(data)
    require(value.get('phase') == 'selection' and value.get('success') is True and value.get('target') == 'slice'
            and value.get('manifest') == str(fixture / 'Cargo.toml') and type(value.get('elapsed_ns')) is int
            and value['elapsed_ns'] > 0, 'selector did not return the exact successful target')
    return value['elapsed_ns'] / 1e9


def summarize(rows, pairs):
    native = {(r['operation'], r['sample'], r['lane']):r['seconds'] for r in rows if r['kind'] == 'diagnostic-timing'}
    component = {(r['sample'], r['lane']):r['component_seconds'] for r in rows if r['kind'] == 'selector-timing'}
    require(len(native) == pairs * 6 == sum(r['kind'] == 'diagnostic-timing' for r in rows), 'missing or repeated native diagnostic pair')
    require(len(component) == pairs * 2 == sum(r['kind'] == 'selector-timing' for r in rows), 'missing or repeated selector pair')
    require(all(S.STATS.positive(value) for value in [*native.values(), *component.values()]), 'invalid diagnostic duration')
    result = []
    for operation in ('check', 'build'):
        for reference, candidate in (('integrated', 'entry'), ('pristine', 'integrated'), ('pristine', 'entry')):
            result.append({'operation':operation, 'reference':reference, 'candidate':candidate,
                           'analysis':S.STATS.summarize([(native[operation,i,reference], native[operation,i,candidate]) for i in range(pairs)], 10000, B.SEED)})
    return {'native':result, 'selector':S.STATS.summarize([(component[i,'fresh'], component[i,'cached']) for i in range(pairs)], 10000, B.SEED)}


class Diagnostic(S.Campaign):
    def __init__(self, args, primary, selector_receipt):
        parent = SimpleNamespace(output=args.output, timeout=180, pairs=primary['pairs'],
                                 built=Path(primary['tool_paths']['entry']).parent.parent.parent,
                                 pristine_lust=Path(primary['tool_paths']['pristine']),
                                 pristine_source=Path(primary['pristine_source']), repository=Path(primary['candidate_source']))
        super().__init__(parent)
        self.fixture = args.campaign / 'fixture'
        self.selector, self.primary = args.selector, primary
        self.expected_output = (args.campaign / 'runtime-oracle.stdout').read_bytes()
        (self.output / 'runtime-oracle.stdout').write_bytes(self.expected_output)
        self.report = {'schema':SCHEMA, 'status':'failed', 'pairs':primary['pairs'], 'source':primary['candidate_identity'],
                       'scope':__doc__, 'source_campaign_sha256':S.digest(args.campaign / 'report.json'),
                       'selector_receipt_sha256':S.digest(args.selector_receipt), 'selector':str(self.selector),
                       'tool_paths':{lane:str(self.tools[lane]) for lane in LANES}, 'fixture':str(self.fixture),
                       'fixture_files':primary['fixture_files'], 'runtime_oracle_sha256':S.digest(self.output / 'runtime-oracle.stdout'),
                       'selector_binary_sha256':selector_receipt['binary_sha256'],
                       'environment_note':f'All native lanes explicitly set {OPT_OUT}=1, matching the public parser injection. Lane state paths are separate.',
                       'timing_note':'Paired retained requests on held native daemons after three primes, with all six lane orders balanced in seeded blocks. No trace/stats in measured commands. Separate selector helper pairs follow.',
                       'native_timings':[], 'selector_timings':[]}

    def environment(self, state, socket, cold=False):
        return dict(super().environment(state, socket, cold), **{OPT_OUT:'1'})

    def daemon_tool(self, lane):
        return self.tools['pristine'] if lane == 'pristine' else self.tools['integrated']

    def archive(self, command, suffix, data):
        path = self.output / 'commands' / f"{command['id']:05}.{suffix}"
        path.write_bytes(data)
        return {'file':str(path.relative_to(self.output)), 'sha256':S.digest(path), 'bytes':len(data)}

    def qualification(self, sockets, stage):
        for operation in ('check', 'build'):
            for lane in LANES:
                state = self.output / 'work' / stage / operation / lane
                env = dict(self.environment(state, sockets / (lane + '.sock')), LUST_STATS='1')
                selection, entry = state / 'selection.jsonl', state / 'entry.jsonl'
                if lane == 'entry':
                    env.update(LUSTR_SELECTION_DIAGNOSTICS=str(selection), LUSTR_TRACE_PATH=str(entry))
                with S.Daemon(self.recorder, self.daemon_tool(lane), env, self.fixture, lane, stage + '-' + operation + '-' + lane) as daemon:
                    for _ in range(3): self.compile(lane, operation, state, env, 'diagnostic-prime')
                    before = (selection.read_bytes(), entry.read_bytes()) if lane == 'entry' else None
                    command, binary = self.compile(lane, operation, state, env, 'diagnostic-retained')
                    stderr = (self.output / command['stderr']['file']).read_bytes()
                    self.recorder.event({'kind':'retained-proof', 'stage':stage, 'lane':lane, 'operation':operation,
                                         'command':command['id'], **B.retained_proof(stderr, operation)})
                    if lane == 'entry':
                        after = (selection.read_bytes(), entry.read_bytes())
                        require(all(new.startswith(old) for old,new in zip(before,after)), 'diagnostic traces were replaced')
                        cache, route = (new[len(old):] for old,new in zip(before,after))
                        require([json.loads(line) for line in cache.splitlines()] == [{'selection_cache':'hit'}], 'diagnostic public entry did not reuse selection')
                        require([json.loads(line) for line in route.splitlines()] == [{'engine':'lust','entry':'daemon'}], 'diagnostic public entry escaped its daemon')
                        self.recorder.event({'kind':'public-proof', 'command':command['id'], 'selection':self.archive(command,'selection.trace',cache),
                                             'entry':self.archive(command,'entry.trace',route)})
                    if binary is not None: self.runtime(lane, binary, env, 'diagnostic-runtime')
                    daemon.alive()

    def native(self, sockets):
        for operation in ('check', 'build'):
            with ExitStack() as stack:
                states, envs, daemons = {}, {}, {}
                for lane in LANES:
                    state = self.output / 'work/native' / operation / lane
                    states[lane] = state
                    envs[lane] = self.environment(state, sockets / (lane + '.sock'))
                    daemons[lane] = stack.enter_context(S.Daemon(self.recorder, self.daemon_tool(lane), envs[lane], self.fixture, lane, 'native-' + operation + '-' + lane))
                    for _ in range(3): self.compile(lane, operation, state, envs[lane], 'diagnostic-prime')
                for sample, order in enumerate(orders(self.args.pairs)):
                    for lane in order:
                        daemons[lane].alive()
                        command, binary = self.compile(lane, operation, states[lane], envs[lane], 'diagnostic-' + operation)
                        row = {'kind':'diagnostic-timing','operation':operation,'sample':sample,'lane':lane,'command':command['id'],'seconds':command['seconds']}
                        self.recorder.event(row)
                        self.report['native_timings'].append(row)
                        if binary is not None: self.runtime(lane, binary, envs[lane], 'diagnostic-runtime')
                        daemons[lane].alive()

    def component(self, sockets):
        state = self.output / 'work/selector'
        env = self.environment(state, sockets / 'selector-unused.sock')
        argv = [self.selector, 'selection', self.tools['entry'].parent, *self.command('entry','check',state)[1:]]
        trace = state / 'selection.jsonl'
        def proof(stage):
            traced = dict(env, LUSTR_SELECTION_DIAGNOSTICS=str(trace))
            for _ in range(3): self.recorder.run(argv,traced,self.fixture,'cached','selector-prime')
            before = trace.read_bytes()
            command, stdout, _ = self.recorder.run(argv,traced,self.fixture,'cached','selector-proof')
            selector_result(stdout,self.fixture)
            after = trace.read_bytes()
            require(after.startswith(before), 'selector diagnostics were replaced')
            data = after[len(before):]
            require([json.loads(line) for line in data.splitlines()] == [{'selection_cache':'hit'}], 'selector helper has no retained hit')
            self.recorder.event({'kind':'selector-hit','stage':stage,'command':command['id'],'trace':self.archive(command,'selector.trace',data)})
        proof('before')
        for _ in range(3): self.recorder.run(argv,env,self.fixture,'cached','selector-prime')
        for sample in range(self.args.pairs):
            for lane in (('cached','fresh') if sample % 2 == 0 else ('fresh','cached')):
                local = dict(env, LUSTR_NO_SELECTION_CACHE='1') if lane == 'fresh' else env
                command, stdout, _ = self.recorder.run(argv,local,self.fixture,lane,'selector-timed')
                row = {'kind':'selector-timing','sample':sample,'lane':lane,'command':command['id'],
                       'seconds':command['seconds'],'component_seconds':selector_result(stdout,self.fixture)}
                self.recorder.event(row)
                self.report['selector_timings'].append(row)
        proof('after')

    def execute(self):
        identities = {str(path):S.digest(path) for path in [*self.tools.values(), self.selector, Path(__file__), HERE/'run-warm-entry-comparison.py']}
        self.report['identities'] = identities
        with tempfile.TemporaryDirectory(prefix='lustr-diagnostic-') as temporary:
            sockets = Path(temporary)
            self.qualification(sockets,'before')
            self.native(sockets)
            self.qualification(sockets,'after')
            self.component(sockets)
        require(S.inventory(self.fixture) == self.primary['fixture_files'], 'diagnostic fixture changed')
        require(all(S.digest(path) == value for path,value in identities.items()), 'diagnostic compiler or helper changed')
        require(B.source_identity(self.args.repository) == self.primary['candidate_identity'], 'diagnostic source revision changed')
        self.report.update(status='completed', summaries=summarize(self.recorder.records,self.args.pairs))


def render(report):
    lines = ['# Same-host residual diagnostics','', 'Status: '+report['status']+'.','',report['scope'],'',report['environment_note'],'',report['timing_note'],'']
    if report['status'] != 'completed': return '\n'.join(lines + ['No completed diagnostic result.',''])
    lines += ['| Operation | Public Lustr ms | Integrated Lust ms | Pristine Lust ms |','| --- | ---: | ---: | ---: |']
    for operation in ('check','build'):
        values = [statistics.median(row['seconds'] for row in report['native_timings'] if row['operation']==operation and row['lane']==lane)*1000 for lane in LANES]
        lines.append('| '+operation+' | '+' | '.join(f'{value:.6f}' for value in values)+' |')
    lines += ['', 'Paired ratios are reference/candidate; above one favors the candidate.','',
              '| Operation | Reference | Candidate | Ratio | Bootstrap 95% |','| --- | --- | --- | ---: | --- |']
    for row in report['summaries']['native']:
        a=row['analysis']; lines.append(f"| {row['operation']} | {row['reference']} | {row['candidate']} | {a['geometric_ratio']:.6f} | [{a['bootstrap_95'][0]:.6f}, {a['bootstrap_95'][1]:.6f}] |")
    a=report['summaries']['selector']
    lines += ['',f"Production full::select median: cached {a['candidate_median']*1000:.6f} ms; cache disabled {a['baseline_median']*1000:.6f} ms.",
              f"Paired disabled/cached ratio {a['geometric_ratio']:.6f}, interval [{a['bootstrap_95'][0]:.6f}, {a['bootstrap_95'][1]:.6f}].",'',
              'The original four-lane campaign and strict target remain authoritative. These groups use their own warmed state and explicit environment alignment. Selector values exclude parser/startup and use the separately source-built helper identity; do not subtract them as a precise decomposition.','']
    return '\n'.join(lines)


def verify(output):
    report = json.loads((output/'report.json').read_text())
    require(report['schema']==SCHEMA and report['status']=='completed','diagnostic is incomplete')
    require(S.digest(output/'raw.jsonl')==report['raw_sha256'],'diagnostic raw log changed')
    primary=json.loads((output/'source-report.json').read_text())
    require(S.digest(output/'source-report.json')==report['source_campaign_sha256'] and primary['status']=='completed'
            and primary['candidate_identity']==report['source'] and primary['pairs']==report['pairs'],'diagnostic source campaign differs')
    receipt=json.loads((output/'selector-receipt.json').read_text())
    require(S.digest(output/'selector-receipt.json')==report['selector_receipt_sha256'] and receipt['status']=='completed'
            and receipt['source']==report['source'] and receipt['binary_sha256']==report['selector_binary_sha256']==report['identities'][report['selector']], 'selector build binding differs')
    for lane in LANES:
        require(report['tool_paths'][lane]==primary['tool_paths'][lane] and report['identities'][report['tool_paths'][lane]]==primary['compiler_and_runner_sha256'][report['tool_paths'][lane]],'native diagnostic tool differs from the primary campaign')
    def stream(binding):
        path=Path(binding['file'])
        require(not path.is_absolute() and '..' not in path.parts and not (output/path).is_symlink(),'unsafe diagnostic stream')
        data=(output/path).read_bytes()
        require(S.digest(output/path)==binding['sha256'] and ('bytes' not in binding or len(data)==binding['bytes']),'diagnostic stream changed')
        return data
    oracle=(output/'runtime-oracle.stdout').read_bytes()
    require(S.digest(output/'runtime-oracle.stdout')==report['runtime_oracle_sha256']==primary['runtime_oracle_sha256'],'diagnostic oracle changed')
    rows=[json.loads(line) for line in (output/'raw.jsonl').read_text().splitlines()]
    commands, active, built, runtimes, retained, public, hits, retained_public = {}, {}, {}, {}, [], [], [], []
    for index,row in enumerate(rows):
        kind=row['kind']
        if kind=='command':
            require(row['id']==index and row['exit']==row['expected_exit']==0 and row['signal'] is None and not row['timed_out'] and not row['output_limit'] and not row.get('launch_error'),'failed diagnostic command')
            stream(row['stdout']); stream(row['stderr']); commands[row['id']]=row
        if kind=='daemon-start':
            require(row['lane'] not in active,'diagnostic daemon overlap'); active[row['lane']]=row
        if kind=='daemon-stop':
            require(row['lane'] in active and all(row[key]==active[row['lane']][key] for key in ('pid','scope')) and row['exit']==0 and row['error'] is None,'diagnostic daemon failed')
            del active[row['lane']]
            for item in row['streams']: stream(item)
        if kind=='build-identity':
            require(row['build_command'] in commands and commands[row['build_command']]['argv'][1]=='build','diagnostic build identity has no compilation')
            built[row['binary']]={key:row[key] for key in ('build_command','binary_sha256')}
        if kind=='runtime-identity':
            command=commands[row['command']]
            require(built[row['binary']]=={key:row[key] for key in ('build_command','binary_sha256')} and command['argv']==[row['binary']]
                    and command['phase']=='diagnostic-runtime' and stream(command['stdout'])==oracle and stream(command['stderr'])==b'', 'diagnostic runtime differs')
            require(row['build_command'] not in runtimes,'repeated diagnostic runtime binding'); runtimes[row['build_command']]=row['command']
        if kind=='retained-proof':
            command=commands[row['command']]
            require(command['phase']=='diagnostic-retained' and command['lane']==row['lane'] and row['lane'] in active and command['environment']==active[row['lane']]['environment']
                    and command['environment'].get('LUST_STATS')=='1' and command['environment'].get(OPT_OUT)=='1', 'retained diagnostic owner differs')
            state=Path(command['environment']['CARGO_TARGET_DIR']).parent; fixture=Path(report['fixture'])
            expected=[report['tool_paths'][row['lane']],row['operation'],str(fixture/'src/slice_main.rs'),'--manifest-path',str(fixture/'Cargo.toml'),'--bin','slice','--offline','--locked',*S.FLAGS]
            if row['operation']=='build': expected+=['-o',str(state/'program')]
            require(command['argv']==expected and state.parts[-3:]==(row['stage'],row['operation'],row['lane']),'retained diagnostic command differs')
            require(B.retained_proof(stream(command['stderr']),row['operation'])=={key:row[key] for key in ('route','counters')},'retained diagnostic counters differ')
            retained.append((row['stage'],row['operation'],row['lane']))
            if row['lane']=='entry': retained_public.append(row['command'])
        if kind=='public-proof':
            require(commands[row['command']]['lane']=='entry' and [json.loads(line) for line in stream(row['selection']).splitlines()]==[{'selection_cache':'hit'}]
                    and [json.loads(line) for line in stream(row['entry']).splitlines()]==[{'engine':'lust','entry':'daemon'}],'diagnostic public route differs')
            public.append(row['command'])
        if kind=='selector-hit':
            command=commands[row['command']]
            require(command['phase']=='selector-proof' and command['argv'][0]==report['selector'] and 'LUSTR_NO_SELECTION_CACHE' not in command['environment']
                    and [json.loads(line) for line in stream(row['trace']).splitlines()]==[{'selection_cache':'hit'}],'selector retained proof differs')
            selector_result(stream(command['stdout']),Path(report['fixture']))
            hits.append(row['stage'])
        if kind in ('diagnostic-timing','selector-timing'):
            command=commands[row['command']]; env=command['environment']; fixture=Path(command['cwd']); state=Path(env['CARGO_TARGET_DIR']).parent
            require(command['lane']==row['lane'] and command['seconds']==row['seconds'] and env.get(OPT_OUT)=='1'
                    and all(key not in env for key in ('LUST_STATS','LUSTR_TRACE_PATH','LUSTR_SELECTION_DIAGNOSTICS','LUST_NO_DAEMON','LUST_NO_CACHE')),'diagnostic timing controls differ')
            require(str(fixture)==report['fixture'] and env['LUST_CACHE_DIR']==str(state/'native-cache'),'diagnostic state differs')
            native=[str(fixture/'src/slice_main.rs'),'--manifest-path',str(fixture/'Cargo.toml'),'--bin','slice','--offline','--locked',*S.FLAGS]
            if kind=='diagnostic-timing':
                require(row['lane'] in active and active[row['lane']]['environment']==env,'timed diagnostic has no retained daemon')
                expected=[report['tool_paths'][row['lane']],row['operation'],*native]
                if row['operation']=='build': expected+=['-o',str(state/'program')]
                require(command['phase']=='diagnostic-'+row['operation'] and state.parts[-3:]==('native',row['operation'],row['lane']),'native diagnostic phase differs')
            else:
                expected=[report['selector'],'selection',str(Path(report['tool_paths']['entry']).parent),'check',*native]
                require(command['phase']=='selector-timed' and state.name=='selector' and (env.get('LUSTR_NO_SELECTION_CACHE')=='1')==(row['lane']=='fresh'),'selector diagnostic control differs')
                require(selector_result(stream(command['stdout']),fixture)==row['component_seconds'],'selector component value differs')
            require(command['argv']==expected,'diagnostic command changed')
    require(not active and retained==[(stage,op,lane) for stage in ('before','after') for op in ('check','build') for lane in LANES]
            and public==retained_public and len(public)==4 and hits==['before','after'],'diagnostic qualification incomplete')
    native=[row for row in rows if row['kind']=='diagnostic-timing']; component=[row for row in rows if row['kind']=='selector-timing']
    require(native==report['native_timings'] and component==report['selector_timings'],'diagnostic timing rows differ')
    require([(r['operation'],r['sample'],r['lane']) for r in native]==[(op,i,lane) for op in ('check','build') for i,order in enumerate(orders(report['pairs'])) for lane in order],'native diagnostic schedule differs')
    require([(r['sample'],r['lane']) for r in component]==[(i,lane) for i in range(report['pairs']) for lane in (('cached','fresh') if i%2==0 else ('fresh','cached'))],'selector diagnostic schedule differs')
    require(all(row['command'] in runtimes for row in native if row['operation']=='build'),'timed diagnostic build lacks exact runtime proof')
    S.STATS.compare(summarize(rows,report['pairs']),report['summaries'],'residual diagnostics')
    require((output/'README.md').read_text()==render(report),'diagnostic table changed')
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    commands=parser.add_subparsers(dest='command',required=True)
    run=commands.add_parser('run')
    for name in ('campaign','selector','selector-receipt','output'): run.add_argument('--'+name,type=Path,required=True)
    replay=commands.add_parser('verify'); replay.add_argument('output',type=Path)
    args=parser.parse_args()
    if args.command=='verify':
        report=verify(args.output); print(render(report)); return
    args.campaign,args.selector,args.selector_receipt,args.output=(path.resolve() for path in (args.campaign,args.selector,args.selector_receipt,args.output))
    primary=B.verify(args.campaign)
    receipt=json.loads(args.selector_receipt.read_text())
    require(receipt['status']=='completed' and receipt['source']==primary['candidate_identity'] and receipt['binary_sha256']==S.digest(args.selector),'selector source/binary differs')
    args.output.mkdir(mode=0o700,parents=True,exist_ok=False)
    shutil.copyfile(args.campaign/'report.json',args.output/'source-report.json')
    shutil.copyfile(args.selector_receipt,args.output/'selector-receipt.json')
    campaign=Diagnostic(args,primary,receipt)
    try: campaign.execute()
    except BaseException as error:
        campaign.report['error']=f'{type(error).__name__}: {error}'; raise
    finally:
        campaign.recorder.raw.close(); campaign.report['raw_sha256']=S.digest(args.output/'raw.jsonl')
        S.write_json(args.output/'report.json',campaign.report); (args.output/'README.md').write_text(render(campaign.report))
    print(render(verify(args.output)))


if __name__=='__main__': main()
