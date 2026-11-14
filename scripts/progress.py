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

def public_path(path):
    parts = pathlib.PurePosixPath(path)
    if parts.is_absolute() or '..' in parts.parts or '.git' in parts.parts or '\\' in path or str(parts) != path:
        raise ValueError(f'invalid public evidence path: {path}')

def local(path):
    public_path(path)
    p = (ROOT / path).resolve()
    if not p.is_relative_to(ROOT) or not p.is_file():
        raise ValueError(f'missing or nonpublic evidence path: {path}')
    return p

def source_paths(report, current=False):
    """Directory scopes detect added/deleted inputs as well as changed files."""
    for path in [*report['source_hashes'], *report['source_scope']]:
        public_path(path)
    command = (['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z', '--'] if current
               else ['git', 'ls-tree', '-r', '--name-only', '-z', report['commit'], '--'])
    paths = subprocess.check_output(command + report['source_scope'], cwd=ROOT).decode().split('\0')
    return {path for path in paths if path and (not current or (ROOT / path).is_file())}

def validate_sources(report):
    for path, digest in report['source_hashes'].items():
        committed = subprocess.check_output(['git', 'show', report['commit'] + ':' + path], cwd=ROOT)
        if hashlib.sha256(committed).hexdigest() != digest:
            raise ValueError(f"{report['id']}: source does not match tested commit {path}")
    scoped = source_paths(report)
    if not scoped or not scoped <= report['source_hashes'].keys():
        raise ValueError(f"{report['id']}: source scope lacks tested source identities")
    if any(not any(path == scope or path.startswith(scope + '/') for path in scoped)
           for scope in report['source_scope']):
        raise ValueError(f"{report['id']}: source scope has no tested files")
    return scoped

def sources_current(report, scoped):
    return (scoped == source_paths(report, current=True)
            and all(hashlib.sha256(local(path).read_bytes()).hexdigest() == report['source_hashes'][path]
                    for path in scoped))

def implementation_paths(feature, by_id, seen=None):
    seen = set() if seen is None else seen
    if feature['id'] in seen:
        return set()
    seen = seen | {feature['id']}
    return set(feature['implementation_locations']).union(
        *(implementation_paths(by_id[dep], by_id, seen) for dep in feature['dependencies']))

def json_digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                     ensure_ascii=False, allow_nan=False).encode('utf-8')).hexdigest()

def reference_settings(reference):
    # The legacy entitlement field is account state, not an evaluated behavior setting.
    settings = reference.get('settings', {})
    if not isinstance(settings, dict):raise ValueError('reference settings must be an object')
    return {key: value for key, value in settings.items() if key != 'license_entitlement'}

def reference_identity(reference):
    identity = {key: reference.get(key) for key in ['baseline_id', 'baseline_revision', 'product']}
    for section, keys in {
            'installer': ['sha256'],
            'application': ['bundle_name', 'bundle_id', 'bundle_short_version', 'bundle_version',
                            'executable_name', 'executable_sha256', 'observed_ui_version',
                            'observed_ui_build', 'observed_runtime_architecture'],
            'host': ['os', 'version', 'build', 'architecture']}.items():
        values = reference.get(section)
        if values is None:values = {}
        if not isinstance(values, dict):raise ValueError(f'reference {section} must be an object')
        identity[section] = {key: values.get(key) for key in keys}
    return identity

def reference_frozen(reference):
    if reference.get('frozen') is not True:return False
    def text(value):
        return isinstance(value, str) and bool(value.strip())
    def known(value):
        if isinstance(value, dict):return all(known(item) for item in value.values())
        if isinstance(value, list):return all(known(item) for item in value)
        if isinstance(value, str):return text(value)
        return value is not None
    settings = reference_settings(reference)
    if (not isinstance(settings.get('units'), (str, dict)) or not settings['units']
            or not text(settings.get('locale'))
            or not all(isinstance(settings.get(key), dict) and settings[key] for key in ['preferences', 'graphics'])
            or not known(settings)):
        return False
    identity = reference_identity(reference)
    application = identity['application']
    if (not all(text(value) for value in [identity['baseline_id'], identity['product'], application['bundle_id'],
                                         application['observed_ui_version'], application['observed_runtime_architecture'],
                                         *identity['host'].values()])
            or type(identity['baseline_revision']) is not int or identity['baseline_revision'] < 1):
        return False
    build = application['observed_ui_build']
    return ((text(build) or type(build) is int and build > 0)
            and all(isinstance(value, str) and re.fullmatch('[0-9a-f]{64}', value)
                    for value in [identity['installer']['sha256'], application['executable_sha256']]))

