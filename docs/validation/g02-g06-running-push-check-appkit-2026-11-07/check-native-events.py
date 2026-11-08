"""Read-only assertions over the captured native AppKit acceptance audit."""
from pathlib import Path
import json
out = Path(__file__).resolve().parent
events = [json.loads(line) for line in (out / 'native-events.jsonl').read_text().splitlines()]
names = [event['event'] for event in events]
sequence = ['appkit_terminate_requested', 'appkit_terminate_dispatched',
            'appkit_termination_intercepted', 'appkit_terminate_returned']
assert all(names.count(name) == 1 for name in sequence)
assert [names.index(name) for name in sequence] == sorted(names.index(name) for name in sequence)
dispatch = next(event['detail'] for event in events if event['event'] == sequence[1])
assert dispatch == {'attached_sheet': False, 'mode': 'kCFRunLoopDefaultMode'}
assert [event['detail']['id'] for event in events if event['event'] == 'menu_action'] == ['accept-appkit-terminate']
sessions = []
for event in events:
    if event['event'] == 'startup':
        sessions.append([])
    sessions[-1].append(event)
assert len(sessions) == 2
for session in sessions:
    loads = [event for event in session if event['event'] == 'main_page_load']
    shows = [event for event in session if event['event'] == 'main_initial_display']
    assert [event['detail']['event'] for event in loads] == ['Started', 'Finished']
    assert all(event['detail']['visible'] is False for event in loads)
    assert len(shows) == 1 and shows[0]['detail']['visible'] is True
    assert session.index(loads[1]) < session.index(shows[0])
result = {'appkit_request_dispatch_intercept_return_order': 'passed',
          'dispatch': dispatch, 'native_test_actions': ['accept-appkit-terminate'],
          'startup_readiness_both_hosts': 'passed'}
(out / 'native-event-assertions.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
