"""Negative controls for P2.5 archive inspection, independent of native builds."""
import base64
import hashlib
import io
import json
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from p2_packages import audit_npm, audit_wheel


ELF = b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(44)


class PackageControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def npm(self, extra=None, target='linux', binary=ELF):
        files = dict(zip(('package.json', 'index.cjs', 'index.d.ts', 'wyc_lib.node', 'README.md'),
                         (json.dumps(dict(name='wyc-lib', version='0.1.0', os=[target], cpu=['x64'],
                                          engines=dict(node='22.22.0'))).encode(), b'wrapper', b'types', binary, b'readme')))
        if extra:
            files.update(extra)
        archive = self.root / 'package.tgz'
        with tarfile.open(archive, 'w:gz') as package:
            for name, data in files.items():
                member = tarfile.TarInfo('package/' + name)
                member.size = len(data)
                package.addfile(member, io.BytesIO(data))
        return archive

    def wheel(self, corrupt=False):
        dist = 'wyc_lib-0.1.0.dist-info/'
        files = {'wyc_lib/__init__.py': b'wrapper', 'wyc_lib/__init__.pyi': b'types',
                 'wyc_lib/_conversion.py': b'conversion', 'wyc_lib/py.typed': b'',
                 'wyc_lib/_native.cpython-310-x86_64-linux-gnu.so': ELF,
                 dist + 'METADATA': b'Name: wyc-lib\nVersion: 0.1.0\nRequires-Python: >=3.10,<3.11\n',
                 dist + 'WHEEL': b'Root-Is-Purelib: false\nTag: cp310-cp310-linux_x86_64\n'}
        rows = []
        for name, data in files.items():
            digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode()
            rows.append(f'{name},sha256={digest},{len(data)}\n')
        rows.append(f'{dist}RECORD,,\n')
        files[dist + 'RECORD'] = ''.join(rows).encode()
        if corrupt:
            files['wyc_lib/__init__.py'] = b'changed after RECORD'
        archive = self.root / 'wyc_lib-0.1.0-cp310-cp310-linux_x86_64.whl'
        with zipfile.ZipFile(archive, 'w') as package:
            for name, data in files.items():
                package.writestr(name, data)
        return archive

    def test_expected_archives(self):
        self.assertEqual(audit_npm(self.npm(), 'linux')['cpu'], 'x64')
        self.assertTrue(audit_wheel(self.wheel(), 'linux')['crc_verified'])

    def test_reject_extra_or_traversal_members(self):
        for name in ('secret.env', '../escape', 'src/lib.rs'):
            with self.assertRaises(ValueError):
                audit_npm(self.npm({name: b'forbidden'}), 'linux')

    def test_reject_wrong_platform(self):
        with self.assertRaises(ValueError):
            audit_npm(self.npm(target='win32'), 'linux')
        with self.assertRaises(ValueError):
            audit_wheel(self.wheel(), 'win32')

    def test_reject_wrong_binary(self):
        with self.assertRaises(ValueError):
            audit_npm(self.npm(binary=b'not a binary'), 'linux')

    def test_reject_record_tampering(self):
        with self.assertRaises(ValueError):
            audit_wheel(self.wheel(corrupt=True), 'linux')


if __name__ == '__main__':
    unittest.main()
