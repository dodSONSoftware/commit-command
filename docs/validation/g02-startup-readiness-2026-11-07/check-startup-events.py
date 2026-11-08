"""Assert native startup ordering from the captured fixture audit, without driving UI."""
from pathlib import Path
import json
out = Path(__file__).resolve().parent
sessions = []
for line in (out / 'native-events.jsonl').read_text().splitlines():
    event = json.loads(line)
    if event['event'] == 'startup':
        sessions.append([])
    sessions[-1].append(event)
assert len(sessions) == 3
finished_ms = []
for session in sessions:
    loads = [event for event in session if event['event'] == 'main_page_load']
    displays = [event for event in session if event['event'] == 'main_initial_display']
    assert loads[0]['detail']['event'] == 'Started'
    assert loads[1]['detail']['event'] == 'Finished'
    assert loads[0]['detail']['visible'] is False
    assert loads[1]['detail']['visible'] is False
    assert len(displays) == 1 and displays[0]['detail']['visible'] is True
    assert session.index(loads[1]) < session.index(displays[0])
    finished_ms.append(loads[1]['detail']['elapsed_ms'])
# Each restarted host also exercised a subsequent renderer navigation.
assert all(len([e for e in s if e['event'] == 'main_page_load']) == 4 for s in sessions[1:])
assert all(any(e['event'] == 'main_renderer_reset' for e in s) for s in sessions[1:])
assert any(e['event'] == 'renderer_kill_result' and e['detail']['success'] for e in sessions[2])
result = {'host_processes': 3, 'hidden_until_initial_document_finished': True,
          'one_initial_display_per_process': True, 'initial_document_finished_ms': finished_ms,
          'renderer_reload_and_actual_renderer_loss_recovered': True}
(out / 'startup-event-assertions.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