def reference_qualification(report, data, reference, commit=None):
    """Pin normative inputs separately from mutable implementation/status prose."""
    if type(reference['baseline_revision']) is not int or reference['baseline_revision'] < 1:
        raise ValueError('reference baseline revision must be a positive integer')
    def contents(path):
        public_path(path)
        return (local(path).read_bytes() if commit is None else
                subprocess.check_output(['git', 'show', commit + ':' + path], cwd=ROOT))
    by_id = {feature['id']: feature for feature in data['features']}
    pending = list(report['feature_ids'])
    contracts = {}
    while pending:
        fid = pending.pop()
        if fid in contracts:continue
        feature = by_id[fid]
        if feature['reference_baseline'] != reference['baseline_id']:
            raise ValueError(f'{fid}: qualified contract baseline differs')
        documents = feature.get('contract_documents', {})
        for path, digest in documents.items():
            if hashlib.sha256(contents(path)).hexdigest() != digest:
                raise ValueError(f'{fid}: reviewed contract document hash differs {path}')
        scenarios = []
        for scenario in feature['scenarios']:
            for path, digest in scenario['fixture_hashes'].items():
                if hashlib.sha256(contents(path)).hexdigest() != digest:
                    raise ValueError(f'{fid}: qualified fixture hash differs {path}')
            scenarios.append(scenario | {'definition_sha256': hashlib.sha256(contents(scenario['path'])).hexdigest()})
        contracts[fid] = {
            'contract_sha256': json_digest({'required_reference_behavior': feature['required_reference_behavior'],
                                           'contract_documents': documents, 'dependencies': feature['dependencies']}),
            'scenarios_sha256': json_digest(scenarios),
            'comparison_criteria_sha256': json_digest(feature['comparison_criteria'])}
        pending.extend(feature['dependencies'])
    return {'baseline_revision': reference['baseline_revision'], 'baseline_sha256': json_digest(reference_identity(reference)),
            'settings_sha256': json_digest(reference_settings(reference)), 'contracts': contracts}

def qualification_current(report, data, reference):
    return (report['reference_baseline'] == reference['baseline_id']
            and report.get('reference_qualification') == reference_qualification(report, data, reference))

def qualification_matches_commit(report):
    committed_data = json.loads(subprocess.check_output(
        ['git', 'show', report['commit'] + ':parity/features.yaml'], cwd=ROOT))
    committed_reference = json.loads(subprocess.check_output(
        ['git', 'show', report['commit'] + ':parity/reference.json'], cwd=ROOT))
    return (report['reference_baseline'] == committed_reference['baseline_id']
            and report['reference_qualification'] == reference_qualification(
                report, committed_data, committed_reference, commit=report['commit']))

