#!/usr/bin/env python3
"""Validate attributable evidence and deterministically publish the parity registry."""
import argparse
import collections
import hashlib
import html
import json
import pathlib
import re
import subprocess
import sys
import jsonschema

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = 'https://github.com/thematheusmello/weftforge'

def read(path):
    return json.loads((ROOT / path).read_text())

def local(path):
    p = (ROOT / path).resolve()
    if not p.is_relative_to(ROOT) or not p.is_file():
        raise ValueError(f'missing or nonpublic evidence path: {path}')
    return p

def validate(data):
    jsonschema.Draft202012Validator(read('parity/schema.json')).validate(data)
    ids = [f['id'] for f in data['features']]
    if len(set(ids)) != len(ids):
        raise ValueError('duplicate feature ID')
    reference = read('parity/reference.json')
    for f in data['features']:
        if f['reference_baseline'] != reference['baseline_id']:
            raise ValueError(f"{f['id']}: baseline mismatch")
        for dep in f['dependencies']:
            if dep not in ids or dep == f['id']:
                raise ValueError(f"{f['id']}: unknown or self dependency {dep}")
        for path in f['implementation_locations'] + f['evidence_ids']:
            local(path)
        for scenario in f['scenarios']:
            local(scenario['path'])
            for fixture, digest in scenario['fixture_hashes'].items():
                if hashlib.sha256(local(fixture).read_bytes()).hexdigest() != digest:
                    raise ValueError(f"{f['id']}: stale fixture {fixture}")
        for run in f['verification_runs']:
            report = read(run['path'])
            if f['id'] not in report.get('feature_ids', []):
                raise ValueError(f"{f['id']}: report does not cover this feature")
            if report.get('reference_baseline') != reference['baseline_id']:
                raise ValueError(f"{f['id']}: report baseline differs")
            if report['id'] != run['id'] or report['commit'] != run['commit'] or report['timestamp'] != run['timestamp'] or report['platform'] != run['platform']:
                raise ValueError(f"{f['id']}: report attribution differs")
            subprocess.run(['git','cat-file','-e',run['commit']+'^{commit}'],cwd=ROOT,check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            for path,digest in run['artifact_hashes'].items():
                if hashlib.sha256(local(path).read_bytes()).hexdigest() != digest:
                    raise ValueError(f"{f['id']}: stale artifact {path}")
            if not report.get('source_hashes'):
                raise ValueError(f"{f['id']}: report lacks tested source identities")
            for path,digest in report['source_hashes'].items():
                committed = subprocess.check_output(['git','show',run['commit']+':'+path],cwd=ROOT)
                if hashlib.sha256(committed).hexdigest() != digest:
                    raise ValueError(f"{f['id']}: source does not match tested commit {path}")
                if f['freshness'] == 'current' and hashlib.sha256(local(path).read_bytes()).hexdigest() != digest:
                    raise ValueError(f"{f['id']}: changed source requires rerun or stale evidence: {path}")
        if f['verification_runs'] and f['verification_status'] == 'passed':
            latest = max((read(r['path']) for r in f['verification_runs']), key=lambda r:r['timestamp'])
            if latest['status'] != 'passed':
                raise ValueError(f"{f['id']}: latest failed verification must remain visible")
        if f['verification_status'] == 'passed' and not f['verification_runs']:
            raise ValueError(f"{f['id']}: passing claim requires attributable run")
    active=set()
    visited=set()
    by_id={f['id']:f for f in data['features']}
    def visit(fid):
        if fid in active: raise ValueError('dependency cycle')
        if fid in visited:return
        active.add(fid)
        for dep in by_id[fid]['dependencies']:visit(dep)
        active.remove(fid);visited.add(fid)
    for fid in ids:visit(fid)
    return reference

def parity_verified(feature, by_id, reference, seen=None):
    seen=set() if seen is None else seen
    if feature['id'] in seen:return False
    seen=seen|{feature['id']}
    if reference.get('frozen') is not True or feature['implementation_status'] != 'implemented' or feature['verification_status'] != 'passed' or feature['reference_observation_status'] != 'observed' or feature['freshness'] != 'current':
        return False
    reports=[read(r['path']) for r in feature['verification_runs']]
    if not any(r.get('kind')=='reference_comparison' and r.get('status')=='passed' and r.get('complete_contract') is True and feature['id'] in r.get('feature_ids', []) and r.get('reference_baseline') == feature['reference_baseline'] for r in reports):return False
    return all(parity_verified(by_id[d],by_id,reference,seen) for d in feature['dependencies'])

def generate(data, reference):
    fs=data['features'];n=len(fs);by_id={f['id']:f for f in fs}
    verified={f['id']:parity_verified(f,by_id,reference) for f in fs}
    implementations=collections.Counter(f['implementation_status'] for f in fs)
    checks=collections.Counter(f['verification_status'] for f in fs)
    observations=collections.Counter(f['reference_observation_status'] for f in fs)
    lines=['# WeftForge progress','',f"Generated from [features.yaml]({BASE}/blob/main/parity/features.yaml); do not edit this report.",'',f"Reference: **{reference['baseline_id']}**, baseline revision {reference['baseline_revision']}, state **{reference['baseline_status']}**.",'',f"Discovery coverage: **incomplete**. {n} feature contracts inventoried; menu, dialog, shortcut, default, error and workflow discovery is unfinished. The whole product denominator remains unknown.",'',f"Implementation: {implementations['implemented']}/{n} implemented; {implementations['partial']}/{n} partial; {implementations['in_progress']}/{n} in progress; {implementations['planned']}/{n} planned.",'',f"Verified reference parity: **{sum(verified.values())}/{n}** inventoried contracts. Implementation-only checks do not count as reference parity.",'',f"Verification states: {', '.join(f'{k} {checks[k]}/{n}' for k in ['passed','failed','blocked','stale','not_run'])}.",'',f"Reference observation: {', '.join(f'{k} {observations[k]}/{n}' for k in ['observed','documentation_only','unobserved','blocked'])}.",'','## Per-domain status','','| Domain | Inventoried | Implemented | Partial | Verified parity |','|---|---:|---:|---:|---:|']
    for domain in sorted({f['category'] for f in fs}):
        group=[f for f in fs if f['category']==domain];lines.append(f"| {domain} | {len(group)} | {sum(f['implementation_status']=='implemented' for f in group)} | {sum(f['implementation_status']=='partial' for f in group)} | {sum(verified[f['id']] for f in group)} |")
    lines+=['','## Contracts','','| ID | Capability | Implementation | Verification | Reference | Evidence / issue |','|---|---|---|---|---|---|']
    rows=[]
    for f in fs:
        links=[f"[run]({BASE}/blob/main/{r['path']})" for r in f['verification_runs']]+[f"[issue]({u})" for u in f['issues']]
        lines.append(f"| {f['id']} | {f['title']} | {f['implementation_status']} | {f['verification_status']} | {f['reference_observation_status']} | {' · '.join(links) or 'No qualifying run'} |")
        rows.append('<tr>'+''.join('<td>'+html.escape(str(v))+'</td>' for v in [f['id'],f['title'],f['implementation_status'],f['verification_status'],f['reference_observation_status']])+'</tr>')
    lines+=['','## Current gaps and next delivery','','The first gate remains open: complete an empty-project shirt authoring, sewing, fitting, save/reopen/edit and independently checked export through the actual desktop. Reference access is held at user-reviewed Terms of Service and trial activation. See [STATUS.md]('+BASE+'/blob/main/STATUS.md), [ROADMAP.md]('+BASE+'/blob/main/ROADMAP.md), and [workflow runs]('+BASE+'/actions).','', 'Known solver limits: discrete vertex/triangle contact has no full edge/edge continuous contact, contact search is quadratic, and fabric approximations are uncalibrated. Cross-platform launch, animation, advanced authoring and native commercial-project compatibility remain open.','']
    for f in fs:
        if f['known_gaps'] or f['blockers'] or f['contradictions']:
            lines.append(f"- **{f['id']}**: "+'; '.join(f['known_gaps']+f['blockers']+f['contradictions']))
    lines+=['','## Latest attributable runs','']
    for path in sorted((ROOT/'parity/reports').glob('*.json')):
        r=json.loads(path.read_text());lines.append(f"- [{r['id']}]({BASE}/blob/main/parity/reports/{path.name}): {r['kind']}, {r['status']}, commit `{r['commit']}`, {r['platform']}, {r['timestamp']}.")
    markdown='\n'.join(lines).rstrip()+'\n'
    dashboard='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>WeftForge progress</title><style>body{font:16px system-ui;background:#111b24;color:#dbe9ec;margin:3rem auto;max-width:1250px;padding:0 1rem}a{color:#7dd9ba}h1{font-size:3rem;margin-bottom:.2em}.count{font-size:2rem;color:#7dd9ba}table{width:100%;border-collapse:collapse}td,th{text-align:left;padding:.7em;border-bottom:1px solid #33414c}input{padding:.8em;width:95%;background:#24313d;color:white;border:1px solid #667985;border-radius:6px}small{color:#abc}tr[hidden]{display:none}</style><h1>WeftForge</h1><p>Open garment design. Patterns, fabric, and freedom.</p>'''
    dashboard+=f'<p class="count">{sum(verified.values())} / {n} inventoried contracts verified against reference</p><p>{implementations["partial"]} partial implementations; {implementations["implemented"]} implemented. Discovery incomplete; full-product denominator unknown.</p><p>Baseline {html.escape(reference["baseline_id"])} revision {reference["baseline_revision"]}: {html.escape(reference["baseline_status"])}. Implementation-only tests never establish parity.</p><p><a href="{BASE}/blob/main/docs/progress.md">Evidence and gaps</a> · <a href="{BASE}/blob/main/ROADMAP.md">Roadmap</a> · <a href="{BASE}/issues">Issues</a> · <a href="{BASE}/actions">CI</a></p>'
    dashboard+='<label for="filter">Filter capabilities or status</label><p><input id="filter" type="search" placeholder="e.g. contact, blocked, planned"></p><table><thead><tr><th>ID</th><th>Capability</th><th>Implementation</th><th>Verification</th><th>Reference</th></tr></thead><tbody>'+''.join(rows)+'</tbody></table><script>document.querySelector("#filter").addEventListener("input",e=>{for(const row of document.querySelectorAll("tbody tr"))row.hidden=!row.textContent.toLowerCase().includes(e.target.value.toLowerCase())})</script></html>\n'
    return {'docs/progress.md':markdown,'docs/dashboard/index.html':dashboard}

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    data=read('parity/features.yaml');reference=validate(data)
    for path,content in generate(data,reference).items():
        if args.check:
            if not (ROOT/path).exists() or (ROOT/path).read_text()!=content:raise ValueError(f'{path} is stale; run cargo xtask progress')
        else:(ROOT/path).write_text(content)
    print(f"Registry valid: {len(data['features'])} inventoried contracts; generated reports {'checked' if args.check else 'updated'}.")
if __name__=='__main__':
    try:main()
    except (ValueError,KeyError,jsonschema.ValidationError,subprocess.CalledProcessError) as e:
        print(f'progress validation failed: {e}',file=sys.stderr);sys.exit(1)
