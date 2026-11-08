"""Read-only renderer-loss/reset/startup ordering assertions over native audit."""
from pathlib import Path
import json
out = Path(__file__).resolve().parent
events = [json.loads(line) for line in (out / 'native-events.jsonl').read_text().splitlines()]
proof = json.loads((out / 'native-running-proof.json').read_text())
sessions = []
for event in events:
    if event['event'] == 'startup':
        sessions.append([])
    sessions[-1].append(event)
assert len(sessions) == 2
kills = [e for e in events if e['event'] == 'renderer_kill_requested']
assert len(kills) == 1
pid = kills[0]['detail']['pid']
assert pid > 1 and pid != proof['host_pid']
assert kills[0]['detail']['host_pid'] == proof['host_pid']
results = [e for e in events if e['event'] == 'renderer_kill_result']
assert len(results) == 1 and results[0]['detail'] == {'pid': pid, 'success': True}
assert [e['detail']['id'] for e in events if e['event'] == 'menu_action'] == ['accept-kill-renderer', 'accept-reload']
for session in sessions:
    loads = [e for e in session if e['event'] == 'main_page_load']
    shows = [e for e in session if e['event'] == 'main_initial_display']
    assert [e['detail']['event'] for e in loads] == ['Started', 'Finished', 'Started', 'Finished']
    assert all(e['detail']['visible'] is False for e in loads[:2])
    assert len(shows) == 1 and shows[0]['detail']['visible'] is True
    assert session.index(loads[1]) < session.index(shows[0])
    reset = [e for e in session if e['event'] == 'main_renderer_reset']
    assert len(reset) == 1
    assert session.index(loads[2]) < session.index(reset[0]) < session.index(loads[3])
result = {'actual_renderer_kill': 'passed', 'host_pid': proof['host_pid'],
          'renderer_pid': pid, 'renderer_reset_after_kill': 'passed',
          'restart_document_completed_and_displayed': True,
          'explicit_reload_reset_on_restart': 'passed',
          'note': 'Page completion/display audit does not prove painted content; native restart was blank until explicit reload.'}
(out / 'native-event-assertions.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
