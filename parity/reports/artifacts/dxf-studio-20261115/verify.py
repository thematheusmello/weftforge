#!/usr/bin/env python3
"""STATIC DXF file/transcript reproduction only; never invokes a producer or UI."""
import argparse
import copy
from datetime import datetime
import hashlib
import importlib.util
import inspect
import json
from pathlib import Path
import re
import sys
import tempfile

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
PRIVATE_MARKER=re.compile(rb'/' + rb'Users/|:' + rb'Users:|/' + rb'home/|/' + rb'var/folders/')
SUCCESS=['typed-command','cli-import','mcp-import','in-place']
def require(value,message):
    if not value:raise ValueError(message)
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def read(path):return json.loads(path.read_text())
def child(root,name):
    relative=Path(name)
    require(not relative.is_absolute() and '..' not in relative.parts,'Invalid artifact path')
    path=root/relative
    require(not path.is_symlink() and path.resolve().is_relative_to(root.resolve()),'Artifact escaped root')
    return path
def verify_tree():
    manifest=read(HERE/'MANIFEST.json')
    inventory={p.relative_to(HERE).as_posix() for p in HERE.rglob('*') if p.is_file()}
    require(inventory==set(manifest['files'])|{'MANIFEST.json'},'Packet inventory changed')
    for p in HERE.rglob('*'):
        require(not p.is_symlink(),'Public links are not executable evidence')
        if p.is_file():require(not PRIVATE_MARKER.search(p.read_bytes()),'Private absolute path found')
    for name,digest in manifest['files'].items():require(sha(child(HERE,name))==digest,'Packet artifact changed: '+name)
    return read(HERE/'BINDINGS.json')
def source_binding(repo,bound):
    source=read(HERE/'gates/source-pretest.json')
    require(source['source_commit']==bound['source_commit'] and len(source['source_hashes'])==49,'Wrong source binding')
    for name,digest in source['source_hashes'].items():require(sha(child(repo,name))==digest,'Runtime source differs: '+name)
    package=read(HERE/'gates/package-binding.json')
    require(package['all_five_gates_exit_zero'] and package['rust_tests']==187 and package['python_tests']==['48','6'],'Archived gates failed')
    require(package['binaries']['target/release/weftforge']==bound['native_sha256'] and
            package['binaries']['target/release/weftforge-cli']==bound['cli_sha256'],'Archived binary identities differ')
    require(all(job['status']=='completed' and job['exit_code']==0 for job in read(HERE/'gates/jobs.json')),'Gate job is not terminal/pass')
def snapshot(module):
    functions={name:value for name,value in vars(module).items() if inspect.isfunction(value)}
    return {'globals':dict(vars(module)),'functions':functions,
            'code':{name:value.__code__ for name,value in functions.items()},
            'defaults':{name:value.__defaults__ for name,value in functions.items()},
            'bodies':{name:inspect.getsource(value) for name,value in functions.items()}}
def unchanged(module,before):
    require(vars(module).keys()==before['globals'].keys(),'Frozen globals changed')
    for name,value in before['globals'].items():require(vars(module)[name] is value,'Frozen global replaced')
    for name,value in before['functions'].items():
        require(getattr(module,name) is value and value.__code__ is before['code'][name],'Frozen function/code changed')
        require(value.__defaults__==before['defaults'][name] and inspect.getsource(value)==before['bodies'][name],'Frozen default/body changed')
def differences(actual,expected,path=''):
    result=[]
    if isinstance(actual,dict) and isinstance(expected,dict):
        for key in expected:
            if key not in actual:result.append({'path':path+'/'+key,'kind':'missing'})
            else:result.extend(differences(actual[key],expected[key],path+'/'+key))
        for key in actual.keys()-expected.keys():result.append({'path':path+'/'+key,'kind':'extra'})
    elif isinstance(actual,list) and isinstance(expected,list):
        if len(actual)!=len(expected):result.append({'path':path,'kind':'length'})
        else:
            for i,(a,b) in enumerate(zip(actual,expected)):result.extend(differences(a,b,path+'/'+str(i)))
    elif actual!=expected:result.append({'path':path,'kind':'value','actual':actual,'expected':expected})
    return result
def complete(response,error=False):
    result=response['result']
    require(result['resultType']=='complete' and result['isError'] is error,'MCP result flags differ')
    require(json.loads(result['content'][0]['text'])==result['structuredContent'],'MCP text/structured JSON differ')
    return result['structuredContent']
