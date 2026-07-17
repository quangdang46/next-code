#!/usr/bin/env python3
"""Compat re-export — prefer this name; `jcode_monitor.py` kept for one release."""
import runpy
if __name__ == "__main__":
    runpy.run_module("jcode_monitor", run_name="__main__")
else:
    from jcode_monitor import *  # noqa: F401,F403
