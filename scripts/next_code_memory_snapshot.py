#!/usr/bin/env python3
"""Compat re-export — prefer this name; `jcode_memory_snapshot.py` kept for one release."""
import runpy
if __name__ == "__main__":
    runpy.run_module("jcode_memory_snapshot", run_name="__main__")
else:
    from jcode_memory_snapshot import *  # noqa: F401,F403
