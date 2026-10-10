#!/bin/sh
# Opens the accent workbench: a page of knobs and sliders, on this computer only.
# The first run sets itself up (a few minutes, about 350 MB of voice models).
set -e
cd "$(dirname "$0")"
[ -x .venv/bin/python ] || python3 -m venv .venv
.venv/bin/python -m pip install -q --disable-pip-version-check -r requirements.txt
.venv/bin/python voice.py setup --studio
exec .venv/bin/python voice.py studio "$@"