def archive_reader():
    provenance=read(HERE/'PUBLICATION-PROVENANCE.json')
    copies={item['original_logical_path']:item for item in provenance['copies']}
    empty={item['original_logical_path'] for item in read(HERE/'OMISSIONS.json')['records'] if item.get('original_bytes')==0}
    def data(role,name):
        folder='dxf-headless-acceptance-v2-20261115' if role=='headless-v2' else 'dxf-headless-acceptance-20261115'
        key='.local/'+folder+'/'+name
        if key in empty:return b''
        item=copies[key];raw=(HERE/item['public_path']).read_bytes()
        if item.get('derivation','').startswith('Labeled'):return json.loads(raw)['text'].encode()
        return raw
    return data
def wire(root):
    requests=[json.loads(line) for line in (root/'stdin.jsonl').read_text().splitlines()]
    responses=[json.loads(line) for line in (root/'stdout.jsonl').read_text().splitlines()]
    indexed={row['id']:row for row in responses if 'id' in row}
    ids=[row['id'] for row in requests if 'id' in row]
    require(len(ids)==len(set(ids)),'Duplicate request id')
    require(set(ids)==set(indexed),'Missing/extra completed wire response')
    for response in root.glob('*-response.json'):
        record=read(response)
        require(record==indexed[record['id']],'Response record differs from raw wire')
    require(read(root/'result.json')['exit_code']==0,'Closed MCP child failed')
    return len(ids)
def pair_metadata(role,count):
    root=HERE/role/'trace'
    wanted=['000-launch']+[f'{i:03}' for i in range(1,count)]
    require(sorted(p.name for p in root.glob('*-before.json'))==[name+'-before.json' for name in wanted],'Native before set incomplete')
    for name in wanted:
        before,after=read(root/(name+'-before.json')),read(root/(name+'-after.json'))
        require(before.get('id',name)==after.get('id',name)==name and before['label']==after['label'],'Native pair identity differs')
        require(after['status']=='returned','Native action did not return')
        start=datetime.fromisoformat(before['started_utc'].replace('Z','+00:00'))
        end=datetime.fromisoformat(after['completed_utc'].replace('Z','+00:00'))
        require(start<=end,'Native call timestamps reversed')
def file_retention():
    results=read(HERE/'headless-v2/RESULTS.json')
    copies={row['original_logical_path']:row for row in read(HERE/'PUBLICATION-PROVENANCE.json')['copies']}
    require(results['case_count']==18 and results['independent_import_and_export_passes']==4 and
            results['rejection_and_retention_passes']==14 and results['exact_mcp_project_inspection_passes']==4 and
            results['mcp_exact_project_json_mismatches']==0 and results['import_retries']==results['continuations']==0,'Successor scope changed')
    for case in results['cases']:
        root=HERE/'headless-v2/cases'/case['case']
        before,final=read(root/'snapshot-before/manifest.json'),read(root/'snapshot-final/manifest.json')
        require(before['patterns.dxf']==final['patterns.dxf'],'Source copy changed')
        if case['kind']=='rejection':require(before==final and case['all_data_entries_retained'],'Closed rejection snapshot changed')
        for name,item in final.items():
            if item['kind']=='file':
                logical='.local/dxf-headless-acceptance-v2-20261115/cases/'+case['case']+'/data/'+name
                public=copies[logical]
                path=child(HERE,public['public_path'])
                require(public['original_sha256']==item['sha256'] and public['original_bytes']==item['size'],
                        'Original data differs from final snapshot')
                require(path.is_file() and sha(path)==public['public_sha256'] and path.stat().st_size==public['public_bytes'],
                        'Published data differs from declared original/alias provenance')
        if case['case']=='in-place':
            require(final['baseline.weftforge.recovery']['sha256']==before['baseline.weftforge']['sha256'],'In-place recovery differs from pre-save primary')
    return results
def terminal(role,expected):
    record=read(HERE/role/'terminal-jobs.json')
    require(record['state']=='ALL_TERMINAL' and len(record['jobs'])==record['product_and_grader_child_process_count']==expected,'Terminal job scope differs')
    require(all(row['status']=='completed' and isinstance(row['exit_code'],int) for row in record['jobs']),'Unfinished acceptance child')
    return record['tool_exec_session']
