#!/usr/bin/env python3
"""Sample explicit, independently attributed PIDs; never infer XPC ownership from PPID."""
import argparse
import json
import subprocess
import time
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--pids', required=True, help='Comma-separated main/WebKit PIDs')
parser.add_argument('--seconds', type=int, default=300)
parser.add_argument('--interval', type=int, default=5)
parser.add_argument('--condition', required=True, help='App version, scene and attribution evidence')
parser.add_argument('--output', required=True)
args = parser.parse_args()
pids = [int(p) for p in args.pids.split(',')]
if args.seconds < 1 or args.interval < 1 or not pids:
    parser.error('Durations and PIDs must be positive')

def cpu_seconds(value):
    parts = [float(p) for p in value.split(':')]
    return sum(p * 60 ** i for i, p in enumerate(reversed(parts)))

def snapshot():
    result = subprocess.run(['ps', '-p', ','.join(map(str, pids)), '-o', 'pid=,rss=,time=,comm='], capture_output=True, text=True, check=True)
    rows = []
    for line in result.stdout.splitlines():
        pid, rss, cpu, name = line.split(None, 3)
        rows.append(dict(pid=int(pid), rssKiB=int(rss), cpuSeconds=cpu_seconds(cpu), name=name))
    if {p['pid'] for p in rows} != set(pids):
        raise RuntimeError('A selected process exited; measurement is incomplete')
    return dict(elapsedSeconds=time.monotonic() - started, processes=rows,
                sumRssMiB=sum(p['rssKiB'] for p in rows) / 1024)

started = time.monotonic()
samples = [snapshot()]
while time.monotonic() - started < args.seconds:
    time.sleep(min(args.interval, max(0, args.seconds - (time.monotonic() - started))))
    samples.append(snapshot())
elapsed = samples[-1]['elapsedSeconds'] - samples[0]['elapsedSeconds']
cpu = sum(p['cpuSeconds'] for p in samples[-1]['processes']) - sum(p['cpuSeconds'] for p in samples[0]['processes'])
summary = dict(condition=args.condition, elapsedSeconds=elapsed, singleCoreCpuPercent=cpu / elapsed * 100,
               minSumRssMiB=min(s['sumRssMiB'] for s in samples), maxSumRssMiB=max(s['sumRssMiB'] for s in samples),
               caveat='RSS sums may double-count shared pages; selected PIDs need independent ownership evidence.')
Path(args.output).write_text(json.dumps(dict(summary=summary, samples=samples), ensure_ascii=False, indent=2) + '\n')
print(json.dumps(summary, ensure_ascii=False))
