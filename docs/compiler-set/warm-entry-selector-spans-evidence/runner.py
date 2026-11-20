#!/usr/bin/env python3
"""One predeclared selector span diagnostic; no speedup or whole-command claim."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import time


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_hash(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):
            h.update(block)
    return h.hexdigest()


def write_json(path,data):
    path.write_text(json.dumps(data,indent=2,sort_keys=True)+'\n')


def require(condition,message):
    if not condition:
        raise ValueError(message)


def source_identity(source):
    rev,tree=subprocess.check_output(['git','rev-parse','HEAD','HEAD^{tree}'],cwd=source,text=True).splitlines()
    require(not subprocess.check_output(['git','status','--porcelain'],cwd=source),'profile source is dirty')
    names=['tools/profile-native-preflight.rs','tools/selection-spans.rs','src/full.rs','src/selection_cache.rs',
           'src/owned_child.rs','src/host_runtime.rs','compiler/native/src/protocol.rs','upstream/bluefe-core.pin']
    return {'revision':rev,'tree':tree,'files':{name:file_hash(source/name) for name in names}}


def package_inventory(package):
    inventory=[]
    for directory in ('bin','lib'):
        for path in sorted((package/directory).iterdir()):
            if path.name=='lustr-selection-verified':continue
            if path.is_file():
                m=path.stat(); lm=path.lstat()
                inventory.append({'path':str(path.relative_to(package)),'link':os.readlink(path) if path.is_symlink() else None,
                                  'resolved':str(path.resolve(strict=True)),'sha256':file_hash(path),
                                  'stat':{'mode':m.st_mode,'dev':m.st_dev,'ino':m.st_ino,'uid':m.st_uid,'gid':m.st_gid,
                                          'size':m.st_size,'mtime_ns':m.st_mtime_ns,'ctime_ns':m.st_ctime_ns},
                                  'lstat_mode':lm.st_mode})
            else:
                inventory.append({'path':str(path.relative_to(package)),'kind':'directory' if path.is_dir() else 'other',
                                  'link':os.readlink(path) if path.is_symlink() else None})
    return inventory


def semantic(value):
    return {k:v for k,v in value.items() if k not in ('elapsed_ns','spans','cache_state')}


def distribution(values):
    ordered=sorted(values)
    return {'minimum_ns':ordered[0],'median_ns':statistics.median(ordered),
            'p95_ns':ordered[(len(ordered)*95+99)//100-1],'maximum_ns':ordered[-1]}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('source','profiled','original','package','fixture','output','package-receipt','profiled-receipt','original-receipt'):
        parser.add_argument('--'+name,type=Path,required=True)
    args=parser.parse_args()
    for key,value in vars(args).items():setattr(args,key,value.resolve(strict=True) if key!='output' else value.absolute())
    args.output.mkdir(parents=True,exist_ok=False)
    output=args.output
    (output/'streams').mkdir()
    project=output/'fixture';shutil.copytree(args.fixture,project)
    for name in ('home','cargo-home','tmp','cache'):(output/name).mkdir()
    environment={'PATH':str(args.package/'bin')+':/usr/local/bin:/usr/bin:/bin','HOME':str(output/'home'),
                 'LANG':'C','LC_ALL':'C','TZ':'UTC','CARGO_HOME':str(output/'cargo-home'),
                 'CARGO_TARGET_DIR':str(output/'target'),'CARGO_BUILD_BUILD_DIR':str(output/'intermediates'),
                 'CARGO_NET_OFFLINE':'true','CARGO_TERM_COLOR':'never','TMPDIR':str(output/'tmp'),
                 'LUST_CACHE_DIR':str(output/'cache'),'LUSTR_PROFILE_SELECTION_DETAIL':'1',
                 '__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT':'1'}
    native=['check',str(project/'src/slice_main.rs'),'--manifest-path',str(project/'Cargo.toml'),'--bin','slice',
            '--offline','--locked','--target','x86_64-unknown-linux-gnu','-C','panic=unwind',
            '-C','overflow-checks=on','-C','debug-assertions=on']
    schedule=[('prime',i) for i in range(3)]+[('excluded',i) for i in range(8)]+[('measured',i) for i in range(96)]
    fixture_hashes={str(p.relative_to(project)):file_hash(p) for p in sorted(project.rglob('*')) if p.is_file()}
    for label,path in [('package',args.package_receipt),('profiled',args.profiled_receipt),('original',args.original_receipt)]:
        shutil.copyfile(path,output/(label+'-receipt.json'))
    plan={'schema':'lustr-selector-deferred-span-plan-v1','source':source_identity(args.source),
          'helpers':{'profiled':{'path':str(args.profiled),'sha256':file_hash(args.profiled)},
                     'original':{'path':str(args.original),'sha256':file_hash(args.original)}},
          'package':str(args.package),'package_inventory':package_inventory(args.package),
          'receipt_sha256':{name:file_hash(output/(name+'-receipt.json')) for name in ('package','profiled','original')},
          'fixture_files':fixture_hashes,'native_args':native,'environment':environment,
          'schedule':[{'phase':phase,'index':index} for phase,index in schedule],
          'gate_order':['profiled-prime-tools','original-prime-tools','original-before',
                        '3primes','8excluded','96measured','original-after'],
          'scope':'Inclusive nested selector stage distributions only; no paired performance ratio, whole-command attribution or platform qualification.',
          'limits':['Reduced real package has no liblustr_native_core.so; exact layout/receipt captured.',
                    'Instrumentation calls a clock and thread-local counter for every span; no stage output or dynamic formatting until full::select returns.',
                    'Nested inclusive stages must not be summed. Residual uses only disjoint top-level capture/load/diagnostic spans.',
                    'No trace/stats diagnostic path is enabled during measured helper calls.',
                    'Uninstrumented fresh-Cargo before/after observations supply complete Selection equivalence, not a timing reference.'],
          'runner_sha256':file_hash(Path(__file__)),'created_unix_ns':time.time_ns()}
    write_json(output/'predeclared-plan.json',plan)
    records=[];frames=[]
    report={'schema':'lustr-selector-deferred-span-report-v1','status':'running','plan_sha256':file_hash(output/'predeclared-plan.json'),
            'records':records,'frames':frames,'error':None}
    def save():write_json(output/'report.json',report)
    save()
    def run(label,command,env):
        record={'sequence':len(records),'label':label,'argv':command,'cwd':str(project),'environment':env}
        prefix=f'{len(records):03}-{label}'
        records.append(record);save()
        started=time.monotonic_ns()
        try:
            result=subprocess.run(command,cwd=project,env=env,capture_output=True,timeout=180)
            code=result.returncode;stdout=result.stdout;stderr=result.stderr;timeout=False
        except subprocess.TimeoutExpired as error:
            code=None;stdout=error.stdout or b'';stderr=error.stderr or b'';timeout=True
        record.update(exit=code,timeout=timeout,wall_ns=time.monotonic_ns()-started)
        for name,data in [('stdout',stdout),('stderr',stderr)]:
            path=output/'streams'/(prefix+'.'+name);path.write_bytes(data)
            record[name]={'file':str(path.relative_to(output)),'sha256':digest(data),'bytes':len(data)}
        save();require(code==0 and not timeout,label+' failed; original streams retained')
        require(stderr==b'',label+' emitted unexpected stderr')
        return record,stdout
    try:
        for label,helper in [('profiled',args.profiled),('original',args.original)]:
            _,out=run(label+'-prime-tools',[str(helper),'prime-selection-tools',str(args.package/'bin')],environment)
            require(out==b'',label+' tool initialization emitted stdout')
        _,out=run('original-before',[str(args.original),'selection',str(args.package/'bin'),*native],dict(environment,LUSTR_NO_SELECTION_CACHE='1'))
        oracle=semantic(json.loads(out));require(oracle['success'] is True and isinstance(oracle['selection'],dict),'fresh selection lacks complete identity')
        for phase,index in schedule:
            record,out=run(phase+'-'+str(index),[str(args.profiled),'selection',str(args.package/'bin'),*native],environment)
            data=json.loads(out)
            require(semantic(data)==oracle,'profiled helper differs from fresh complete Selection')
            require(isinstance(data['elapsed_ns'],int) and data['elapsed_ns']>0,'missing outer selector duration')
            spans=data['spans'];require(len(spans)==35,'unexpected span set')
            require(all(set(v)=={'ns','calls'} and all(type(n) is int and n>=0 for n in v.values()) for v in spans.values()),'invalid span shape')
            require(spans['full_select']['calls']==1,'outer selector span missing')
            require(data['elapsed_ns']>=spans['full_select']['ns'],'outer duration below inner full_select span')
            if phase!='prime' or index>0:require(data['cache_state']=='hit','warm observation did not reuse exact selection')
            if phase=='measured':
                frame={'index':index,'command':record['sequence'],'elapsed_ns':data['elapsed_ns'],'cache_state':data['cache_state'],'spans':spans}
                frame['unclassified_top_level_ns']=spans['full_select']['ns']-sum(spans[k]['ns'] for k in ('capture','load','diagnostic'))
                require(frame['unclassified_top_level_ns']>=0,'top-level spans overlap or exceed full selector')
                frames.append(frame);save()
        _,out=run('original-after',[str(args.original),'selection',str(args.package/'bin'),*native],dict(environment,LUSTR_NO_SELECTION_CACHE='1'))
        require(semantic(json.loads(out))==oracle,'fresh complete Selection changed after observations')
        require({str(p.relative_to(project)):file_hash(p) for p in sorted(project.rglob('*')) if p.is_file()}==fixture_hashes,'fixture changed')
        require(len(frames)==96,'measured frame count differs')
        summary={name:{**distribution([f['spans'][name]['ns'] for f in frames]),
                       'calls':sorted(set(f['spans'][name]['calls'] for f in frames))} for name in sorted(frames[0]['spans'])}
        report['summary']={'outer_elapsed':distribution([f['elapsed_ns'] for f in frames]),
                           'unclassified_top_level':distribution([f['unclassified_top_level_ns'] for f in frames]),'spans':summary}
        report['status']='completed';save()
        print(json.dumps({'status':report['status'],'observations':len(frames),'outer_elapsed':report['summary']['outer_elapsed'],
                          'top_level':{k:summary[k] for k in ('full_select','capture','load','diagnostic')},
                          'unclassified_top_level':report['summary']['unclassified_top_level']},indent=2))
    except BaseException as error:
        report['status']='failed';report['error']=type(error).__name__+': '+str(error);save();raise


if __name__=='__main__':main()
