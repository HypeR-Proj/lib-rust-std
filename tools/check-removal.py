#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 roolrz
# SPDX-License-Identifier: Apache-2.0

"""Test the platform removal policy without a booted kernel."""

import argparse
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
binary = args.output.resolve() / 'removal-tests'
source = Path(__file__).resolve().parents[1] / 'tests/removal.rs'
subprocess.run(['rustc', '--edition=2024', '--test', str(source), '-o', str(binary)], check=True)
subprocess.run([str(binary)], check=True)
