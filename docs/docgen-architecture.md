# Documentation Generation Architecture

## Overview

pyo3-stub-gen includes a documentation generation feature that creates Sphinx-compatible API reference documentation directly from the same type metadata used for stub generation. This addresses limitations in existing tools (sphinx-autodoc2, mkdocstrings) that cannot properly handle:

- `@overload`-only functions with all signature variants
- Type alias definitions with docstrings
- Type alias preservation (not expanded)
- Re-export handling with correct link resolution
- Links to re-exported items pointing to their public module

## Architecture: Pattern C - Direct Sphinx Doctree Construction

The documentation generation follows a two-stage architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│ Stage 1: Rust - JSON IR Generation                              │
│ (cargo run --bin stub_gen)                                      │
│                                                                 │
│  StubInfo (from stub generation)                                │
│       ↓                                                         │
│  DocPackageBuilder                                              │
│       ↓                                                         │
│  DocPackage (Intermediate Representation)                       │
│       ↓                                                         │
│  JSON serialization → api_reference.json                        │
│                                                                 │
│  Also generates:                                                │
│  - pyo3_stub_gen_ext.py (Sphinx extension)                     │
│  - *.rst files (module pages, when `separate-pages = true`)    │
│  - index.rst (table of contents, when `separate-pages = true`) │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Stage 2: Python - Sphinx Doctree Construction                   │
│ (sphinx-build docs docs/_build)                                 │
│                                                                 │
│  pyo3_stub_gen_ext.py (Sphinx extension)                       │
│       ↓                                                         │
│  Reads api_reference.json                                       │
│       ↓                                                         │
│  Constructs Sphinx doctree nodes directly                       │
│       ↓                                                         │
│  Sphinx renders to HTML/PDF/etc.                               │
└─────────────────────────────────────────────────────────────────┘
```

Typical workflow:
```bash
cargo run --bin stub_gen        # Stage 1: Generate JSON, RST, and extension
sphinx-build docs docs/_build   # Stage 2: Build HTML documentation
```

### Why Pattern C?

Alternative approaches were considered:

- **Pattern A (RST Generation)**: Generate `.rst` files with Sphinx directives. Limited control over linking and formatting.
- **Pattern B (Sphinx Domain)**: Create a custom Sphinx domain. High complexity, maintenance burden.
- **Pattern C (Doctree Construction)**: Generate JSON IR, construct doctree nodes in Python. Best balance of control and simplicity.

Pattern C was chosen because:
1. Full control over link targets and display text
2. Reuses Sphinx's existing Python domain for cross-references
3. JSON IR is debuggable and can be used by other tools
4. Sphinx extension is self-contained and portable

## JSON Intermediate Representation

### Schema Overview

```
DocPackage
├── name: String                    # Package name
├── modules: Map<String, DocModule> # Module FQN → DocModule
├── export_map: Map<String, String> # Item FQN → Export module
└── config: DocGenConfig            # Generation settings

DocModule
├── name: String                    # Module FQN
├── doc: String                     # Module docstring (MyST markdown)
├── items: Vec<DocItem>             # Functions, classes, etc.
└── submodules: Vec<String>         # Submodule names

DocItem (tagged enum)
├── Function(DocFunction)
├── Class(DocClass)
├── TypeAlias(DocTypeAlias)
├── Variable(DocVariable)
└── Module(DocSubmodule)
```

### Type Expression Design

A key design decision is the separation of **display text** and **link targets** in type expressions:

```
DocTypeExpr
├── display: String                 # What to show: "Optional[MyClass]"
├── link_target: Option<LinkTarget> # Where to link: module.MyClass
└── children: Vec<DocTypeExpr>      # Generic parameters (recursive)

LinkTarget
├── fqn: String                     # Fully qualified name
├── doc_module: String              # Module where documented
├── kind: ItemKind                  # Currently Class for type links
└── attribute: Option<String>       # For enum variants
```

This separation enables:
- **Short display**: Show `MyClass` instead of `package.submodule.MyClass`
- **Accurate links**: Link to the correct module where the item is exported
- **Nested linking**: Each generic parameter can have its own link

Example:
```json
{
  "display": "Optional[MyClass]",
  "link_target": null,
  "children": [
    {
      "display": "MyClass",
      "link_target": {
        "fqn": "mypackage.MyClass",
        "doc_module": "mypackage",
        "kind": "Class"
      },
      "children": []
    }
  ]
}
```

## Link Resolution: Haddock-Style Rules (Current Status)

The link resolver (`LinkResolver::resolve_link`) follows rules inspired by Haskell's Haddock documentation tool:

1. **If exported from current module**: Link to current module
2. **Else if exported from a public module**: Link to that module
3. **Else (private module only)**: No link

In the current implementation:
- `resolve_link` is used for class-attribute links in default values (e.g. `C.C1`)
- Type links are primarily built by `TypeRenderer` from `type_refs` + `export_map`
- If a type is not in `export_map`, `TypeRenderer` currently falls back to `current_module` rather than dropping the link

So rule (3) above is not fully enforced yet for general type links.

### Private Module Detection

Modules with any path component starting with `_` are considered private:

```rust
fn is_private_module(module_name: &str) -> bool {
    module_name.split('.').any(|part| part.starts_with('_'))
}
```

Examples:
- `package._internal` → private
- `package._core.submodule` → private
- `package.public` → public

## Module Structure

```
pyo3-stub-gen/src/docgen/
├── mod.rs              # Module exports
├── builder.rs          # DocPackageBuilder - converts StubInfo to DocPackage
├── ir.rs               # Intermediate representation structs
├── types.rs            # Type expression parsing and rendering
├── link.rs             # Haddock-style link resolution
├── export.rs           # Export map building
├── config.rs           # DocGenConfig from pyproject.toml
├── default_parser.rs   # Default value parsing with type refs
├── render.rs           # JSON output and RST generation
├── util.rs             # Prefix stripping utilities
└── sphinx_ext.py       # Embedded Sphinx extension
```

### Data Flow

```
StubInfo
    │
    ↓ DocPackageBuilder::new()
    │
