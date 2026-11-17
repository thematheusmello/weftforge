"""Only changed runner guards/static source checks; never invokes a model or evaluator."""
import ast
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from unittest.mock import patch

import common as b
from common import ROOT, r


def main():
    reused=b.read(ROOT/"REUSED-SOURCES.json");same=[]
    for name,digest in reused["original_files"].items():
        if name.startswith(("consumer/","producer/")):
            r.require(r.sha(ROOT/name)==digest,"Changed numerical evaluator/producer: "+name);same.append(name)
    for file in [*ROOT.glob('*.py'),*(ROOT/'consumer').glob('*.py')]:ast.parse(file.read_text(),filename=str(file))
    if b.SOURCE is None:
        try:b.require_source()
        except ValueError as error:r.require('pending' in str(error),'wrong pending-source refusal')
        else:raise ValueError('pending source unexpectedly accepted')
    else:b.require_source()
    out=ROOT/'checks/tiny';out.mkdir();real_wait4=os.wait4;results=[]
    def child(name,code,seconds,cap,wait):
        directory=out/name;directory.mkdir()
        with patch.object(r.os,'wait4',wait):
            result=r.monitor([sys.executable,'-B','-c',code],ROOT,directory/'stdout',directory/'stderr',time.monotonic()+seconds,lambda:sum(p.stat().st_size for p in r.files_below(directory)),rss_cap=268435456,output_cap=cap,reserve=2*r.RECEIPT_RESERVE)
        b.persist(directory/'receipt.json',result);r.require(result['exit_code']==0 and result['peak_rss_bytes'] is not None,'fake child not normally reaped');results.append(result);return result
    normal=child('normal',"print('fake normal')",5,1048576,real_wait4);r.require(normal['stop_reason'] is None,'normal fake child failed')
    def late(pid,options):time.sleep(.08);return real_wait4(pid,0)
    late_result=child('late_terminal',"print('fake completed child')",.05,1048576,late);r.require(late_result['stop_reason']=='terminal_phase_wall_cap','terminal deadline accepted')
    def completed(pid,options):return real_wait4(pid,0)
    overflow=child('terminal_output',"print('x'*4096)",5,2*r.RECEIPT_RESERVE+1024,completed);r.require(overflow['stop_reason']=='terminal_phase_output_cap','terminal volume accepted')
    for script,args in [('controller.py',['--go','missing','--go-sha256','0'*64]),('bind_source.py',[])]:
        result=subprocess.run([sys.executable,'-B',str(ROOT/script),*args],env=dict(os.environ,PYTHONOPTIMIZE='1',PYTHONDONTWRITEBYTECODE='1'),capture_output=True,text=True,timeout=5)
        r.require(result.returncode!=0 and 'Python optimization guard' in result.stderr,'optimized launch not explicitly rejected')
    r.require(not any((ROOT/n).exists() for n in r.OUTPUT_DIRECTORIES) and not (ROOT/'SOURCE-PREFLIGHT.json').exists() and not (ROOT/'FREEZE.json').exists(),'source-only guard created actual case')
    report={'status':'passed_source_bound' if b.SOURCE else 'passed_source_pending','source_commit':b.SOURCE,'exact_reused_producer_and_consumer_files':same,'Python_AST':'passed','reviewed_source_guard':True,'terminal_phase_and_output_guards':True,'optimization_guards':True,'fake_children_reaped':len(results),'actual_model_initializations':0,'actual_steps':0,'actual_evaluator_runs':0,'Cargo_invocations':0,'real_case_clock_started':False,'old_qualification_reused_only_for_exact_sources':reused['original_qualification']}
    b.persist(ROOT/'checks/TINY-SELFCHECKS.json',report);print(json.dumps(report,indent=2))


if __name__=='__main__':main()
