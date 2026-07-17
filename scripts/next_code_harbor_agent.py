#!/usr/bin/env python3
"""Compat re-export — prefer this name; `jcode_harbor_agent.py` kept for one release."""
import runpy
if __name__ == "__main__":
    runpy.run_module("jcode_harbor_agent", run_name="__main__")
else:
    from jcode_harbor_agent import *  # noqa: F401,F403
