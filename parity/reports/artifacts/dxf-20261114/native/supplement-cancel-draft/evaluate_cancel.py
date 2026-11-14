#!/usr/bin/env python3
"""Separate native Cancel-draft saved-state oracle; no production imports."""
import argparse
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent


def load(path):
    return json.loads(path.read_bytes())


def check_documents(baseline, redone):
    assert baseline == load(HERE/'expected-baseline.weftforge'), 'baseline must be exact source-derived r23/.045 document after one Undo'
    assert redone == load(HERE/'expected-redone.weftforge'), 'redone must be exact source-derived r24/.05 document; Cancel must retain Redo'


def check_observations(observed):
    expected = load(HERE/'observation-template.json')
    for key, value in expected.items():
        if key not in ('post_cancel_image', 'action_log'):
            assert key in observed and type(observed[key]) is type(value) and observed[key] == value, f'observation {key}: {observed.get(key)!r} != {value!r}'


def check_artifact(record):
    path = Path(record['path'])
    assert not path.is_absolute() and '..' not in path.parts, 'artifact path must be acceptance-root relative'
    data = (ROOT/path).read_bytes()
    assert data, 'empty observation artifact'
    assert sha256(data).hexdigest() == record['sha256'], f'observation artifact hash differs: {path}'
    return {'path':str(path),'sha256':record['sha256']}


def self_test():
    baseline = load(HERE/'expected-baseline.weftforge')
    redone = load(HERE/'expected-redone.weftforge')
    observations = load(HERE/'observation-template.json')
    check_documents(baseline, redone)
    check_observations(observations)
    witnesses = []
    def reject(label, action):
        try:
            action()
        except AssertionError as exc:
            witnesses.append({'name':label,'rejected':True,'reason':str(exc)})
        else:
            raise AssertionError('negative witness accepted: '+label)
    def wrong_document(label, revision=None, resolution=None, baseline_change=False):
        changed = deepcopy(baseline if baseline_change else redone)
        if revision is not None:
            changed['project']['revision']=revision
        if resolution is not None:
            changed['project']['resolution']=resolution
        reject(label,lambda:check_documents(changed,redone) if baseline_change else check_documents(baseline,changed))
    wrong_document('original precommit-on-Cancel clears Redo witness',25,0.045)
    wrong_document('wrong Redo revision despite correct resolution',23,0.05)
    wrong_document('Redo unavailable leaves baseline resolution',24,0.045)
    wrong_document('pending .055 draft committed',24,0.055)
    wrong_document('Undo did not restore .045',23,0.05,True)
    changed=deepcopy(redone);changed['project']['points'][0]['position'][0]+=0.001
    reject('unrelated authored geometry changed',lambda:check_documents(baseline,changed))
    for key,value,label in [('post_cancel_pending_text','0.045','Cancel discarded pending field text'),('post_cancel_unsaved_visible',True,'Cancel committed an authoring change'),('post_cancel_vertex_count',1,'Cancel changed rest mesh'),('redo_invocations',2,'extra Redo attempt'),('undo_invocations',0,'missing Undo setup')]:
        changed_obs=deepcopy(observations);changed_obs[key]=value
        reject(label,lambda:check_observations(changed_obs))
    return {'accepted':True,'supplemental_native_outputs_read':False,'negative_witnesses':witnesses,'negative_witness_count':len(witnesses)}


def evaluate():
    errors=[]
    result={'accepted':False,'errors':errors,'files':{},'observation_evidence':{}}
    def attempt(label,action):
        try:
            return action()
        except Exception as exc:
            errors.append(f'{label}: {type(exc).__name__}: {exc}')
            return None
    paths=[ROOT/'native/cancel-draft-baseline.weftforge',ROOT/'native/cancel-draft-redone.weftforge',ROOT/'native/cancel-draft-observations.json']
    for path in paths:
        raw=attempt(path.name,lambda path=path:path.read_bytes())
        if raw is not None:
            result['files'][str(path.relative_to(ROOT))]=sha256(raw).hexdigest()
    baseline=attempt('read baseline',lambda:load(paths[0]))
    redone=attempt('read redone',lambda:load(paths[1]))
    observed=attempt('read observations',lambda:load(paths[2]))
    if baseline is not None and redone is not None:
        attempt('saved document/history consequence',lambda:check_documents(baseline,redone))
    if observed is not None:
        attempt('Cancel UI observations',lambda:check_observations(observed))
        for key in ('post_cancel_image','action_log'):
            artifact=attempt(key,lambda key=key:check_artifact(observed[key]))
            if artifact is not None:
                result['observation_evidence'][key]=artifact
    for name,wanted in load(HERE/'primary-file-hashes.json').items():
        attempt('primary immutable '+name,lambda name=name,wanted=wanted:require(sha256((ROOT/name).read_bytes()).hexdigest()==wanted,'primary file hash changed'))
    result['accepted']=not errors
    result['visual_evidence_interpretation']='Artifact references and producer fields checked; independent human/agent image review is separately required for the visual assertions.'
    return result


def require(condition,message):
    assert condition,message


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--self-test',action='store_true')
    parser.add_argument('--report',type=Path,required=True)
    args=parser.parse_args()
    result=self_test() if args.self_test else evaluate()
    result['scope']='Supplement only: pending numeric field and Redo preservation on native ExportDXF Cancel; original FILE-005 oracle unchanged.'
    result['evaluator_sha256']=sha256(Path(__file__).read_bytes()).hexdigest()
    args.report.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))
    return 0 if result['accepted'] else 1


if __name__=='__main__':
    raise SystemExit(main())
