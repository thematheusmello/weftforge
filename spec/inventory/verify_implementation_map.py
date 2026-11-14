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
    assert len(catalogs) == 949
    assert Counter(catalog for catalog, _ in catalogs.values()) == {
        'menus': 593, 'components': 356
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
    expansion = json.loads(previous_raw)
    expansion_entries = {entry['catalog_id']: entry for entry in expansion['entries']}
    assert len(expansion_entries) == origin['preserved_catalog_ids'] == 928
    added_ids = catalogs.keys() - expansion_entries.keys()
    assert Counter(catalogs[entry_id][0] for entry_id in added_ids) == {
        'menus': origin['additional_menu_ids'], 'components': origin['additional_component_ids']
    } == {'menus': 4, 'components': 17}
    assert {entry_id for entry_id in added_ids if catalogs[entry_id][0] == 'menus'} == {
        'MDUI-A376C6210DDA', 'MDUI-BDCB24B978FF', 'MDUI-7132EF1C1807', 'MDUI-C698D7B5E4AC'
    }
    assert all(catalogs[entry_id][1]['path'][0] == 'Open Project'
               for entry_id in added_ids if catalogs[entry_id][0] == 'components')
    assert origin['previous_source_commit'] == expansion['source_commit']
    subprocess.run(['git', 'merge-base', '--is-ancestor', expansion['source_commit'], commit], cwd=ROOT, check=True)

    update = data['source_update_origin']
    previous_commit = update['previous_map_commit']
    assert re.fullmatch(r'[0-9a-f]{40}', previous_commit)
    subprocess.run(['git', 'merge-base', '--is-ancestor', previous_commit, commit], cwd=ROOT, check=True)
    subprocess.run(['git', 'merge-base', '--is-ancestor', origin['previous_map_commit'], previous_commit],
                   cwd=ROOT, check=True)
    previous_raw = subprocess.check_output(
        ['git', 'show', f'{previous_commit}:spec/inventory/weftforge-implementation-map.json'], cwd=ROOT
    )
    assert hashlib.sha256(previous_raw).hexdigest() == update['previous_map_sha256']
    previous = json.loads(previous_raw)
    previous_entries = {entry['catalog_id']: entry for entry in previous['entries']}
    assert len(previous_entries) == update['preserved_catalog_ids'] == 949
    assert all(previous_entries[entry_id] == entry for entry_id, entry in expansion_entries.items())
    assert data['entries'] == previous['entries']
    assert update['previous_source_commit'] == previous['source_commit'] == origin['original_catalog_commit']
    subprocess.run(['git', 'merge-base', '--is-ancestor', previous['source_commit'], commit], cwd=ROOT, check=True)
    for source in data['inputs']:
        original = subprocess.check_output(
            ['git', 'show', f"{origin['original_catalog_commit']}:{source['path']}"], cwd=ROOT
        )
        assert original == pinned(source['path'])
    assert data['work_packets'] == previous['work_packets'] and len(data['work_packets']) == 205
    assert len(data['work_packets']) == update['preserved_packets']
    assert len(data['source_landmarks']) == update['preserved_landmarks'] == 56
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
    rechecks = {review['group_id']: review for review in data['source_rechecks']}
    assert len(rechecks) == len(data['source_rechecks'])
    reference_rechecks = {review['group_id']: review for review in data['reference_rechecks']}
    assert len(reference_rechecks) == len(data['reference_rechecks'])
    contract_paths = {contract['path'] for contract in data['reviewed_contract_sources']}
    for old_group in previous['groups']:
        assert old_group['id'] in groups
        review = rechecks.get(old_group['id'])
        reference_review = reference_rechecks.get(old_group['id'])
        if reference_review:
            assert reference_review['reason']
            assert reference_review['changed_fields'] and set(reference_review['changed_fields']) <= {
                'missing_functionality', 'runtime_questions', 'reviewed_reference_scope'
            }
            assert reference_review['contract_path'] in contract_paths
            assert all(groups[old_group['id']].get(key) != old_group.get(key)
                       for key in reference_review['changed_fields'])
        if review:
            assert review['reason'] and review['source_refs']
            assert set(review['changed_fields']) <= {'candidate_behavior', 'missing_functionality', 'shared_commands'}
            assert review['changed_fields']
            assert set(review['source_refs']) <= data['source_landmarks'].keys()
            changed_source = False
            for source_id in review['source_refs']:
                path = data['source_landmarks'][source_id]['path']
                old_source = subprocess.check_output(['git', 'show', f"{previous['source_commit']}:{path}"], cwd=ROOT)
                changed_source |= old_source != pinned(path)
            assert changed_source, old_group['id']
        for key, value in old_group.items():
            current = groups[old_group['id']][key]
            if (review and key in review['changed_fields']) or (reference_review and key in reference_review['changed_fields']):
                assert current != value, (old_group['id'], key)
                continue
            assert (current[:len(value)] == value if isinstance(value, list) else current == value), (old_group['id'], key)
    assert set(rechecks) <= {group['id'] for group in previous['groups']}
    assert set(reference_rechecks) <= {group['id'] for group in previous['groups']}
    packets = {packet['id']: packet for packet in data['work_packets']}
    assert len(packets) == len(data['work_packets']) == len(groups)
    assert {packet['order'] for packet in packets.values()} == set(range(1, len(packets) + 1))
    sources = data['source_landmarks']
    landmark_rechecks = {review['source_id']: review for review in data['source_landmark_rechecks']}
    assert len(landmark_rechecks) == len(data['source_landmark_rechecks'])
    assert set(sources) == set(previous['source_landmarks'])
    assert set(landmark_rechecks) <= sources.keys()
    for source_id, source in sources.items():
        old_source = previous['source_landmarks'][source_id]
        assert source.keys() == old_source.keys() and source['path'] == old_source['path'], source_id
        review = landmark_rechecks.get(source_id)
        changed_fields = set() if review is None else set(review['changed_fields'])
        if review:
            assert review['reason'] and changed_fields and changed_fields <= {'scope', 'anchor'}, source_id
            old_raw = subprocess.check_output(
                ['git', 'show', f"{previous['source_commit']}:{source['path']}"], cwd=ROOT
            )
            assert old_source['anchor'] in old_raw.decode() and old_raw != pinned(source['path']), source_id
            if 'anchor' in changed_fields:
                assert review['previous_anchor'] == old_source['anchor'], source_id
                assert review['current_anchor'] == source['anchor'], source_id
        for key, value in old_source.items():
            assert (source[key] != value if key in changed_fields else source[key] == value), (source_id, key)
        assert (ROOT / source['path']).is_file(), source['path']
        assert source['anchor'] in pinned(source['path']).decode(), source
        assert source['scope'], source
    document = pinned('crates/weftforge-core/src/document.rs').decode()
    command_enum = document.split('pub enum Command {', 1)[1].split('#[derive', 1)[0]
    commands = set(re.findall(r'^    ([A-Z]\w*)\s*[({]', command_enum, re.MULTILINE))
    assert commands == {
        'AddMaterial', 'AddPolygon', 'AddRectangle', 'AddSeam', 'AddSewingGroup',
        'JoinCurve', 'MovePoint', 'RemovePiece', 'RemoveSeam', 'RemoveSewingGroup',
        'ResizePiece', 'SetCurveControls', 'SetGrain', 'SetMannequin', 'SetMaterial',
        'SetPlacement', 'SetResolution', 'SplitCurve', 'Transaction',
        'UpdateMaterial', 'UpdateSeam', 'UpdateSewingGroup',
    }

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
    assert len(entries) == 949 and set(entry_counts.values()) == {1}
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
            if entry['catalog_id'] == 'MDUI-D76A4744102C':
                assert classification == 'excluded' and entry['group_id'] == 'vendor-session-administration', entry
            else:
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
        'action': 502, 'component': 355, 'container': 86, 'heading': 3, 'excluded': 3
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
    assert commit in readable and '949 unique IDs' in readable
    print(f'PASS: 949 unique catalog IDs; preserved 949 entries/205 packets and original 928-entry expansion; '
          f'{len(groups)} reviewed groups/packets; '
          f'{len(sources)} source landmarks; catalog hashes, feature/source links and acyclic dependencies valid.')


if __name__ == '__main__':
    check()
