"""Check the interpreter and the Python-version-specific example in CI."""

import importlib.util
import os
import sys
from pathlib import Path

expected = tuple(map(int, os.environ["EXPECTED_PYTHON_VERSION"].split(".")))
assert sys.version_info[:2] == expected, (sys.version_info[:2], expected)

# PyO3 embeds this interpreter in the stub generator. It must match pytest.
pyo3_python = Path(os.environ["PYO3_PYTHON"]).resolve()
python = Path(sys.executable).resolve()
assert pyo3_python == python, (pyo3_python, python)

has_type_statements = importlib.util.find_spec("type_statement_alias") is not None
assert has_type_statements == (expected >= (3, 12)), (expected, has_type_statements)
print(f"Python {expected}: type-statement-alias enabled={has_type_statements}")