def validate(data):
    schema = read('parity/schema.json')
    jsonschema.Draft202012Validator(schema).validate(data)
    report_schema = jsonschema.Draft202012Validator(schema['$defs']['verification_report'])
    ids = [f['id'] for f in data['features']]
    if len(set(ids)) != len(ids):
        raise ValueError('duplicate feature ID')
    reference = read('parity/reference.json')
    by_id = {f['id']: f for f in data['features']}
    for f in data['features']:
        for dep in f['dependencies']:
            if dep not in ids or dep == f['id']:
                raise ValueError(f"{f['id']}: unknown or self dependency {dep}")
    reports = {}
    report_paths = {run['path'] for f in data['features'] for run in f['verification_runs']}
    report_paths.update(path.relative_to(ROOT).as_posix() for path in (ROOT / 'parity/reports').glob('*.json'))
    report_ids = set()
    for path in sorted(report_paths):
        report = json.loads(local(path).read_text())
        report_schema.validate(report)
        if not set(report['feature_ids']) <= by_id.keys():
            raise ValueError(f"{report['id']}: report covers an unknown feature")
        if report['id'] in report_ids:
            raise ValueError(f"duplicate run ID: {report['id']}")
        report_ids.add(report['id'])
        subprocess.run(['git', 'cat-file', '-e', report['commit'] + '^{commit}'], cwd=ROOT,
                       check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        scoped = validate_sources(report)
        for artifact, digest in report['artifact_hashes'].items():
            if hashlib.sha256(local(artifact).read_bytes()).hexdigest() != digest:
                raise ValueError(f"{report['id']}: stale artifact {artifact}")
        if report['kind'] == 'reference_comparison' and 'reference_qualification' in report:
            if not qualification_matches_commit(report):
                raise ValueError(f"{report['id']}: qualification differs from tested commit")
        if report['freshness'] == 'current':
            if report['reference_baseline'] != reference['baseline_id']:
                raise ValueError(f"{report['id']}: report baseline differs")
            if not sources_current(report, scoped):
                raise ValueError(f"{report['id']}: changed scoped source requires rerun or stale evidence")
            for fid in report['feature_ids']:
                if not implementation_paths(by_id[fid], by_id) <= scoped:
                    raise ValueError(f"{fid}: current report scope omits feature or dependency implementation")
            if report['kind'] == 'reference_comparison' and (report['reference_executed'] or report['status'] != 'blocked'):
                if not report['reference_executed'] or not qualification_current(report, data, reference):
                    raise ValueError(f"{report['id']}: changed or missing reference qualification requires rerun or stale evidence")
                if (report['complete_contract'] and report['status'] == 'passed'
                        and any(not by_id[fid].get('contract_documents') or not by_id[fid]['scenarios']
                                for fid in report['reference_qualification']['contracts'])):
                    raise ValueError(f"{report['id']}: complete comparison requires reviewed contract documents and scenarios")
        reports[path] = report, scoped
    for f in data['features']:
        if f['reference_baseline'] != reference['baseline_id']:
            raise ValueError(f"{f['id']}: baseline mismatch")
        for path in f['implementation_locations'] + f['evidence_ids']:
            local(path)
        for path, digest in f.get('contract_documents', {}).items():
            if hashlib.sha256(local(path).read_bytes()).hexdigest() != digest:
                raise ValueError(f"{f['id']}: reviewed contract document hash differs {path}")
        for scenario in f['scenarios']:
            local(scenario['path'])
            for fixture, digest in scenario['fixture_hashes'].items():
                if hashlib.sha256(local(fixture).read_bytes()).hexdigest() != digest:
                    raise ValueError(f"{f['id']}: stale fixture {fixture}")
        current_reports = []
        for run in f['verification_runs']:
            report, scoped = reports[run['path']]
            if f['id'] not in report.get('feature_ids', []):
                raise ValueError(f"{f['id']}: report does not cover this feature")
            if any(report[key] != run[key] for key in ['id', 'commit', 'timestamp', 'platform', 'artifact_hashes']):
                raise ValueError(f"{f['id']}: report attribution differs")
            if report['freshness'] == 'current':
                current_reports.append(report)
        if f['freshness'] == 'current' and not current_reports:
            raise ValueError(f"{f['id']}: current evidence requires a current source-qualified run")
        if f['verification_status'] == 'passed':
            if f['freshness'] != 'current' or not any(r['status'] == 'passed' for r in current_reports):
                raise ValueError(f"{f['id']}: passing claim requires a current attributable run")
            if any(r['freshness'] == 'current' and r['status'] == 'failed' and f['id'] in r['feature_ids']
                   for r, _ in reports.values()):
                raise ValueError(f"{f['id']}: current failed verification must remain visible")
    active=set()
    visited=set()
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
    if (not reference_frozen(reference) or feature['implementation_status'] != 'implemented'
            or feature['verification_status'] != 'passed' or feature['reference_observation_status'] != 'observed'
            or feature['freshness'] != 'current' or not feature.get('contract_documents') or not feature['scenarios']):
        return False
    required = implementation_paths(feature, by_id)
    reports = (read(run['path']) for run in feature['verification_runs'])
    if not any(r.get('freshness') == 'current' and r.get('kind') == 'reference_comparison'
               and r.get('status') == 'passed' and r.get('complete_contract') is True
               and r.get('reference_executed') is True and feature['id'] in r.get('feature_ids', [])
               and r.get('reference_baseline') == feature['reference_baseline'] == reference.get('baseline_id')
               and qualification_current(r, {'features': list(by_id.values())}, reference)
               and qualification_matches_commit(r)
               and required <= (scoped := validate_sources(r)) and sources_current(r, scoped)
               for r in reports):return False
    return all(parity_verified(by_id[d],by_id,reference,seen) for d in feature['dependencies'])

def generate(data, reference):
    fs=data['features'];n=len(fs);by_id={f['id']:f for f in fs}
    verified={f['id']:parity_verified(f,by_id,reference) for f in fs}
    implementations=collections.Counter(f['implementation_status'] for f in fs)
    checks=collections.Counter(f['verification_status'] for f in fs)
    observations=collections.Counter(f['reference_observation_status'] for f in fs)
    freshness = collections.Counter(f['freshness'] for f in fs)
    runs = [(path, json.loads(path.read_text())) for path in (ROOT / 'parity/reports').glob('*.json')]
    runs.sort(key=lambda item: (item[1]['timestamp'], item[1]['id']), reverse=True)
    workflow = next(((path, report) for path, report in runs
                     if report.get('single_identified_candidate_workflow') is True
                     and report['freshness'] == 'current' and report['status'] == 'passed'), None)
    if workflow:
        path, report = workflow
        url = f'{BASE}/blob/main/parity/reports/{path.name}'
        workflow_markdown = f"Complete identified implementation workflow: **passed**, [{report['id']}]({url}), tested commit `{report['commit']}`. This bounded replay does not establish reference parity or complete the remaining product gates."
        workflow_html = f'<p>Complete identified implementation workflow: <strong>passed</strong>, <a href="{html.escape(url)}">{html.escape(report["id"])}</a>, tested commit <code>{report["commit"]}</code>. This bounded replay does not establish reference parity or complete the remaining product gates.</p>'
        next_milestone = 'Continue professional editing and sewing with relationship-preserving changes; retain tight-garment failures and investigate convergence/contact. Reference workflow acceptance remains required.'
    else:
        workflow_markdown = 'No current complete identified implementation workflow run qualifies. Replay empty-project authoring, sewing, fitting, save/reopen/edit and independently checked export on the current relevant source.'
        workflow_html = '<p>' + html.escape(workflow_markdown) + '</p>'
        next_milestone = 'Rerun the bounded complete native garment workflow on one identified candidate, then continue professional editing and sewing. Reference workflow acceptance remains required.'
    lines=['# WeftForge progress','',f"Generated from [features.yaml]({BASE}/blob/main/parity/features.yaml); do not edit this report.",'',f"Reference: **{reference['baseline_id']}**, baseline revision {reference['baseline_revision']}, state **{reference['baseline_status']}**.",'',f"Discovery coverage: **incomplete**. {n} feature contracts inventoried; menu, dialog, shortcut, default, error and workflow discovery is unfinished. The whole product denominator remains unknown.",'',f"Implementation: {implementations['implemented']}/{n} implemented; {implementations['partial']}/{n} partial; {implementations['in_progress']}/{n} in progress; {implementations['planned']}/{n} planned.",'',f"Verified reference parity: **{sum(verified.values())}/{n}** inventoried contracts. Implementation-only checks do not count as reference parity.",'',f"Verification states: {', '.join(f'{k} {checks[k]}/{n}' for k in ['passed','failed','blocked','stale','not_run'])}.",'',f"Reference observation: {', '.join(f'{k} {observations[k]}/{n}' for k in ['observed','documentation_only','unobserved','blocked'])}.",'',f"Evidence freshness: {', '.join(f'{state} {freshness[state]}/{n}' for state in ['current', 'stale', 'missing'])}.",'','## Per-domain status','','| Domain | Inventoried | Implemented | Partial | Verified parity |','|---|---:|---:|---:|---:|']
    domains = []
    for domain in sorted({f['category'] for f in fs}):
        group=[f for f in fs if f['category']==domain]
        values = [domain, len(group), sum(f['implementation_status']=='implemented' for f in group),
                  sum(f['implementation_status']=='partial' for f in group), sum(verified[f['id']] for f in group)]
        lines.append('| ' + ' | '.join(map(str, values)) + ' |')
        domains.append('<tr>' + ''.join('<td>' + html.escape(str(value)) + '</td>' for value in values) + '</tr>')
    lines+=['','## Contracts','','| ID | Capability | Implementation | Verification | Freshness | Reference | Evidence / issue |','|---|---|---|---|---|---|---|']
    rows=[]
    for f in fs:
        links=[f"[run]({BASE}/blob/main/{r['path']})" for r in f['verification_runs']]+[f"[issue]({u})" for u in f['issues']]
        lines.append(f"| {f['id']} | {f['title']} | {f['implementation_status']} | {f['verification_status']} | {f['freshness']} | {f['reference_observation_status']} | {' · '.join(links) or 'No qualifying run'} |")
        cells = ''.join('<td>'+html.escape(str(v))+'</td>' for v in [f['id'],f['title'],f['implementation_status'],f['verification_status'],f['freshness'],f['reference_observation_status']])
        issue_links = ' · '.join(f'<a href="{html.escape(url)}">issue</a>' for url in f['issues'])
        rows.append(f'<tr id="feature-{f["id"]}">{cells}<td>{issue_links}</td></tr>')
    lines+=['','## Current gaps and next delivery','',workflow_markdown,'',next_milestone,'','Current reference access and remaining operator actions are recorded in [reference setup]('+BASE+'/blob/main/docs/reference-setup.md). See [STATUS.md]('+BASE+'/blob/main/STATUS.md), [ROADMAP.md]('+BASE+'/blob/main/ROADMAP.md), [workflow runs]('+BASE+'/actions), and [releases]('+BASE+'/releases).','', 'Known solver limits: discrete vertex/triangle contact has no full edge/edge continuous contact, contact search is quadratic, and fabric approximations are uncalibrated. Cross-platform launch, animation, advanced authoring and native commercial-project compatibility remain open.','']
    for f in fs:
        if f['known_gaps'] or f['blockers'] or f['contradictions']:
            lines.append(f"- **{f['id']}**: "+'; '.join(f['known_gaps']+f['blockers']+f['contradictions']))
    lines+=['','## Latest attributable runs','']
    run_rows = []
    for path, r in runs:
        lines.append(f"- [{r['id']}]({BASE}/blob/main/parity/reports/{path.name}): {r['kind']}, {r['status']}, **{r['freshness']}**, commit `{r['commit']}`, {r['platform']}, {r['timestamp']}." + (f" {r['freshness_reason']}" if r['freshness'] == 'stale' else ''))
        sources = []
        for scope in r['source_scope']:
            view = 'tree' if any(source.startswith(scope + '/') for source in r['source_hashes']) else 'blob'
            sources.append(f'<a href="{html.escape(BASE + "/" + view + "/" + r["commit"] + "/" + scope)}">{html.escape(scope)}</a>')
        reason = '<p>' + html.escape(r['freshness_reason']) + '</p>' if r['freshness'] == 'stale' else ''
        run_rows.append(f'<li><a href="{html.escape(BASE + "/blob/main/parity/reports/" + path.name)}">{html.escape(r["id"])}</a>: {html.escape(r["kind"])}, <strong>{r["status"]} / {r["freshness"]}</strong>; tested commit <a href="{BASE}/commit/{r["commit"]}"><code>{r["commit"]}</code></a>; {html.escape(r["timestamp"])}; {html.escape(r["platform"])}; baseline {html.escape(r["reference_baseline"])}.{reason}<details><summary>Relevant source scope</summary>{" · ".join(sources)}</details></li>')
    markdown='\n'.join(lines).rstrip()+'\n'
    dashboard='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>WeftForge progress</title><style>body{font:16px system-ui;background:#111b24;color:#dbe9ec;margin:3rem auto;max-width:1250px;padding:0 1rem}a{color:#7dd9ba}h1{font-size:3rem;margin-bottom:.2em}.count{font-size:2rem;color:#7dd9ba}table{width:100%;border-collapse:collapse}td,th{text-align:left;padding:.7em;border-bottom:1px solid #33414c}input{padding:.8em;width:95%;background:#24313d;color:white;border:1px solid #667985;border-radius:6px}small{color:#abc}tr[hidden]{display:none}</style><h1>WeftForge</h1><p>Open garment design. Patterns, fabric, and freedom.</p>'''
    dashboard+=f'<p class="count">{sum(verified.values())} / {n} inventoried contracts verified against reference</p><p>{implementations["partial"]} partial implementations; {implementations["implemented"]} implemented. Discovery incomplete; full-product denominator unknown.</p><p>Baseline {html.escape(reference["baseline_id"])} revision {reference["baseline_revision"]}: {html.escape(reference["baseline_status"])}. Implementation-only tests never establish parity.</p><p><a href="{BASE}/blob/main/docs/progress.md">Evidence and gaps</a> · <a href="{BASE}/blob/main/ROADMAP.md">Roadmap</a> · <a href="{BASE}/issues">Issues</a> · <a href="{BASE}/actions">CI</a> · <a href="{BASE}/releases">Releases</a></p>'
    dashboard += '<p>Verification: ' + '; '.join(f'{state} {checks[state]}/{n}' for state in ['passed', 'failed', 'blocked', 'stale', 'not_run']) + '. Evidence freshness: ' + '; '.join(f'{state} {freshness[state]}/{n}' for state in ['current', 'stale', 'missing']) + '.</p>'
    dashboard += '<h2>Current identified workflow</h2>' + workflow_html
    dashboard += '<h2>Next milestone</h2><p>' + html.escape(next_milestone) + '</p>'
    dashboard += '<h2>Per-domain status</h2><table><thead><tr><th>Domain</th><th>Inventoried</th><th>Implemented</th><th>Partial</th><th>Verified parity</th></tr></thead><tbody>' + ''.join(domains) + '</tbody></table>'
    attention = []
    for feature in fs:
        if feature['verification_status'] in ['failed', 'blocked']:
            gaps = '; '.join(feature['known_gaps'] + feature['contradictions'] + feature['blockers'])
            attention.append(f'<li><a href="#feature-{feature["id"]}">{feature["id"]} — {html.escape(feature["title"])}</a>: <strong>{feature["verification_status"]}</strong>. {html.escape(gaps)}</li>')
    dashboard += '<h2>Failed and blocked contracts</h2><ul>' + ''.join(attention) + '</ul>'
    dashboard += '<h2>Latest attributable runs</h2><ul>' + ''.join(run_rows) + '</ul>'
    dashboard+='<h2>Contracts</h2><label for="filter">Filter capabilities or status</label><p><input id="filter" type="search" placeholder="e.g. contact, blocked, planned"></p><table id="contracts"><thead><tr><th>ID</th><th>Capability</th><th>Implementation</th><th>Verification</th><th>Freshness</th><th>Reference</th><th>Issue</th></tr></thead><tbody>'+''.join(rows)+'</tbody></table><script>document.querySelector("#filter").addEventListener("input",e=>{for(const row of document.querySelectorAll("#contracts tbody tr"))row.hidden=!row.textContent.toLowerCase().includes(e.target.value.toLowerCase())})</script></html>\n'
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
