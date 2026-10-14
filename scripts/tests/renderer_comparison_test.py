#!/usr/bin/env python3
"""Portable failure tests. No native/GPU work or generated evidence outside tempdir."""
import csv
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest
ROOT = Path(__file__).resolve().parents[2]
A = runpy.run_path(str(ROOT/'scripts/analyze-renderer-comparison'))
R = runpy.run_path(str(ROOT/'scripts/run-renderer-comparison'))

class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
    def fixture(self, backend):
        p = self.root/backend; p.mkdir()
        m = dict(schema_version=1,status='complete',backend=backend,backend_variant='test',gpu_timing='on',fixture='live',width=16,height=16,mode='completion',warmup=1,frames_requested=3,duration_seconds=None,repeat_id='pilot-0',source_revision='test',source_diff_hash='test',build={},dependencies={},runtime={},display={},power={},timing_provenance={},unsupported={},fixture_hash='x',input_hash='y',image_hash='z',init_ms=1,first_use={},completed_sample_count=3)
        (p/'manifest.json').write_text(json.dumps(m))
        rows = []
        for phase,count in [('first-use',1),('warmup',1),('measured',3)]:
            for i in range(count):
                row = dict.fromkeys(A['FIELDS'],'')
                row.update(phase=phase,frame_index=i,state_hash=str(i),gpu_missing_reason='unsupported',present_missing_reason='unsupported')
                for field in A['FIELDS']:
                    if field.endswith(('_ms','_s')) and not field.startswith(('gpu_','presented_')): row[field] = i+1
                rows.append(row)
        self.write_rows(p,rows)
        (p/'readback.json').write_text(json.dumps({'width':16,'height':16}))
        (p/'final.png').write_bytes(b'fixture')
        (p/'run.log').touch()
        (p/'host-status.json').write_text(json.dumps({'status':'complete'}))
        return p,rows
    def write_rows(self,p,rows):
        with (p/'frames.csv').open('w',newline='') as f:
            w=csv.DictWriter(f,fieldnames=A['FIELDS']);w.writeheader();w.writerows(rows)
    def test_known_statistics_and_missing(self):
        self.assertEqual(A['distribution']([1,2,3,4,5])['p95'],5)
        self.assertEqual(A['distribution'](list(range(1,101)))['p99'],99)
        paths=[self.fixture(b)[0] for b in ('skia','vello')]
        result=A['analyze'](paths)
        self.assertIsNone(result['runs'][0]['metrics']['gpu_span_ms']['p50'])
        self.assertEqual(result['paired'][0]['vello_minus_skia_ms']['service_ms']['p50'],0)
    def test_partial_run_rejected(self):
        p,rows=self.fixture('skia');self.write_rows(p,rows[:-1])
        with self.assertRaisesRegex(ValueError,'count'): A['validate'](p)
    def test_missing_gpu_reason_rejected(self):
        p,rows=self.fixture('skia');rows[-1]['gpu_missing_reason']='';self.write_rows(p,rows)
        with self.assertRaisesRegex(ValueError,'GPU'): A['validate'](p)
    def test_pair_geometry_mismatch(self):
        a,_=self.fixture('skia');b,rows=self.fixture('vello');rows[-1]['state_hash']='wrong';self.write_rows(b,rows)
        with self.assertRaisesRegex(ValueError,'hash'): A['analyze']([a,b])
    def test_invalidated_manifest_rejected(self):
        p,_=self.fixture('skia');m=json.loads((p/'manifest.json').read_text());m['status']='invalidated';(p/'manifest.json').write_text(json.dumps(m))
        with self.assertRaisesRegex(ValueError,'incomplete'): A['validate'](p)
    def test_nan_rejected(self):
        p,rows=self.fixture('skia');rows[-1]['service_ms']='nan';self.write_rows(p,rows)
        with self.assertRaisesRegex(ValueError,'invalid timing'): A['validate'](p)
    def test_truncated_sustained_duration_rejected(self):
        p,_=self.fixture('skia')
        m=json.loads((p/'manifest.json').read_text())
        m.update(duration_seconds=180,frames_requested=None,measured_elapsed_s=179.999)
        (p/'manifest.json').write_text(json.dumps(m))
        with self.assertRaisesRegex(ValueError,'truncated sustained'): A['validate'](p)
        m['measured_elapsed_s']=180.001
        (p/'manifest.json').write_text(json.dumps(m))
        A['validate'](p)
        m['measured_elapsed_s']=float('nan')
        (p/'manifest.json').write_text(json.dumps(m))
        with self.assertRaisesRegex(ValueError,'truncated sustained'): A['validate'](p)
    def test_command_failure_and_recovery(self):
        for b in ('skia','vello'): self.fixture(b)
        suite = self.root/'suite.json'
        suite.write_text(json.dumps({'status':'failed','gpu_timing':'on','completed':1,'trials':[{},{}]}))
        command = [sys.executable,str(ROOT/'scripts/analyze-renderer-comparison'),str(self.root)]
        result = subprocess.run(command,capture_output=True,text=True)
        self.assertNotEqual(result.returncode,0)
        self.assertFalse((self.root/'summary.json').exists())
        suite.write_text(json.dumps({'status':'complete','gpu_timing':'on','completed':2,'trials':[{},{}]}))
        self.assertEqual(subprocess.run(command,capture_output=True).returncode,0)
        self.assertTrue((self.root/'summary.json').exists())
    def test_invalid_cli_fails_without_output(self):
        result = subprocess.run([sys.executable,str(ROOT/'scripts/run-renderer-comparison'),'--suite','bogus','--output',str(self.root/'out')],capture_output=True)
        self.assertNotEqual(result.returncode,0)
        self.assertFalse((self.root/'out').exists())
    def test_gpu_timing_disabled_primary_rejected_before_launch(self):
        env = dict(os.environ, RENDERER_GPU_TIMING='off')
        for suite in ('primary','sustained'):
            command = [sys.executable,str(ROOT/'scripts/run-renderer-comparison'),'--suite',suite,'--output',str(self.root/'out'),'--dry-run']
            result = subprocess.run(command,capture_output=True,text=True,env=env)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('require on',result.stderr)
        command[command.index('--suite')+1]='pilot'
        result=subprocess.run(command,capture_output=True,text=True,env=env)
        self.assertEqual(result.returncode,0)
        self.assertEqual(json.loads(result.stdout)['gpu_timing'],'off')
        self.assertFalse((self.root/'out').exists())
    def test_mixed_gpu_timing_pair_rejected(self):
        a,_=self.fixture('skia');b,_=self.fixture('vello')
        m=json.loads((b/'manifest.json').read_text());m['gpu_timing']='off'
        (b/'manifest.json').write_text(json.dumps(m))
        with self.assertRaisesRegex(ValueError,'pair mismatch gpu_timing'): A['analyze']([a,b])
        m['repeat_id']='primary-0';(b/'manifest.json').write_text(json.dumps(m))
        with self.assertRaisesRegex(ValueError,'require GPU timing on'): A['validate'](b)
    def test_host_invalidation_overrides_complete_manifest(self):
        p,_=self.fixture('skia')
        (p/'host-status.json').write_text(json.dumps({'status':'invalidated','reason':'occluded during output drain'}))
        with self.assertRaisesRegex(ValueError,'native host did not complete'): A['validate'](p)
    def test_repaired_lifecycle_requires_assertion_and_visibility(self):
        p,_=self.fixture('skia')
        state=dict(window_visible=True,window_unoccluded=True,session_on_console=True,window_miniaturized=False,session_resigned=False,system_sleeping=False,screens_sleeping=False,display_asleep=False)
        host=dict(status='complete',lifecycle_start=state,lifecycle_end=dict(state),lifecycle_events=[],activity_held_through_output=True,activity_options=['userInitiated','latencyCritical','idleDisplaySleepDisabled'])
        path=p/'host-status.json';path.write_text(json.dumps(host));A['validate'](p)
        host['activity_held_through_output']=False;path.write_text(json.dumps(host))
        with self.assertRaisesRegex(ValueError,'assertion'): A['validate'](p)
        host['activity_held_through_output']=True;host['lifecycle_end']['window_unoccluded']=False;path.write_text(json.dumps(host))
        with self.assertRaisesRegex(ValueError,'lifecycle state'): A['validate'](p)
        host['lifecycle_end']['window_unoccluded']=True;host['lifecycle_events']=[dict(event='occluded',invalidating=True)];path.write_text(json.dumps(host))
        with self.assertRaisesRegex(ValueError,'invalidating'): A['validate'](p)
    def replay_source(self, suite='primary'):
        source=self.root/'original';source.mkdir()
        trials=R['matrix'](suite)
        (source/'suite.json').write_text(json.dumps(dict(suite=suite,status='complete',completed=len(trials),trials=trials)))
        for i,t in enumerate(trials):
            path=source/R['trial_name'](i,t);path.mkdir()
            (path/'manifest.json').write_text(json.dumps(dict(t,status='complete',source_revision='original')))
        return source
    def test_replay_expands_pairs_and_preserves_indices(self):
        source=self.replay_source()
        before=(source/'suite.json').read_bytes()
        selected,meta=R['replay_plan']('primary',source,'58,70,71,72,224,227,228,230,233')
        self.assertEqual([i for i,_ in selected],[58,59,70,71,72,73,224,225,226,227,228,229,230,231,232,233])
        self.assertEqual(len(selected),16)
        self.assertEqual(meta['original_trial_count'],320)
        self.assertTrue(meta['replay'])
        self.assertEqual(meta['source_revisions'],['original'])
        self.assertEqual(before,(source/'suite.json').read_bytes())
        self.assertTrue(R['trial_name'](*selected[0]).startswith('058-'))
    def test_replay_rejects_invalid_and_incomplete_source(self):
        source=self.replay_source()
        for indices in ('','1,','-1','320','1,1','1.5','abc'):
            with self.assertRaises(ValueError): R['replay_plan']('primary',source,indices)
        with self.assertRaises(ValueError): R['replay_plan']('sustained',source,'0')
        manifest=next(source.glob('*/manifest.json'));value=json.loads(manifest.read_text());value['backend']='wrong';manifest.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError,'mismatched'): R['replay_plan']('primary',source,'0')
        manifest.unlink()
        with self.assertRaisesRegex(ValueError,'missing'): R['replay_plan']('primary',source,'0')
        suite=json.loads((source/'suite.json').read_text());suite['status']='failed';(source/'suite.json').write_text(json.dumps(suite))
        with self.assertRaisesRegex(ValueError,'incomplete'): R['replay_plan']('primary',source,'0')
    def test_matrix_coverage(self):
        primary=R['matrix']('primary');self.assertEqual(len(primary),320)
        self.assertEqual(len(R['matrix']('sustained')),6)
        for i in range(0,len(primary),2):
            self.assertEqual({primary[i]['backend'],primary[i+1]['backend']},{'skia','vello'})
            self.assertEqual(primary[i]['repeat_id'],primary[i+1]['repeat_id'])
        self.assertEqual([x['backend'] for x in primary[:4]],['skia','vello','vello','skia'])
if __name__ == '__main__': unittest.main()
