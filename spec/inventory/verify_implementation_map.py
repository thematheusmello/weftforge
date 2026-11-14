#!/usr/bin/env python3
"""Check the source-pinned engineering crosswalk; this does not verify behavior."""

import hashlib
import json
import re
import subprocess
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAP = ROOT / 'spec/inventory/weftforge-implementation-map.json'


def check():
    data = json.loads(MAP.read_text())
    assert data['schema_version'] == 1
    commit = data['source_commit']
    assert re.fullmatch(r'[0-9a-f]{40}', commit)
    subprocess.run(['git', 'merge-base', '--is-ancestor', commit, 'HEAD'], cwd=ROOT, check=True)

    def pinned(path):
        assert not Path(path).is_absolute() and '..' not in Path(path).parts, path
        return subprocess.check_output(['git', 'show', f'{commit}:{path}'], cwd=ROOT)

    catalogs = {}
    for source in data['inputs']:
        raw = (ROOT / source['path']).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == source['sha256'], source['path']
        assert raw == pinned(source['path']), source['path']
        records = json.loads(raw)['records']
        assert len(records) == source['records']
        for record in records:
            assert record['id'] not in catalogs, record['id']
            catalogs[record['id']] = (source['catalog'], record)
    assert len(catalogs) == 755
    assert Counter(catalog for catalog, _ in catalogs.values()) == {
        'menus': 589, 'components': 166
    }

    origin = data['expansion_origin']
    assert origin['catalog_commit'] == commit
    previous_commit = origin['previous_map_commit']
    assert re.fullmatch(r'[0-9a-f]{40}', previous_commit)
    subprocess.run(['git', 'merge-base', '--is-ancestor', previous_commit, commit], cwd=ROOT, check=True)
    previous_raw = subprocess.check_output(
        ['git', 'show', f'{previous_commit}:spec/inventory/weftforge-implementation-map.json'], cwd=ROOT
    )
    assert hashlib.sha256(previous_raw).hexdigest() == origin['previous_map_sha256']
    previous = json.loads(previous_raw)
    previous_entries = {entry['catalog_id']: entry for entry in previous['entries']}
    assert len(previous_entries) == origin['preserved_catalog_ids'] == 650
    assert len(catalogs.keys() - previous_entries.keys()) == origin['additional_component_ids'] == 105
    assert all(catalogs[entry_id][0] == 'components' for entry_id in catalogs.keys() - previous_entries.keys())
    assert data['work_packets'][:len(previous['work_packets'])] == previous['work_packets']
    assert data['next_executable_parts'] == previous['next_executable_parts']
    assert data['status_authority']['path'] == previous['status_authority']['path']
    assert data['status_authority']['lookup'] == previous['status_authority']['lookup']
    for contract in data['reviewed_contract_sources']:
        raw = (ROOT / contract['path']).read_bytes()
        assert raw == pinned(contract['path'])
        assert hashlib.sha256(raw).hexdigest() == contract['sha256']
        assert contract['scope']
    assert data['source_snapshot_note']

    registry_path = data['status_authority']['path']
    registry_raw = pinned(registry_path)
    assert hashlib.sha256(registry_raw).hexdigest() == data['status_authority']['pinned_registry_sha256']
    # The repository stores its YAML registry in JSON-compatible YAML syntax.
    feature_ids = {feature['id'] for feature in json.loads(registry_raw)['features']}
    assert len(feature_ids) == 56
    current_features = {feature['id'] for feature in json.loads((ROOT / registry_path).read_text())['features']}
    assert feature_ids <= current_features

    groups = {group['id']: group for group in data['groups']}
    assert len(groups) == len(data['groups'])
    for old_group in previous['groups']:
        assert old_group['id'] in groups
        for key, value in old_group.items():
            current = groups[old_group['id']][key]
            assert (current[:len(value)] == value if isinstance(value, list) else current == value), (old_group['id'], key)
    packets = {packet['id']: packet for packet in data['work_packets']}
    assert len(packets) == len(data['work_packets']) == len(groups)
    assert {packet['order'] for packet in packets.values()} == set(range(1, len(packets) + 1))
    sources = data['source_landmarks']
    for source in sources.values():
        assert (ROOT / source['path']).is_file(), source['path']
        assert source['anchor'] in pinned(source['path']).decode(), source
        assert source['scope'], source
    document = pinned('crates/weftforge-core/src/document.rs').decode()
    command_enum = document.split('pub enum Command {', 1)[1].split('#[derive', 1)[0]
    commands = set(re.findall(r'^    ([A-Z]\w*)\s*[({]', command_enum, re.MULTILINE))
    assert len(commands) == 18

    fixture_ids = set(data['fixture_recipes'])
    for fixture in data['fixture_recipes'].values():
        assert fixture['setup'] and fixture['rule']
        for seed in fixture['existing_original_seeds']:
            assert (ROOT / seed).is_file(), seed
            pinned(seed)
    lanes = data['owner_lanes']
    for lane, owner in lanes.items():
        assert owner['role'] and owner['owns'], lane
        if lane == 'reference':
            continue
        for path in owner['owns']:
            assert (ROOT / path).exists(), (lane, path)
    for group in groups.values():
        assert group['feature_ids'] and set(group['feature_ids']) <= feature_ids, group['id']
        assert group['mapping_relation'] in data['mapping_relation_values'], group['id']
        assert set(group['source_refs']) <= sources.keys(), group['id']
        assert set(group['shared_commands']) <= commands, group['id']
        assert group['candidate_behavior'] and group['missing_functionality'] and group['runtime_questions'], group['id']
        assert group['fixture_ids'] and set(group['fixture_ids']) <= fixture_ids, group['id']
        assert group['packet_id'] in packets, group['id']
        assert packets[group['packet_id']]['group_id'] == group['id'], group['id']
        if group['mapping_relation'] == 'candidate_partial':
            assert group['source_refs'], group['id']

    entries = data['entries']
    entry_counts = Counter(entry['catalog_id'] for entry in entries)
    assert set(entry_counts) == set(catalogs)
    assert len(entries) == 755 and set(entry_counts.values()) == {1}
    current_entries = {entry['catalog_id']: entry for entry in entries}
    assert all(current_entries[entry_id] == entry for entry_id, entry in previous_entries.items())
    used_groups = set()
    for entry in entries:
        catalog, original = catalogs[entry['catalog_id']]
        assert entry['catalog'] == catalog and entry['path'] == original['path'], entry
        assert entry['group_id'] in groups, entry
        used_groups.add(entry['group_id'])
        classification = entry['classification']
        assert classification in {'action', 'component', 'container', 'heading', 'excluded'}, entry
        if catalog == 'components':
            assert classification == 'component', entry
        elif original['node_kind'] in {'submenu', 'empty_submenu'}:
            assert classification == 'container', entry
        else:
            assert classification in {'action', 'heading', 'excluded'}, entry
        if classification == 'heading':
            assert original['state_at_observation']['disabled_marker'], entry
            assert groups[entry['group_id']]['mapping_relation'] == 'ambiguous', entry
        if classification == 'excluded':
            assert groups[entry['group_id']]['scope_exclusion'], entry
    assert used_groups == set(groups)
    assert Counter(entry['classification'] for entry in entries) == {
        'action': 498, 'component': 166, 'container': 86, 'heading': 3, 'excluded': 2
    }

    required_checks = {
        'cargo fmt --all --check',
        'cargo test --workspace',
        'cargo clippy --workspace --all-targets -- -D warnings',
        'cargo xtask verify',
    }
    rules = data['packet_rules']
    assert set(rules['integration_checks']) == required_checks
    assert rules['scope_bound'] and rules['contract_input'] and rules['parity_rule']
    profiles = data['acceptance_profiles']
    for profile in profiles.values():
        assert profile['owner_lanes'] and set(profile['owner_lanes']) <= lanes.keys()
        assert profile['title'] and profile['decisive_acceptance']
    for packet in packets.values():
        assert packet['group_id'] in groups
        assert packet['acceptance_profile'] in profiles, packet['id']
        assert set(packet['depends_on']) <= packets.keys(), packet['id']
        assert packet['id'] not in packet['depends_on'], packet['id']
        assert all(packets[dependency]['order'] < packet['order'] for dependency in packet['depends_on']), packet['id']
    visited = set()
    active = set()

    def visit(packet_id):
        assert packet_id not in active, f'Packet dependency cycle at {packet_id}'
        if packet_id in visited:
            return
        active.add(packet_id)
        for dependency in packets[packet_id]['depends_on']:
            visit(dependency)
        active.remove(packet_id)
        visited.add(packet_id)

    for packet_id in packets:
        visit(packet_id)
    first = data['next_executable_parts']
    assert [part['packet_id'] for part in first] == [
        'PACK-rectangle', 'PACK-point-pattern-edit', 'PACK-segment-sewing'
    ]
    assert [packets[part['packet_id']]['order'] for part in first] == [1, 2, 3]
    for part in first:
        assert part['scope'] and part['wait_for'] and part['done_when']
        for lane, paths in part['owned_files_by_lane'].items():
            assert set(paths) <= set(lanes[lane]['owns']), (lane, paths)
    for lead in data['required_discovery_leads']:
        assert set(lead['feature_ids']) <= feature_ids and lead['reason']
    linked_features = {feature for group in groups.values() for feature in group['feature_ids']}
    linked_features.update(feature for lead in data['required_discovery_leads'] for feature in lead['feature_ids'])
    assert linked_features == feature_ids

    # Status and qualifying evidence stay in parity/features.yaml, never this crosswalk.
    forbidden = {
        'implementation_status', 'verification_status', 'reference_observation_status',
        'freshness', 'verification_runs', 'evidence_ids',
    }

    def no_status_copy(value):
        if isinstance(value, dict):
            assert not forbidden.intersection(value), forbidden.intersection(value)
            for item in value.values():
                no_status_copy(item)
        elif isinstance(value, list):
            for item in value:
                no_status_copy(item)

    no_status_copy(data)
    readable = (MAP.with_suffix('.md')).read_text()
    assert all(f'`{packet_id}`' in readable for packet_id in packets)
    assert commit in readable and '755 unique IDs' in readable
    print(f'PASS: 755 unique catalog IDs; preserved 650 entries/packets; {len(groups)} reviewed groups/packets; '
          f'{len(sources)} source landmarks; catalog hashes, feature/source links and acyclic dependencies valid.')


if __name__ == '__main__':
    check()
