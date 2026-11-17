#!/usr/bin/env python3
"""Synthetic evidence tests. Their times are fixtures, never benchmark results."""
import hashlib
import importlib.util
import io
import json
import math
from pathlib import Path
import tempfile
import unittest
import urllib.request
import zipfile

SPEC = importlib.util.spec_from_file_location('performance_report', Path(__file__).with_name('performance_report.py'))
p = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(p)


class StatisticsTests(unittest.TestCase):
    def test_known_constant_and_sign_test(self):
        a = p.estimate([(2., 1.)] * 8, 1000, 7)
        self.assertAlmostEqual(a['geometric_ratio'], 2.)
        self.assertEqual(a['bootstrap_95'], [2., 2.])
        self.assertEqual(a['exact_sign_flip_p'], 2 / 256)
        self.assertEqual(p.estimate([(1., 1.)] * 8, 100, 0)['exact_sign_flip_p'], 1.)

    def test_reverse_ratio_and_intervals(self):
        pairs = [(1., 2.), (4., 2.), (3., 2.), (8., 2.)]
        a = p.estimate(pairs, 2000, 19)
        b = p.estimate([(y, x) for x, y in pairs], 2000, 19)
        self.assertAlmostEqual(a['geometric_ratio'] * b['geometric_ratio'], 1.)
        self.assertAlmostEqual(a['bootstrap_95'][0] * b['bootstrap_95'][1], 1.)
        self.assertEqual(a['exact_sign_flip_p'], b['exact_sign_flip_p'])
        self.assertEqual(a['baseline_median'], 3.5)
        self.assertEqual(a['baseline_p95'], 8.)

    def test_invalid_timings(self):
        for bad in [0, -1, float('nan'), float('inf'), True]:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                p.estimate([(bad, 1.)], 100, 0)

    def test_determinism(self):
        self.assertEqual(p.estimate([(1., 1.), (2., 1.)], 100, 42), p.estimate([(1., 1.), (2., 1.)], 100, 42))

    def test_duplicate_json_and_nonfinite(self):
        for data in ['{"x":1,"x":2}', '{"x":NaN}', '{"x":Infinity}']:
            with self.assertRaises(ValueError):
                p.decode(data)

    def test_replayed_summary_cannot_be_changed(self):
        s = p.estimate([(1., 2.)] * 3, 100, 0)
        bad = dict(s, geometric_ratio=2.)
        with self.assertRaises(ValueError):
            p.equivalent(bad, s)

    def test_ci_crossing_one_is_not_equivalence(self):
        s = p.estimate([(1., 2.), (2., 1.)] * 6, 1000, 0)
        self.assertLess(s['bootstrap_95'][0], 1)
        self.assertGreater(s['bootstrap_95'][1], 1)

    def test_p95_is_maximum_with_twelve_pairs(self):
        s = p.estimate([(float(x), 1.) for x in range(1, 13)], 100, 0)
        self.assertEqual(s['baseline_p95'], 12.)


class ArchiveTests(unittest.TestCase):
    def archive(self, names):
        out = io.BytesIO()
        with zipfile.ZipFile(out, 'w') as z:
            for name in names:
                z.writestr(name, 'sample')
        return out.getvalue()

    def test_valid_archive_is_not_executed(self):
        data = self.archive(['evidence/run.txt'])
        self.assertEqual(p.safe_zip(data, p.sha(data)), {'evidence/run.txt': b'sample'})

    def test_digest_mismatch(self):
        with self.assertRaises(ValueError):
            p.safe_zip(self.archive(['good']), '0' * 64)

    def test_path_traversal_and_absolute(self):
        for name in ['../bad', '/bad', 'good/../../bad', 'good\\bad', './bad']:
            data = self.archive([name])
            with self.subTest(name=name), self.assertRaises(ValueError):
                p.safe_zip(data, p.sha(data))

    def test_duplicate_members(self):
        data = self.archive(['duplicate', 'duplicate'])
        with self.assertRaises(ValueError):
            p.safe_zip(data, p.sha(data))

    def test_symlink_member(self):
        out = io.BytesIO()
        with zipfile.ZipFile(out, 'w') as z:
            link = zipfile.ZipInfo('link')
            link.external_attr = 0o120777 << 16
            z.writestr(link, 'outside')
        with self.assertRaises(ValueError):
            p.safe_zip(out.getvalue(), p.sha(out.getvalue()))

    def test_redirect_removes_credentials(self):
        request = urllib.request.Request('https://api.github.com/repos/x/y', headers={'Authorization':'Bearer dummy-test'})
        redirected = p.PrivateRedirect().redirect_request(request, None, 302, 'Found', {}, 'https://blob.example.test/archive.zip')
        self.assertIsNone(redirected.get_header('Authorization'))
        with self.assertRaises(ValueError):
            p.PrivateRedirect().redirect_request(request, None, 302, 'Found', {}, 'http://blob.example.test/archive.zip')


class CompleteEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.campaign = {'head_sha': 'a'*40, 'checkout_sha': 'b'*40, 'run_id':1, 'artifact_id':2, 'sha256':'c'*64}
        cls.files = {'lustr-source-revision.txt': ('b'*40 + '\n' + 'd'*40 + '\n').encode()}
        hashes = {f'/compiler/binary{i}': 'e'*64 for i in range(10)}
        for name in ('BUILD-PROFILE.txt', 'COMPONENTS.txt', 'NATIVE-BUILD-RECEIPT.txt'):
            cls.files[p.MEASUREMENTS + name] = name.encode()
            hashes['/build/' + name] = p.sha(name.encode())
        events = []
        for family in ('native','cargo'):
            lanes = ('pristine','integrated','entry') if family == 'native' else ('reference','mixed')
            for phase in p.PHASES:
                for sample in range(12):
                    for lane in lanes:
                        name = f'{family}-{phase}-{sample}-{lane}'
                        code = (1 if family == 'native' else 101) if phase == 'rejection' else 0
                        event = {'kind':'command','family':family,'phase':phase,'lane':lane,'sample':sample,
                                 'seconds':1. if lane in ('pristine','reference') else 2.,
                                 'exit':code,'expected_exit':code,'signal':None,'timed_out':False,
                                 'source_sha256':{'main.rs':'f'*64}, 'source_after_sha256':{'main.rs':'f'*64}}
                        for stream in ('stdout','stderr'):
                            event[stream] = name + '.' + stream
                            event[stream+'_sha256'] = p.sha(b'')
                            event[stream+'_bytes'] = 0
                            cls.files[p.MEASUREMENTS+event[stream]] = b''
                        events.append(event)
        summary = p.estimate([(1.,2.)]*12, 100, 0)
        rows = [{'family':family,'baseline':baseline,'candidate':candidate,'phase':phase,
                 'analysis':summary, 'observation':'slower-on-this-corpus'}
                for family,baseline,candidate in p.COMPARISONS for phase in p.PHASES]
        cls.report = {'schema':1,'status':'completed','performance_admission':'not granted by this runner',
                      'pairs':12,'records':events,'summaries':rows,'compiler_sha256':hashes}
        cls.seal(cls.files, cls.report)

    @staticmethod
    def seal(files, report):
        raw = b''.join((json.dumps(x)+'\n').encode() for x in report['records'])
        report['raw_sha256'] = p.sha(raw)
        files[p.MEASUREMENTS+'raw.jsonl'] = raw
        files[p.MEASUREMENTS+'report.json'] = json.dumps(report).encode()

    def modified(self):
        return dict(self.files), p.decode(json.dumps(self.report))

    def test_complete_matrix_replays(self):
        report, pairs, source = p.validate(self.files,self.campaign)
        self.assertEqual(len(pairs),324)
        self.assertEqual(source[0], 'b'*40)
        self.assertEqual(report['pairs'],12)

    def test_missing_sample_fails(self):
        files, report = self.modified()
        report['records'].pop()
        self.seal(files,report)
        with self.assertRaises(ValueError): p.validate(files,self.campaign)

    def test_duplicate_sample_fails(self):
        files, report = self.modified()
        report['records'].append(report['records'][0])
        self.seal(files,report)
        with self.assertRaises(ValueError): p.validate(files,self.campaign)

    def test_corrupt_command_output_fails(self):
        files, report = self.modified()
        files[p.MEASUREMENTS+report['records'][0]['stdout']] = b'changed'
        with self.assertRaises(ValueError): p.validate(files,self.campaign)

    def test_failed_or_incomplete_campaign_is_not_published(self):
        for value in ('failed','running'):
            files, report = self.modified()
            report['status'] = value
            self.seal(files,report)
            with self.assertRaises(ValueError): p.validate(files,self.campaign)

    def test_changed_paired_source_fails(self):
        files, report = self.modified()
        report['records'][1]['source_sha256'] = {'main.rs':'0'*64}
        report['records'][1]['source_after_sha256'] = {'main.rs':'0'*64}
        self.seal(files,report)
        with self.assertRaises(ValueError): p.validate(files,self.campaign)

    def test_wrong_source_revision_fails(self):
        with self.assertRaises(ValueError): p.validate(self.files,dict(self.campaign,checkout_sha='0'*40))

    def test_summary_or_label_cannot_hide_regression(self):
        for field,value in [('observation','faster-on-this-corpus'),('phase','invented')]:
            files, report = self.modified()
            report['summaries'][0][field] = value
            self.seal(files,report)
            with self.assertRaises(ValueError): p.validate(files,self.campaign)

    def test_unchanged_build_receipt_required(self):
        files = dict(self.files)
        files[p.MEASUREMENTS+'COMPONENTS.txt'] = b'other components'
        with self.assertRaises(ValueError): p.validate(files,self.campaign)

    def test_stale_generated_document_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'report.md'
            p.write_or_check(path,'expected',False)
            p.write_or_check(path,'expected',True)
            with self.assertRaises(ValueError): p.write_or_check(path,'invented',True)


if __name__ == '__main__':
    unittest.main(verbosity=2)
