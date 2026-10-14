#!/usr/bin/env python3
import hashlib
import importlib.util
import json
from pathlib import Path
import runpy
import unittest
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('fixtures',ROOT/'scripts/tests/renderer_comparison_test.py')
fixtures=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixtures)
MODULE=runpy.run_path(str(ROOT/'scripts/assemble-renderer-replay'))

class AssemblyTests(unittest.TestCase):
    def setUp(self):
        self.f=fixtures.EvidenceTests();self.f.setUp();self.addCleanup(self.f.doCleanups)
        self.root=self.f.root
        original=self.root/'original';original.mkdir();(original/'suite.json').write_text('{}')
        self.identity=dict(source_suite=str(original),source_suite_sha256=hashlib.sha256(b'{}').hexdigest())
    def source(self,name,indices,invalid=(),suite_type='primary',replay=True):
        source=self.root/name;source.mkdir()
        full=MODULE['RUNNER']['matrix'](suite_type)
        for i in indices:
            trial=full[i]
            p,_=self.f.fixture(trial['backend']);m=json.loads((p/'manifest.json').read_text());m.update(trial);m["quality"]={}
            if suite_type=='sustained': m.update(duration_seconds=180,measured_elapsed_s=180.1,frames_requested=None)
            (p/'readback.json').write_text(json.dumps({'width':trial['width'],'height':trial['height']}))
            if i in invalid: (p/'host-status.json').write_text('{"status":"invalidated"}')
            (p/'manifest.json').write_text(json.dumps(m));p.rename(source/MODULE['RUNNER']['trial_name'](i,trial))
        suite=dict(suite=suite_type,replay=replay,original_trial_count=len(full),original_trial_indices=indices,trials=[full[i] for i in indices],gpu_timing='on',memory_sampling='on',memory_interval_s=.25,status='failed' if invalid else 'complete',elapsed_s=10,**self.identity)
        (source/'suite.json').write_text(json.dumps(suite));return source
    def test_invalid_counterpart_excluded_and_complete_replacement_preserved(self):
        failed=self.source('failed',[0,1],[1]);valid=self.source('valid',[0,1])
        before=(failed/'suite.json').read_bytes()
        result=MODULE['assemble']([failed,valid],self.root/'derived',expected=[0,1])
        self.assertEqual(result['status'],'complete');self.assertEqual(result['completed'],2)
        self.assertEqual(result['elapsed_s'],20);self.assertEqual(len(result['excluded_trials']),2)
        self.assertEqual(before,(failed/'suite.json').read_bytes())
        self.assertEqual(result['original_trial_count'],320)
    def test_missing_pair_rejected_without_output(self):
        failed=self.source('failed',[0,1],[1]);output=self.root/'derived'
        with self.assertRaisesRegex(ValueError,'incomplete explicit'):MODULE['assemble']([failed],output,expected=[0,1])
        self.assertFalse(output.exists())
    def test_duplicate_complete_pairs_rejected(self):
        a=self.source('a',[0,1]);b=self.source('b',[0,1])
        with self.assertRaisesRegex(ValueError,'duplicate complete pair'):MODULE['assemble']([a,b],self.root/'derived',expected=[0,1])
    def test_mixed_measured_revisions_rejected(self):
        a=self.source('a',[0,1]);b=self.source('b',[2,3])
        for p in b.glob('*/manifest.json'):
            m=json.loads(p.read_text());m['source_revision']='different';p.write_text(json.dumps(m))
        with self.assertRaisesRegex(ValueError,'mixed measured revisions'):MODULE['assemble']([a,b],self.root/'derived',expected=[0,1,2,3])
    def test_sustained_full_guarded_and_replay_assembly(self):
        original=self.root/'original'
        full=MODULE['RUNNER']['matrix']('sustained')
        (original/'suite.json').write_text(json.dumps(dict(suite='sustained',status='complete',completed=6,trials=full)))
        self.identity['source_suite_sha256']=hashlib.sha256((original/'suite.json').read_bytes()).hexdigest()
        first=self.source('first',list(range(6)),invalid=(2,3,4,5),suite_type='sustained',replay=False)
        remaining=self.source('remaining',[2,3,4,5],suite_type='sustained')
        result=MODULE['assemble']([first,remaining],self.root/'derived',suite_type='sustained',original=original)
        self.assertEqual(result['completed'],6);self.assertEqual(result['suite'],'sustained')
        self.assertEqual(result['original_trial_count'],6);self.assertEqual(len(result['excluded_trials']),4)
    def test_mixed_original_sources_rejected(self):
        a=self.source('a',[0,1]);b=self.source('b',[2,3])
        meta=json.loads((b/'suite.json').read_text());meta['source_suite']='different-original';(b/'suite.json').write_text(json.dumps(meta))
        with self.assertRaisesRegex(ValueError,'different original'):MODULE['assemble']([a,b],self.root/'derived',expected=[0,1,2,3])
    def test_failed_explicit_original_rejected(self):
        original=self.root/'original';full=MODULE['RUNNER']['matrix']('sustained')
        (original/'suite.json').write_text(json.dumps(dict(suite='sustained',status='failed',completed=2,trials=full)))
        with self.assertRaisesRegex(ValueError,'complete matching'):MODULE['assemble']([],self.root/'derived',suite_type='sustained',original=original)
    def test_mixed_instrumentation_rejected(self):
        a=self.source('a',[0,1]);b=self.source('b',[2,3]);s=json.loads((b/'suite.json').read_text());s['memory_sampling']='off';(b/'suite.json').write_text(json.dumps(s))
        with self.assertRaisesRegex(ValueError,'mixed instrumentation'):MODULE['assemble']([a,b],self.root/'derived',expected=[0,1,2,3])
if __name__=='__main__':unittest.main()
