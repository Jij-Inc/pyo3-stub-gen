"""External docs are shared by runtime objects and the committed generated outputs."""

import ast
import doctest
import inspect
import json
from pathlib import Path

import pure


EXAMPLE_ROOT = Path(__file__).resolve().parents[1]
EXTERNAL_DOC = (EXAMPLE_ROOT / "src/docs/external.md").read_text()


def test_runtime_docstrings():
    assert pure.DocumentedClass.__doc__ == (
        "Inline comment.\nLiteral attribute.\n"
        f"{EXTERNAL_DOC}\nConcatenated text.\nFinal comment."
    )
    assert pure.DocumentedClass.answer.__doc__ == EXTERNAL_DOC
    assert pure.documented_answer.__doc__ == (
        f"Free function docs.\n\n{EXTERNAL_DOC}"
        "\n    assert_eq!(6 * 7, 42);\nLast macro."
    )
    assert getattr(pure.DocumentedClass, "value").__doc__ == EXTERNAL_DOC
    assert getattr(pure.DocumentedClass, "computed").__doc__ == "Getter docs."
    assert pure.NumberRenameAll.__doc__ == EXTERNAL_DOC
    assert pure.DocumentedComplexEnum.__doc__ == EXTERNAL_DOC


def test_generated_docstrings_match_runtime():
    # CI regenerates these outputs and checks for a clean diff. Compare their
    # effective docs with the extension as well, so all three surfaces agree.
    tree = ast.parse((EXAMPLE_ROOT / "pure.pyi").read_text())
    cls = next(
        node
        for node in tree.body
        if isinstance(node, ast.ClassDef) and node.name == "DocumentedClass"
    )
    method = next(
        node
        for node in cls.body
        if isinstance(node, ast.FunctionDef) and node.name == "answer"
    )
    function = next(
        node
        for node in tree.body
        if isinstance(node, ast.FunctionDef) and node.name == "documented_answer"
    )
    package = json.loads((EXAMPLE_ROOT / "docs/api/api_reference.json").read_text())
    items = package["modules"]["pure"]["items"]
    class_info = next(item for item in items if item["name"] == "DocumentedClass")
    method_info = next(
        item for item in class_info["methods"] if item["name"] == "answer"
    )
    function_info = next(item for item in items if item["name"] == "documented_answer")

    for node, info, runtime_doc in (
        (cls, class_info, pure.DocumentedClass.__doc__),
        (method, method_info, pure.DocumentedClass.answer.__doc__),
        (function, function_info, pure.documented_answer.__doc__),
    ):
        assert runtime_doc is not None
        assert ast.get_docstring(node) == info["doc"] == inspect.cleandoc(runtime_doc)


def test_external_markdown_doctest():
    doc = pure.DocumentedClass.answer.__doc__
    assert doc is not None
    test = doctest.DocTestParser().get_doctest(doc, {}, "answer", "external.md", 0)
    result = doctest.DocTestRunner().run(test)
    assert result.failed == 0
    assert result.attempted == 2
