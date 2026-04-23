"""Sphinx configuration for mixed example API documentation."""

import sys
from pathlib import Path

# Add the API docs directory to Python path so Sphinx can find the extension
sys.path.insert(0, str(Path(__file__).parent / "api"))

# Project information
project = "mixed"
copyright = "2024, pyo3-stub-gen"
author = "pyo3-stub-gen"

# Extensions
extensions = [
    "pyo3_stub_gen_ext",  # Our generated extension
    "sphinx.ext.intersphinx",  # For cross-references
]

# Intersphinx mapping - enables cross-references to external projects
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "numpy": ("https://numpy.org/doc/stable/", None),
}

# HTML theme
html_theme = "sphinx_rtd_theme"
html_theme_options = {
    "collapse_navigation": False,  # Keep navigation expanded
    "navigation_depth": -1,  # Unlimited navigation depth
}
