import inspect
import pydoc

import generate_init_py


def test_module_docstring():
    assert inspect.getdoc(generate_init_py) == (
        "This is the main module docstring for generate_init_py.\n"
        "This example demonstrates the generate-init-py feature.\n"
        "\n"
        "    This must be indented by four spaces."
    )


def test_module_help_includes_docstring():
    help_text = pydoc.render_doc(generate_init_py, renderer=pydoc.TextDoc())
    assert "This is the main module docstring for generate_init_py." in help_text
