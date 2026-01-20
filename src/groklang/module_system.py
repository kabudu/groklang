from typing import Dict, List, Set
from .ast_nodes import *

class ModuleResolver:
    def __init__(self):
        self.modules: Dict[str, Dict] = {}  # module_name -> {items, public_items}
        self.current_module = "root"

    def define_module(self, name: str, items: List):
        """Define a module with its items"""
        public_items = set()
        all_items = {}
        
        for item in items:
            if isinstance(item, tuple) and len(item) > 1 and item[0] in ('FunctionDef', 'StructDef', 'EnumDef', 'TraitDef'):
                # Handle tuple representation
                item_name = item[1]  # Assuming name is second element
                all_items[item_name] = item
                visibility = item[5] if len(item) > 5 else None  # visibility is 6th element
                if visibility == 'pub':
                    public_items.add(item_name)
            elif hasattr(item, 'name'):
                item_name = item.name
                all_items[item_name] = item
                if hasattr(item, 'visibility') and getattr(item, 'visibility', None) == 'pub':
                    public_items.add(item_name)
        
        self.modules[name] = {
            'items': all_items,
            'public': public_items
        }

    def resolve_use(self, path: List[str], importing_module: str) -> List[str]:
        """Resolve a use statement, return list of imported names"""
        if not path:
            return []
        
        # Simple resolution: assume all paths are absolute from root
        module_name = path[0]  # For now, handle single level
        if module_name in self.modules:
            return list(self.modules[module_name]['public'])
        return []

    def can_access(self, module: str, name: str, from_module: str) -> bool:
        """Check if name is accessible from from_module"""
        if module not in self.modules:
            return False
        
        mod_info = self.modules[module]
        if name in mod_info['public']:
            return True
        
        # Allow access within same module
        return module == from_module

class PrivacyChecker:
    def __init__(self, resolver: ModuleResolver):
        self.resolver = resolver
        self.current_module = "root"

    def check_privacy(self, ast):
        """Check privacy violations in AST"""
        errors = []
        
        def check_node(node):
            if isinstance(node, tuple):
                if node[0] == 'Use':
                    # Check if imported items are public
                    path = node[1]
                    imported = self.resolver.resolve_use(path, self.current_module)
                    if not imported:
                        errors.append(f"Cannot import from {'.'.join(path)}")
                else:
                    for child in node[1:] if len(node) > 1 else []:
                        check_node(child)
            elif isinstance(node, list):
                for item in node:
                    check_node(item)
            elif hasattr(node, '__dict__'):
                for attr, value in node.__dict__.items():
                    if attr.startswith('_'):
                        continue
                    check_node(value)
        
        check_node(ast)
        return errors