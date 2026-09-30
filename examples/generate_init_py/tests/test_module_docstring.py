import inspect
import pydoc

import generate_init_py


def test_module_docstring():
    assert inspect.getdoc(generate_init_py) == (
        "This is the main module docstring for generate_init_py.\n"
        "This example demonstrates the generate-init-py feature.\n"
        "\n"
        "    This must be indented by four spaces.\n"
        "\n"
        "Literal Python strings:\n"
        "\n"
        '    text = """A triple-quoted string"""\n'
        '    path = r"C:\\new\\tools"\n'
        '    pattern = r"\\d+\\s"\n'
        "    continued = \\\n"
        '        "next line"'
    )


def test_module_help_includes_docstring():
    help_text = pydoc.render_doc(generate_init_py, renderer=pydoc.TextDoc())
    assert "This is the main module docstring for generate_init_py." in help_text
    assert 'text = """A triple-quoted string"""' in help_text
    assert r'path = r"C:\new\tools"' in help_text