ExportResolver
    │
    ├── Analyzes __all__ exports
    └── Builds export_map (FQN → module)
    │
    ↓ DocPackageBuilder::build()
    │
DocPackage
    │
    ├── For each public module:
    │   ├── Build DocFunction (with all overloads)
    │   ├── Build DocClass (with methods, attributes)
    │   ├── Build DocTypeAlias
    │   └── Build DocVariable
    │
    ├── Correct link targets using export_map
    └── (Normalization happens later in render_to_json())
    │
    ↓ render_to_json()
    │
api_reference.json
```

## Sphinx Extension

The generated `pyo3_stub_gen_ext.py` provides two directives:

### `pyo3-api` Directive

Renders a single module's API:

```rst
.. pyo3-api:: mypackage.submodule
```

### `pyo3-api-package` Directive

Renders all modules in a package:

```rst
.. pyo3-api-package:: mypackage
```

### Features

- **Intersphinx support**: External types (`typing.Optional`, `collections.abc.Callable`) link via intersphinx
- **Python domain integration**: Registers objects with `py:class`, `py:func`, etc.
- **Index generation**: Creates entries for genindex
- **MyST markdown**: Docstrings are parsed as MyST markdown
- **Module contents table**: Optional summary table at top of each module

## Configuration

Configure in `pyproject.toml`:

```toml
[tool.pyo3-stub-gen.doc-gen]
# Output directory for generated files (relative to pyproject.toml)
output-dir = "docs/api"

# JSON output filename
json-output = "api_reference.json"

# Generate separate .rst pages for each module
separate-pages = true

# Custom title for index.rst (default: "{package} API Reference")
index-title = "API Reference"

# Intro message for index.rst (empty string to skip)
intro-message = "Welcome to the API documentation."

# Show module contents table
contents-table = true
```

Note: `json-output` currently affects where Rust writes the JSON file, but the generated Sphinx extension still reads `api/api_reference.json` (or fallback `api_reference.json`) with a fixed filename.

## Sphinx Project Setup

The docgen feature generates files that work as a Sphinx extension. Users need to set up a Sphinx project to build the final HTML documentation.

### Generated vs User-Provided Files

**Auto-generated** (by `stub_gen` binary, in `output-dir`):
- `api_reference.json` - Structured documentation data
- `pyo3_stub_gen_ext.py` - Sphinx extension (embedded in pyo3-stub-gen, copied at generation time)
- `index.rst` - API reference index with toctree (when `separate-pages = true`)
- `<module>.rst` - One page per module (when `separate-pages = true`)

**User-provided** (in `docs/` or similar):
- `conf.py` - Sphinx configuration (required)
- Top-level `index.rst` - Project documentation root (optional, can reference `api/index`)

### conf.py

```python
import sys
from pathlib import Path

# Add API docs directory to path
sys.path.insert(0, str(Path(__file__).parent / "api"))

extensions = [
    "pyo3_stub_gen_ext",      # Generated extension
    "sphinx.ext.intersphinx", # For external type links
    "myst_parser",            # For MyST markdown in docstrings
]

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}
```

### Dependencies

```toml
[dependency-groups]
dev = ["sphinx", "myst-parser"]
```

## Type Prefix Stripping

Display text is simplified by stripping common prefixes:

| Original | Display |
|----------|---------|
| `typing.Optional[T]` | `Optional[T]` |
| `builtins.str` | `str` |
| `collections.abc.Callable` | `Callable` |
| `mypackage.submodule.MyClass` | `MyClass` |
| `_internal.PrivateClass` | `PrivateClass` |

This is handled by `util::prefix_stripper`:

```rust
// Standard library prefixes
let stdlib_prefixes = ["typing.", "builtins.", "collections.abc.", ...];

// Package prefixes (heuristic: lowercase.Uppercase pattern)
// "main_mod.ClassA" → "ClassA"

// Internal module prefixes
// "_core.Type" → "Type"
```

## Deterministic Output

The `DocPackage::normalize()` method ensures consistent JSON output.
It is currently called from `render_to_json()` before serialization:

1. Sort items by kind priority, then by name
2. Sort methods and attributes alphabetically
3. Sort signatures by parameter count

This enables `git diff --exit-code` checks in CI to verify generated docs are up-to-date.

## Related Documentation

- [Architecture](./architecture.md) - Overall pyo3-stub-gen architecture
- [Type System](./type-system.md) - Rust to Python type mappings
- [Stub File Generation](./stub-file-generation.md) - Stub file output rules
