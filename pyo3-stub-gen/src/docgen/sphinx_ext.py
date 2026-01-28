"""Sphinx extension for pyo3-stub-gen API documentation"""

import json
import re
from pathlib import Path
from docutils import nodes
from sphinx.addnodes import (
    desc, desc_signature, desc_name, desc_parameterlist,
    desc_parameter, desc_returns, pending_xref, desc_content
)
from sphinx.util.docutils import SphinxDirective

# Helper functions for building documentation nodes

def _parse_and_link_type(type_str):
    """Parse type string and create intersphinx links for external types"""
    # Known external modules that should be linked via intersphinx
    EXTERNAL_MODULES = {
        'builtins', 'typing', 'collections', 'collections.abc',
        'typing_extensions', 'decimal', 'datetime', 'pathlib',
        'numpy', 'numpy.typing'
    }

    # Special constants
    SPECIAL_CONSTANTS = {'None': 'constants', 'True': 'constants', 'False': 'constants'}

    # Recursively parse and link types
    def parse_recursive(s):
        result = []
        i = 0

        while i < len(s):
            # Skip whitespace
            if s[i].isspace():
                result.append(nodes.Text(s[i]))
                i += 1
                continue

            # Handle brackets and special chars
            if s[i] in '[](),|':
                result.append(nodes.Text(s[i]))
                i += 1
                continue

            # Try to match a qualified name or identifier
            match = re.match(r'([a-zA-Z_][a-zA-Z0-9_.]*)', s[i:])
            if match:
                name = match.group(1)
                i += len(name)

                # Check for special constants (not in intersphinx, render as text)
                if name in SPECIAL_CONSTANTS:
                    result.append(nodes.Text(name))
                    continue

                # Check if it's a qualified external type
                if '.' in name:
                    parts = name.split('.')
                    # Try to find which part is the module
                    for k in range(len(parts), 0, -1):
                        module = '.'.join(parts[:k])
                        if module in EXTERNAL_MODULES:
                            # Skip builtins - they're not in intersphinx inventory
                            if module == 'builtins':
                                result.append(nodes.Text(name))
                                break

                            # Determine reftype based on module and type name
                            type_name = parts[-1]
                            if module == 'typing':
                                # Check if it's py:data or py:class in typing module
                                if type_name in ('Any', 'Optional', 'Literal', 'LiteralString',
                                                 'AnyStr', 'NoReturn', 'Never', 'Self',
                                                 'TypeAlias', 'ClassVar', 'Final'):
                                    reftype = 'data'
                                else:
                                    reftype = 'class'
                            else:
                                reftype = 'class'

                            xref = pending_xref(
                                '',
                                refdomain='py',
                                reftype=reftype,
                                reftarget=name,
                                refexplicit=False,
                            )
                            xref += nodes.literal(text=name)
                            result.append(xref)
                            break
                    else:
                        # Not an external type
                        result.append(nodes.Text(name))
                # Check for bare builtins (not in intersphinx, render as text)
                elif name in ('int', 'str', 'float', 'bool', 'bytes', 'list', 'dict',
                              'tuple', 'set', 'frozenset', 'type', 'object', 'complex'):
                    # Bare builtins are not in Python's intersphinx inventory
                    result.append(nodes.Text(name))
                # Check for bare typing types (mixed py:class and py:data)
                elif name in ('Any', 'Optional', 'Literal', 'LiteralString', 'AnyStr',
                              'NoReturn', 'Never', 'Self', 'TypeAlias', 'ClassVar', 'Final'):
                    # These are py:data in Python's intersphinx
                    xref = pending_xref(
                        '',
                        refdomain='py',
                        reftype='data',
                        reftarget=f'typing.{name}',
                        refexplicit=False,
                    )
                    xref += nodes.Text(name)
                    result.append(xref)
                elif name in ('Union', 'TypeVar', 'Generic', 'Protocol'):
                    # These are py:class in Python's intersphinx
                    xref = pending_xref(
                        '',
                        refdomain='py',
                        reftype='class',
                        reftarget=f'typing.{name}',
                        refexplicit=False,
                    )
                    xref += nodes.Text(name)
                    result.append(xref)
                # Check for bare collections.abc types
                elif name in ('Sequence', 'Mapping', 'Callable', 'Iterable', 'Iterator',
                              'Collection', 'Container', 'MutableSequence', 'MutableMapping'):
                    xref = pending_xref(
                        '',
                        refdomain='py',
                        reftype='class',
                        reftarget=f'collections.abc.{name}',
                        refexplicit=False,
                    )
                    xref += nodes.Text(name)
                    result.append(xref)
                else:
                    # Unknown type, don't link
                    result.append(nodes.Text(name))
            else:
                # Unrecognized character
                result.append(nodes.Text(s[i]))
                i += 1

        return result

    # Parse the type string
    nodes_list = parse_recursive(type_str)

    # Wrap in a container
    container = nodes.inline()
    for node in nodes_list:
        container += node
    return container