def adapter_negatives(project):
    changed=copy.deepcopy(project)
    changed['materials'][0]['color'][0]+=0.0000001
    require(differences(changed,project),'Exact MCP color mutation was ignored')
    good={'result':{'resultType':'complete','isError':False,'content':[{'text':'{"ok":true}'}],'structuredContent':{'ok':True}}}
    complete(good)
    bad=copy.deepcopy(good);bad['result']['content'][0]['text']='{"ok":false}'
    try:complete(bad)
    except ValueError:pass
    else:raise ValueError('Text/structured mutation accepted')
    try:child(HERE,'../escape')
    except ValueError:pass
    else:raise ValueError('Artifact traversal accepted')
    return ['exact color JSON mutation rejected','text/structured mutation rejected','parent traversal rejected']
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--repo-root',type=Path,default=Path.cwd())
    args=parser.parse_args();repo=args.repo_root.resolve()
    require(__debug__,'Run Python without optimization')
    bound=verify_tree();source_binding(repo,bound)
    require(sha(HERE/'evaluator/FROZEN.json')==bound['frozen_sha256'] and sha(HERE/'evaluator/evaluate.py')==bound['evaluator_sha256'],'Frozen source pin differs')
    require(not Path(tempfile.gettempdir()).resolve().is_relative_to(HERE) and
            not Path(tempfile.gettempdir()).resolve().is_relative_to(repo),'Synthetic scratch must be outside packet/repo')
    spec=importlib.util.spec_from_file_location('frozen_independent_dxf_evaluator',HERE/'evaluator/evaluate.py')
    evaluator=importlib.util.module_from_spec(spec);spec.loader.exec_module(evaluator)
    state,support=snapshot(evaluator),snapshot(evaluator.svg)
    freeze=evaluator.frozen();wanted,baseline=evaluator.expected()
    require(evaluator.self_check(baseline,wanted)==read(HERE/'evaluator/SELF_CHECKS.json'),'Frozen synthetic negatives differ')
    results=file_retention();new_projects={}
    for case,rid in zip(SUCCESS,range(4,8)):
        root=HERE/'headless-v2/cases'/case
        name='baseline.weftforge' if case=='in-place' else 'output.weftforge'
        document=evaluator.svg.load_json(root/'data'/name)
        evaluator.check_native(document,baseline,wanted)
        exports=evaluator.check_exports(document,root/'data/exported.svg',root/'data/exported.dxf')
        response=read(HERE/'headless-v2/mcp-session'/f'{rid:02}-response.json')
        require(complete(response)['project']==document['project'],'Successor full MCP/native JSON mismatch')
        new_projects[case]=document['project']
    for rid in range(3,11):complete(read(HERE/'headless-v2/mcp-session'/f'{rid:02}-response.json'),rid==10)
    wires={name:wire(HERE/'headless-v2'/name) for name in ['mcp-discovery','mcp-session']}
    old=read(HERE/'headless-old/RESULTS.json')
    require(old['mcp_exact_project_json_mismatches']==4 and old['independent_import_and_export_passes']==4 and old['rejection_and_retention_passes']==14,'Historical failure/coverage was lost')
    old_diffs={}
    for case in SUCCESS:
        root=HERE/'headless-old/cases'/case
        doc=read(root/'data'/('baseline.weftforge' if case=='in-place' else 'output.weftforge'))
        note=read(root/'inspection-check.json')
        response=read(HERE/'headless-old'/note['response'])
        actual=complete(response)['project']
        observed=differences(actual,doc['project'])
        require(observed==note['differences'] and len(observed)==3 and note['status']=='MISMATCH','Historical raw mismatch changed')
        old_diffs[case]=observed
    old_native=read(HERE/'native-old/OUTCOME.json')
    require(old_native['first_width_expected_revision']==2 and old_native['first_width_observed_revision']==3 and
            old_native['imports_performed']==0,'Original native failure was regraded')
    documents={name:evaluator.svg.load_json(HERE/'native-v2'/(name+'.weftforge')) for name in wanted['native_revisions']}
    evaluator.check_history(documents,baseline,wanted)
    native_exports={}
    for prefix in ['direct-final','FINAL']:
        native_exports[prefix]=evaluator.check_exports(documents['reopened'],HERE/'native-v2'/(prefix+'.svg'),HERE/'native-v2'/(prefix+'.dxf'))
    for suffix in ['svg','dxf']:require((HERE/'native-v2'/('direct-final.'+suffix)).read_bytes()==(HERE/'native-v2'/('FINAL.'+suffix)).read_bytes(),'Direct/final export bytes differ')
    for name in ['edited','rejected','reopened']:require((HERE/'native-v2'/(name+'.weftforge')).read_bytes()==(HERE/'native-v2/edited.weftforge').read_bytes(),'Sealed native final document differs')
    root_grade=json.loads(archive_reader()('headless-v2','cases/cli-import/logs/independent-grade/stdout.bin'))
    require(root_grade['status']=='INDEPENDENT_STATIC_CHECKS_PASSED','Archived headless grader failed')
    native_original=json.loads((HERE/'evaluator/EVALUATION-v2-native.stdout').read_text())
    require(native_original['status']=='INDEPENDENT_STATIC_CHECKS_PASSED','Original native file grade failed')
    actual_native=[sha(HERE/'native-v2'/(name+'.weftforge')) for name in wanted['native_revisions']]+[sha(HERE/'native-v2/FINAL.svg'),sha(HERE/'native-v2/FINAL.dxf')]
    require(sorted(actual_native)==sorted(native_original['artifact_sha256'].values()),'Native root grade hashes differ')
    review=read(HERE/'native-review/REVIEW.json')
    require(review['status']=='BOUNDED_NATIVE_WORKFLOW_SUPPORTED_WITH_REMAINING_GAPS' and review['unqualified_full_frozen_PLAN_or_contract_pass'] is False,'Native qualifications changed')
    pair_metadata('native-old',13);pair_metadata('native-v2',160)
    for name in bound['force_add_gitignored_sidecars']:
        sidecar=read(HERE/name);expected=copy.deepcopy(baseline);expected['project']['revision']=2
        for i in [1,2]:expected['project']['points'][i]['position'][0]=0.626
        evaluator.svg.exact(sidecar,expected,'TEMP2 sidecar')
        require(sha(HERE/name)=='0f7ccb1d824c005376140efbf813d0aa864b77750536c3a6dbf363b0c766a6f5','Sidecar bytes changed')
    lifecycle={name:read(HERE/'native-v2/lifecycle'/name)['matching_pids'] for name in
               ['PROCESS-INITIAL.json','PROCESS-ABSENT.json','PROCESS-FRESH.json','PROCESS-TEMP-DISCARDED.json','PROCESS-FINAL.json']}
    require(list(lifecycle.values())==[[7590],[],[14754],[],[21561]],'Own archived lifecycle projection differs')
    jobs={'old':terminal('headless-old',29),'successor':terminal('headless-v2',28)}
    negatives=adapter_negatives(new_projects['cli-import'])
    unchanged(evaluator,state);unchanged(evaluator.svg,support)
    evaluator.frozen();verify_tree();source_binding(repo,bound)
    print(json.dumps({'status':'BOUNDED_STATIC_REPRODUCTION_PASSED','source_commit':bound['source_commit'],'source49_hashes_match':True,
       'frozen_evaluator_and_support_function_code_bodies_defaults_globals_unchanged':True,'directory_or_default_rebindings':[],
       'new_headless_independent_grades':4,'new_headless_exact_full_MCP_inspections':4,'new_headless_archived_rejection_retention_cases':14,
       'new_actual_wire_request_response_pairs':wires,'native_nine_checkpoint_history':'passed','native_direct_and_final_export_pairs':'passed',
       'old_exact_MCP_mismatches_preserved':old_diffs,'old_native_Return_expected2_observed3_preserved':True,
       'old_case_grades_or_failures_reinterpreted':False,'native_full_frozen_PLAN_or_contract_pass':False,
       'remaining_native_qualifications':review['required_qualifications'],'sidecar_TEMP2_persistence_exact':True,
       'native_action_pair_metadata':{'old':13,'successor':160},'inventory_return_content_reduced':{'old':1,'successor':5},
       'raw_inventory_or_process_arrays_distributed':False,'archival_own_lifecycle_pids':lifecycle,
       'terminal_tool_handles':jobs,'frozen_synthetic_rejection_checks':13,'adapter_negative_checks':negatives,
       'ezdxf_version':evaluator.ezdxf.__version__,'ordinary_consumer_audit_errors_and_fixes':0,
       'current_hero':'Harbor REST arrangement only; not new drape or quality acceptance',
       'new_ui_product_physics_Cargo_network_runs':0,'new_saved_file_rejections_or_alias_guards_executed':0,
       'tracked_clean_publication_certified':False,'reference_parity':'0/56',
       'limits':'Only bound saved files and archived transcripts/snapshots are reproduced. No UI/product replay, full persistence/printing/DXF coverage, old.12 Redo-after-Cancel proof, all-plan/error-free native acceptance, original timing or reference parity.'},indent=2,allow_nan=False))
if __name__=='__main__':
    try:main()
    except Exception as error:
        print(json.dumps({'status':'FAILED','error_type':type(error).__name__,
                          'scope':'Static preparation refused; inspect private retained diagnostics; no producer/UI/physics execution'}),file=sys.stderr)
        raise
