#!/bin/bash
python -m venv .env
source .env/bin/activate
pip install maturin
maturin develop