def _build_type_expr(type_expr):
    """Build type expression with intersphinx linking for external types"""
    display = type_expr['display']
    link_target = type_expr.get('link_target')

    if link_target:
        # Create pending_xref for our own types
        kind_to_reftype = {
            'Class': 'class',
            'Function': 'func',
            'TypeAlias': 'data',
            'Variable': 'data',
        }
        xref = pending_xref(
            '',
            refdomain='py',
            reftype=kind_to_reftype.get(link_target['kind'], 'obj'),
            reftarget=link_target['fqn'],
            refexplicit=True,
        )
        xref += nodes.literal(text=display)
        return xref
    else:
        # Parse the type expression and create intersphinx links for external types
        return _parse_and_link_type(display)

def _parse_myst(markdown_text):
    """Parse MyST markdown to nodes"""
    # Simplified - just return as paragraph for now
    # Full implementation would use MyST parser
    return nodes.paragraph(text=markdown_text)

def _build_function(env, func, module_name):
    """Build function with all overload signatures"""
    desc_node = desc(domain='py', objtype='function', noindex=False)

    fullname = f"{module_name}.{func['name']}"
    sig_id = fullname

    # Add signature for each overload
    for idx, sig in enumerate(func['signatures']):
        sig_node = desc_signature(module=module_name, fullname=fullname)
        sig_node['module'] = module_name
        sig_node['fullname'] = fullname

        # Add IDs for cross-referencing
        sig_node['ids'].append(sig_id)
        sig_node['first'] = (idx == 0)

        sig_node += desc_name(text=func['name'])

        # Parameters
        param_list = desc_parameterlist()
        for param in sig['parameters']:
            param_node = desc_parameter()
            param_node += nodes.Text(param['name'] + ': ')
            param_node += _build_type_expr(param['type_'])
            if param.get('default'):
                param_node += nodes.Text(' = ' + param['default'])
            param_list += param_node
        sig_node += param_list

        # Return type
        if sig['return_type']:
            returns = desc_returns()
            returns += _build_type_expr(sig['return_type'])
            sig_node += returns

        desc_node += sig_node

    # Docstring (MyST-formatted)
    if func.get('doc'):
        content = desc_content()
        content += _parse_myst(func['doc'])
        desc_node += content

    # Register with Python domain for intersphinx
    if hasattr(env, 'domaindata'):
        py_domain = env.get_domain('py')
        py_domain.note_object(fullname, 'function', sig_id, location=env.docname)

    return [desc_node]

def _build_type_alias(env, alias, module_name):
    """Build type alias documentation"""
    desc_node = desc(domain='py', objtype='data', noindex=False)

    fullname = f"{module_name}.{alias['name']}"
    sig_node = desc_signature(module=module_name, fullname=fullname)
    sig_node['module'] = module_name
    sig_node['fullname'] = fullname
    sig_id = fullname
    sig_node['ids'].append(sig_id)

    sig_node += desc_name(text=alias['name'])
    sig_node += nodes.Text(': TypeAlias = ')
    sig_node += _build_type_expr(alias['definition'])
    desc_node += sig_node

    if alias.get('doc'):
        content = desc_content()
        content += _parse_myst(alias['doc'])
        desc_node += content

    # Register with Python domain
    if hasattr(env, 'domaindata'):
        py_domain = env.get_domain('py')
        py_domain.note_object(fullname, 'data', sig_id, location=env.docname)

    return [desc_node]

def _build_class(env, cls, module_name):
    """Build class documentation"""
    desc_node = desc(domain='py', objtype='class', noindex=False)

    fullname = f"{module_name}.{cls['name']}"
    sig_node = desc_signature(module=module_name, fullname=fullname)
    sig_node['module'] = module_name
    sig_node['fullname'] = fullname
    sig_id = fullname
    sig_node['ids'].append(sig_id)

    sig_node += desc_name(text=cls['name'])
    desc_node += sig_node

    if cls.get('doc'):
        content = desc_content()
        content += _parse_myst(cls['doc'])
        desc_node += content

    # Register with Python domain
    if hasattr(env, 'domaindata'):
        py_domain = env.get_domain('py')
        py_domain.note_object(fullname, 'class', sig_id, location=env.docname)

    return [desc_node]

