#!/usr/bin/env python3
"""Portable failure tests. No native/GPU work or generated evidence outside tempdir."""
import csv
import json
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
        m = dict(schema_version=1,status='complete',backend=backend,backend_variant='test',fixture='live',width=16,height=16,mode='completion',warmup=1,frames_requested=3,duration_seconds=None,repeat_id='pilot-0',source_revision='test',source_diff_hash='test',build={},dependencies={},runtime={},display={},power={},timing_provenance={},unsupported={},fixture_hash='x',input_hash='y',image_hash='z',init_ms=1,first_use={},completed_sample_count=3)
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
    def test_command_failure_and_recovery(self):
        for b in ('skia','vello'): self.fixture(b)
        suite = self.root/'suite.json'
        suite.write_text(json.dumps({'status':'failed','completed':1,'trials':[{},{}]}))
        command = [sys.executable,str(ROOT/'scripts/analyze-renderer-comparison'),str(self.root)]
        result = subprocess.run(command,capture_output=True,text=True)
        self.assertNotEqual(result.returncode,0)
        self.assertFalse((self.root/'summary.json').exists())
        suite.write_text(json.dumps({'status':'complete','completed':2,'trials':[{},{}]}))
        self.assertEqual(subprocess.run(command,capture_output=True).returncode,0)
        self.assertTrue((self.root/'summary.json').exists())
    def test_invalid_cli_fails_without_output(self):
        result = subprocess.run([sys.executable,str(ROOT/'scripts/run-renderer-comparison'),'--suite','bogus','--output',str(self.root/'out')],capture_output=True)
        self.assertNotEqual(result.returncode,0)
        self.assertFalse((self.root/'out').exists())
    def test_matrix_coverage(self):
        primary=R['matrix']('primary');self.assertEqual(len(primary),320)
        self.assertEqual(len(R['matrix']('sustained')),6)
        for i in range(0,len(primary),2):
            self.assertEqual({primary[i]['backend'],primary[i+1]['backend']},{'skia','vello'})
            self.assertEqual(primary[i]['repeat_id'],primary[i+1]['repeat_id'])
        self.assertEqual([x['backend'] for x in primary[:4]],['skia','vello','vello','skia'])
if __name__ == '__main__': unittest.main()
