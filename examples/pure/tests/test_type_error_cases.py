import subprocess
import json
from pathlib import Path
from typing import List
from pydantic import BaseModel


class FilePosition(BaseModel):
    line: int
    character: int


class FileRange(BaseModel):
    start: FilePosition
    end: FilePosition


class Diagnostic(BaseModel):
    # File name is omitted to avoid environment-specific paths
    # Message are managed separately in the snapshot tests
    severity: str
    rule: str
    range: FileRange


def call_pyright(input: Path) -> List[tuple[str, Diagnostic]]:
    result = subprocess.run(
        ["pyright", "--outputjson", input], capture_output=True, text=True
    )
    output = json.loads(result.stdout)
    diagnostics = []
    for diag in output.get("generalDiagnostics", []):
        message = diag["message"]
        del diag["file"]
        del diag["message"]
        diagnostics.append((message, Diagnostic(**diag)))
    return diagnostics


def test_pyright_type_errors(snapshot):
    path = Path(__file__).parent / "type_error_cases" / "numpy_ndarray.py"
    diagnostics = call_pyright(path)
    for message, meta in diagnostics:
        assert snapshot() == message
        assert snapshot("json") == meta.model_dump()
