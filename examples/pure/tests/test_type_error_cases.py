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


ERROR_CASES_DIR = Path(__file__).parent / "type_error_cases"


def call_pyright_error_case(input: Path) -> List[tuple[str, Diagnostic]]:
    result = subprocess.run(
        [
            "pyright",
            "--outputjson",
            "--project",  # override ignore settings in pyproject.toml
            ERROR_CASES_DIR / "pyrightconfig.json",
            input,
        ],
        capture_output=True,
        text=True,
    )
    assert result.returncode != 0, "Expected pyright to report type errors"
    output = json.loads(result.stdout)
    diagnostics = []
    for diag in output.get("generalDiagnostics", []):
        message = diag["message"]
        del diag["file"]
        del diag["message"]
        diagnostics.append((message, Diagnostic(**diag)))
    return diagnostics


def test_pyright_type_errors(snapshot):
    for case in [p for p in ERROR_CASES_DIR.iterdir() if p.suffix == ".py"]:
        diagnostics = call_pyright_error_case(case)
        for i, (message, meta) in enumerate(diagnostics):
            assert snapshot(f"{case.stem}__{i}.txt") == message
            assert snapshot(f"{case.stem}__{i}.json") == meta.model_dump()
