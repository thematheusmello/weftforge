"""Record one root-dispatched call's complete retained-result lifecycle."""
import json
import os
from pathlib import Path
import sys
import time

CASE = Path(__file__).resolve().parent


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write('\n'); stream.flush(); os.fsync(stream.fileno())


phase, label = sys.argv[1:]
if not label or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' for c in label):
    raise RuntimeError('Invalid own call label')
marker = CASE/'CALL-IN-FLIGHT.json'
started = json.loads((CASE/'STARTED.json').read_text())
if phase == 'begin':
    if (CASE/'RESOURCE-ALARM.json').exists() or time.time()+60 >= started['action_cutoff_epoch']:
        raise RuntimeError('Resource alarm or insufficient complete-call allowance')
    now = time.time()
    next_marker = CASE/'CALL-IN-FLIGHT-next.json'
    write(next_marker, {'label': label, 'start_epoch': now, 'deadline_epoch': now+60,
          'limit': 'Whole root lifecycle including binding, helper preparation and retained result; no MCP cancellation guarantee'})
    os.link(next_marker, marker)
    next_marker.unlink()
    dispatched = time.time()
    if (CASE/'RESOURCE-ALARM.json').exists() or dispatched >= now+60 or dispatched+60 >= started['action_cutoff_epoch']:
        write(CASE/(label+'-BEGIN-REFUSAL.json'), {'status': 'NEGATIVE_NO_DISPATCH', 'epoch': dispatched,
              'reason': 'Call allowance or resource state changed during marker persistence; no refund/replay'})
        raise RuntimeError('Post-persistence call gate refused dispatch')
elif phase == 'end':
    call = json.loads(marker.read_text()); now = time.time()
    if call['label'] != label:
        raise RuntimeError('Wrong in-flight call')
    accepted = now < min(call['deadline_epoch'], started['action_cutoff_epoch']) and not (CASE/'RESOURCE-ALARM.json').exists()
    write(CASE/(label+'-CALL-TERMINAL.json'), {'call': call, 'retained_result_closed_epoch': now,
          'candidate_before_final_persistence_clock': accepted, 'elapsed_seconds': now-call['start_epoch']})
    marker.unlink()
    closed = time.time()
    accepted = accepted and closed < min(call['deadline_epoch'], started['action_cutoff_epoch'])
    print(json.dumps({'label': label, 'bounded_lifecycle': accepted,
                      'elapsed_through_persisted_receipt_seconds': closed-call['start_epoch'],
                      'closed_epoch': closed}, sort_keys=True))
    if not accepted:
        raise RuntimeError('Call lifecycle failed time/resource bound; preserve result without replay')
else:
    raise RuntimeError('Expected begin or end')
