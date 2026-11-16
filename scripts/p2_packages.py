"""Audit target-specific P2.5 archives before installation; no extraction."""
import base64
import csv
import hashlib
import io
import json
import struct
import tarfile
import zipfile
from email.parser import Parser


def require(condition, message):
    if not condition:
        raise ValueError(message)


def binary_target(data, target):
    if target == 'win32':
        require(data[:2] == b'MZ' and len(data) >= 64, 'Expected Windows PE binary')
        offset = struct.unpack_from('<I', data, 60)[0]
        require(data[offset:offset + 4] == b'PE\0\0', 'Invalid PE signature')
        require(data[offset + 4:offset + 6] == b'\x64\x86', 'Expected x64 PE')
    else:
        require(data[:6] == b'\x7fELF\x02\x01' and data[18:20] == b'\x3e\x00',
                'Expected little-endian x64 ELF binary')


def audit_npm(archive, target):
    with tarfile.open(archive, 'r:gz') as package:
        members = package.getmembers()
        names = [m.name for m in members]
        expected = {'package/package.json', 'package/index.cjs', 'package/index.d.ts',
                    'package/wyc_lib.node', 'package/README.md'}
        require(len(names) == len(set(names)) and set(names) == expected,
                'npm archive must contain only public wrapper/types/README/binary/manifest')
        require(all(m.isfile() for m in members), 'No links or special files in npm archive')
        content = {m.name: package.extractfile(m).read() for m in members}
    metadata = json.loads(content['package/package.json'])
    require(metadata['name'] == 'wyc-lib' and metadata['version'] == '0.1.0', 'npm identity')
    require(metadata.get('os') == [target] and metadata.get('cpu') == ['x64'], 'npm target guard')
    require(metadata.get('engines') == {'node': '22.22.0'}, 'npm Node version guard')
    require(not metadata.get('scripts') and not metadata.get('dependencies'), 'npm install must be offline')
    binary_target(content['package/wyc_lib.node'], target)
    return dict(files=sorted(names), os=target, cpu='x64', node='22.22.0')


def audit_wheel(archive, target):
    tag = 'cp310-cp310-' + ('win_amd64' if target == 'win32' else 'linux_x86_64')
    require(archive.name == f'wyc_lib-0.1.0-{tag}.whl', 'Unexpected wheel filename/ABI/platform')
    with zipfile.ZipFile(archive) as package:
        require(package.testzip() is None, 'wheel CRC failure')
        names = package.namelist()
        require(len(names) == len(set(names)), 'Duplicate wheel members')
        native = [n for n in names if n.startswith('wyc_lib/_native.') and n.endswith(('.so', '.pyd'))]
        require(len(native) == 1, 'Expected one native extension')
        dist = 'wyc_lib-0.1.0.dist-info/'
        expected = {'wyc_lib/__init__.py', 'wyc_lib/__init__.pyi', 'wyc_lib/_conversion.py',
                    'wyc_lib/py.typed', native[0], dist + 'METADATA', dist + 'WHEEL', dist + 'RECORD'}
        require(set(names) == expected, 'Unexpected wheel contents')
        require(all(not (i.external_attr >> 16) & 0o170000 == 0o120000
                    for i in package.infolist()), 'No wheel symlinks')
        metadata = Parser().parsestr(package.read(dist + 'METADATA').decode())
        require(metadata['Name'] == 'wyc-lib' and metadata['Version'] == '0.1.0', 'wheel identity')
        require(set(metadata['Requires-Python'].replace(' ', '').split(',')) == {'>=3.10', '<3.11'},
                'wheel Python version guard')
        wheel = Parser().parsestr(package.read(dist + 'WHEEL').decode())
        require(wheel.get_all('Tag') == [tag] and wheel['Root-Is-Purelib'] == 'false', 'wheel ABI tag')
        records = list(csv.reader(io.StringIO(package.read(dist + 'RECORD').decode())))
        require(len(records) == len(names) and {r[0] for r in records} == set(names), 'wheel RECORD coverage')
        for name, digest, size in records:
            if name == dist + 'RECORD':
                require(not digest and not size, 'RECORD self entry')
                continue
            data = package.read(name)
            actual = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode()
            require(digest == 'sha256=' + actual and int(size) == len(data), 'wheel RECORD checksum: ' + name)
        binary_target(package.read(native[0]), target)
    return dict(files=sorted(names), tag=tag, record_sha256_verified=True, crc_verified=True)