def _build_variable(env, var, module_name):
    """Build variable documentation"""
    desc_node = desc(domain='py', objtype='data', noindex=False)

    fullname = f"{module_name}.{var['name']}"
    sig_node = desc_signature(module=module_name, fullname=fullname)
    sig_node['module'] = module_name
    sig_node['fullname'] = fullname
    sig_id = fullname
    sig_node['ids'].append(sig_id)

    sig_node += desc_name(text=var['name'])
    if var.get('type_'):
        sig_node += nodes.Text(': ')
        sig_node += _build_type_expr(var['type_'])
    desc_node += sig_node

    if var.get('doc'):
        content = desc_content()
        content += _parse_myst(var['doc'])
        desc_node += content

    # Register with Python domain
    if hasattr(env, 'domaindata'):
        py_domain = env.get_domain('py')
        py_domain.note_object(fullname, 'data', sig_id, location=env.docname)

    return [desc_node]

class Pyo3APIDirective(SphinxDirective):
    """Render API from pyo3-stub-gen JSON IR"""

    required_arguments = 1  # Module name

    def run(self):
        module_name = self.arguments[0]

        # Load JSON IR - check both api/ subdirectory and srcdir
        json_path = Path(self.env.srcdir) / "api" / "api_reference.json"
        if not json_path.exists():
            json_path = Path(self.env.srcdir) / "api_reference.json"

        with open(json_path) as f:
            doc_package = json.load(f)

        # Find module
        if module_name not in doc_package['modules']:
            return [nodes.error(None, nodes.paragraph(
                text=f"Module not found: {module_name}"))]

        doc_module = doc_package['modules'][module_name]

        # Build nodes for all items
        result = []
        for item in doc_module['items']:
            result.extend(self._build_item(item, module_name))

        return result

    def _build_item(self, item, module_name):
        kind = item['kind']
        if kind == 'Function':
            return self._build_function(item, module_name)
        elif kind == 'Class':
            return self._build_class(item, module_name)
        elif kind == 'TypeAlias':
            return self._build_type_alias(item, module_name)
        elif kind == 'Variable':
            return self._build_variable(item, module_name)
        return []

    def _build_function(self, func, module_name):
        return _build_function(self.env, func, module_name)

    def _build_type_alias(self, alias, module_name):
        return _build_type_alias(self.env, alias, module_name)

    def _build_class(self, cls, module_name):
        return _build_class(self.env, cls, module_name)

    def _build_variable(self, var, module_name):
        return _build_variable(self.env, var, module_name)

class Pyo3APIPackageDirective(SphinxDirective):
    """Render API for all modules in a package from pyo3-stub-gen JSON IR"""

    required_arguments = 1  # Package name

    def run(self):
        package_name = self.arguments[0]

        # Load JSON IR - check both api/ subdirectory and srcdir
        json_path = Path(self.env.srcdir) / "api" / "api_reference.json"
        if not json_path.exists():
            json_path = Path(self.env.srcdir) / "api_reference.json"

        with open(json_path) as f:
            doc_package = json.load(f)

        # Find all modules matching the package
        result = []
        for module_name in sorted(doc_package['modules'].keys()):
            # Include the package itself and all submodules
            if module_name == package_name or module_name.startswith(package_name + '.'):
                doc_module = doc_package['modules'][module_name]

                # Add section header for each module
                section = nodes.section(ids=[f'module-{module_name}'])
                title_text = f"{module_name} Module" if module_name != package_name else f"{module_name} Package"
                title = nodes.title(text=title_text)
                section += title

                # Build nodes for all items in this module
                for item in doc_module['items']:
                    section.extend(self._build_item(item, module_name))

                result.append(section)

        return result

    def _build_item(self, item, module_name):
        kind = item['kind']
        if kind == 'Function':
            return self._build_function(item, module_name)
        elif kind == 'Class':
            return self._build_class(item, module_name)
        elif kind == 'TypeAlias':
            return self._build_type_alias(item, module_name)
        elif kind == 'Variable':
            return self._build_variable(item, module_name)
        return []

    def _build_function(self, func, module_name):
        return _build_function(self.env, func, module_name)

    def _build_type_alias(self, alias, module_name):
        return _build_type_alias(self.env, alias, module_name)

    def _build_class(self, cls, module_name):
        return _build_class(self.env, cls, module_name)

    def _build_variable(self, var, module_name):
        return _build_variable(self.env, var, module_name)

def setup(app):
    app.add_directive('pyo3-api', Pyo3APIDirective)
    app.add_directive('pyo3-api-package', Pyo3APIPackageDirective)
    return {'version': '0.1', 'parallel_read_safe': True}
