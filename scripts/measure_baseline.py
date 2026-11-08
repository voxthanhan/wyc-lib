"""Measure a built Rust baseline binary; standard library only, no credentials.

Build separately so CPU/RSS exclude the compiler. RSS is process-level, not heap
or retained-state bytes. CPU includes workload setup/warmups/JSON and timing.
"""
import ctypes
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path


def main():
    binary, output = map(Path, sys.argv[1:3])
    started = time.perf_counter()
    # Files avoid pipe deadlock while the parent samples process memory.
    raw = output.with_suffix('.raw.json')
    err = output.with_suffix('.stderr.txt')
    output.parent.mkdir(parents=True, exist_ok=True)
    with raw.open('w', encoding='utf-8') as stdout, err.open('w', encoding='utf-8') as stderr:
        process = subprocess.Popen([str(binary.resolve())], stdout=stdout, stderr=stderr)
        peak = 0
        cpu = None
        if os.name == 'nt':
            from ctypes import wintypes

            class Counters(ctypes.Structure):
                _fields_ = [('cb', wintypes.DWORD), ('PageFaultCount', wintypes.DWORD)] + [
                    (name, ctypes.c_size_t) for name in ['PeakWorkingSetSize', 'WorkingSetSize',
                        'QuotaPeakPagedPoolUsage', 'QuotaPagedPoolUsage',
                        'QuotaPeakNonPagedPoolUsage', 'QuotaNonPagedPoolUsage',
                        'PagefileUsage', 'PeakPagefileUsage']]

            kernel = ctypes.WinDLL('kernel32', use_last_error=True)
            psapi = ctypes.WinDLL('psapi', use_last_error=True)
            kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            kernel.OpenProcess.restype = wintypes.HANDLE
            kernel.CloseHandle.argtypes = [wintypes.HANDLE]
            kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
            psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
            handle = kernel.OpenProcess(0x410, False, process.pid)
            if not handle:
                process.wait()
                raise ctypes.WinError(ctypes.get_last_error())
            try:
                while process.poll() is None:
                    counters = Counters()
                    counters.cb = ctypes.sizeof(counters)
                    if psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
                        peak = max(peak, counters.PeakWorkingSetSize)
                    time.sleep(0.01)
                times = [wintypes.FILETIME() for _ in range(4)]
                if not kernel.GetProcessTimes(handle, *(ctypes.byref(t) for t in times)):
                    raise ctypes.WinError(ctypes.get_last_error())
                cpu = sum((t.dwHighDateTime << 32) + t.dwLowDateTime for t in times[2:]) / 1e7
            finally:
                kernel.CloseHandle(handle)
            memory_method = 'GetProcessMemoryInfo PeakWorkingSetSize sampled every 10ms; may miss final interval'
        else:
            import resource
            process.wait()
            usage = resource.getrusage(resource.RUSAGE_CHILDREN)
            cpu = usage.ru_utime + usage.ru_stime
            peak = usage.ru_maxrss * (1 if sys.platform == 'darwin' else 1024)
            memory_method = 'getrusage RUSAGE_CHILDREN ru_maxrss; one child, excludes compiler'
    if process.returncode:
        raise RuntimeError(f'baseline failed ({process.returncode}): {err.read_text()}')
    report = json.loads(raw.read_text(encoding='utf-8'))
    report['process'] = dict(platform=platform.platform(), cpu_model=platform.processor(),
        logical_cpu_count=os.cpu_count(), wall_seconds=time.perf_counter()-started,
        cpu_seconds=cpu, peak_rss_bytes=peak or None, memory_method=memory_method,
        scope='whole benchmark process incl. setup/warmups/output; not per-operation heap/state size')
    report['commit'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    report['binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
    report['clean'] = not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=normal'])
    report['rustc'] = subprocess.check_output(['rustc', '-Vv'], text=True).strip()
    output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(f'PASS: {len(report["rows"])} workloads, five samples each; CPU={cpu:.3f}s peak RSS={peak} bytes')


if __name__ == '__main__':
    main()
