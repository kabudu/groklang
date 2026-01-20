from typing import List, Dict, Any
from .ast_nodes import *

class LlmService:
    def call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Call LLM (abstract)"""
        raise NotImplementedError

class MockLlmService(LlmService):
    def call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Mock service for testing"""
        op = request['operation']
        if op == 'optimize':
            return {'success': True, 'output': request['input']}  # Return as-is
        elif op == 'test':
            return {'success': True, 'output': '// Generated tests'}
        elif op == 'translate':
            return {'success': True, 'output': '# Translated code'}
        else:
            return {'success': False, 'error': 'Unknown operation'}

class DecoratorProcessor:
    def __init__(self, llm_service: LlmService):
        self.llm_service = llm_service

    def process_decorators(self, ast):
        """Expand compile-time decorators"""
        new_items = []

        for item in ast[1]:  # program items
            if isinstance(item, FunctionDef) and item.decorators:
                transformed = self.apply_decorators(item)
                new_items.append(transformed)
            else:
                new_items.append(item)

        return ('Program', new_items)

    def apply_decorators(self, item: FunctionDef):
        """Apply each decorator in sequence"""
        current = item

        for decorator in item.decorators:
            if decorator == 'ai_optimize':
                current = self.optimize_decorator(item)
            elif decorator == 'ai_test':
                current = self.test_decorator(item)
            elif decorator == 'ai_translate':
                current = self.translate_decorator(item)
            elif decorator == 'ai_optimize(runtime)':
                # Runtime optimization (not implemented in this phase)
                pass

        return current

    def optimize_decorator(self, item: FunctionDef):
        """Call AI to optimize function"""
        request = {
            'operation': 'optimize',
            'input': self.ast_to_code(item),
            'parameters': {'level': 'intermediate', 'target': 'speed'}
        }

        response = self.llm_service.call(request)
        if response['success']:
            # In runtime mode, could apply optimizations dynamically
            # For now, return optimized version (mock)
            return item  # Placeholder for optimized AST
        else:
            return item

    def test_decorator(self, item: FunctionDef):
        """Generate tests via AI"""
        request = {
            'operation': 'test',
            'input': self.ast_to_code(item),
            'parameters': {'iterations': '100'}
        }

        response = self.llm_service.call(request)
        if response['success']:
            # Generate test functions (simplified)
            test_code = response['output']
            # In full implementation, parse and add test functions
            return (item, test_code)  # Return function + generated tests
        return (item, "")

    def translate_decorator(self, item: FunctionDef):
        """Translate to target language"""
        request = {
            'operation': 'translate',
            'input': self.ast_to_code(item),
            'parameters': {'target_lang': 'py'}
        }

        response = self.llm_service.call(request)
        if response['success']:
            # Translation result
            return (item, response['output'])
        return (item, "")

    def ast_to_code(self, ast_node) -> str:
        """Simple AST to code converter (stub)"""
        if isinstance(ast_node, FunctionDef):
            params = ', '.join(f"{p[0]}: {p[2]}" if p[2] else p[0] for p in ast_node.params)
            return_type = f" -> {ast_node.return_type}" if ast_node.return_type else ""
            body = "// function body"
            return f"fn {ast_node.name}({params}){return_type} {{ {body} }}"
        return "// code"

class OpenAiService(LlmService):
    def __init__(self, api_key: str, model: str = "gpt-4"):
        self.api_key = api_key
        self.model = model

    def call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        import hashlib
        request_hash = hashlib.md5(str(request).encode()).hexdigest()

        # Check cache
        if hasattr(self, 'cache') and request_hash in self.cache:
            cached = self.cache[request_hash]
            # Still analyze cached output
            if 'output' in cached and cached.get('success'):
                security_issues = self._analyze_ai_output(cached['output'])
                if security_issues:
                    return {'success': False, 'error': f'Cached AI-generated content has security issues: {security_issues}'}
            return cached

        import openai
        openai.api_key = self.api_key

        try:
            response = openai.ChatCompletion.create(
                model=self.model,
                messages=[{"role": "user", "content": self.format_prompt(request)}],
                temperature=0.7,
                timeout=5,
            )
            ai_output = response.choices[0].message.content
            # Post-AI static analysis
            security_issues = self._analyze_ai_output(ai_output)
            if security_issues:
                result = {'success': False, 'error': f'AI-generated content has security issues: {security_issues}'}
            else:
                result = {'success': True, 'output': ai_output}
            # Cache result
            if not hasattr(self, 'cache'):
                self.cache = {}
            self.cache[request_hash] = result
            return result
        except Exception as e:
            result = {'success': False, 'error': str(e)}
            if not hasattr(self, 'cache'):
                self.cache = {}
            self.cache[request_hash] = result
            return result

    def _analyze_ai_output(self, code: str) -> list:
        """Post-AI static analysis for security issues"""
        issues = []
        # Basic security checks
        if 'eval(' in code or 'exec(' in code:
            issues.append('Potential code injection via eval/exec')
        if 'import os' in code and 'os.system' in code:
            issues.append('Unsafe system calls detected')
        if 'open(' in code and ('w' in code or 'a' in code):
            issues.append('File write operations without input validation')
        if 'subprocess' in code:
            issues.append('Subprocess usage without sanitization')
        # Add more patterns as needed
        return issues

    def format_prompt(self, request: Dict[str, Any]) -> str:
        op = request['operation']
        code = request['input']

        if op == 'optimize':
            return f"Optimize this function for speed:\n\n{code}\n\nProvide only the optimized code."
        elif op == 'test':
            return f"Generate comprehensive test cases for:\n\n{code}\n\nProvide tests in GrokLang format."
        else:
            return code

class XaiGrokService(LlmService):
    def __init__(self, api_key: str, model: str = "grok-beta"):
        self.api_key = api_key
        self.model = model

    def call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        import requests  # Assuming REST API

        url = "https://api.x.ai/v1/chat/completions"  # Placeholder
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        payload = {
            "model": self.model,
            "messages": [{"role": "user", "content": self.format_prompt(request)}],
            "temperature": 0.7
        }

        try:
            response = requests.post(url, json=payload, headers=headers, timeout=5)
            if response.status_code == 200:
                data = response.json()
                return {'success': True, 'output': data['choices'][0]['message']['content']}
            else:
                return {'success': False, 'error': f"API error: {response.status_code}"}
        except Exception as e:
            return {'success': False, 'error': str(e)}

    def format_prompt(self, request: Dict[str, Any]) -> str:
        op = request['operation']
        code = request['input']

        if op == 'optimize':
            return f"Optimize this GrokLang function for speed:\n\n{code}\n\nProvide only the optimized GrokLang code."
        elif op == 'test':
            return f"Generate comprehensive test cases for this GrokLang function:\n\n{code}\n\nProvide tests in GrokLang format."
        elif op == 'translate':
            target = request['parameters'].get('target_lang', 'py')
            return f"Translate this GrokLang code to {target}:\n\n{code}\n\nProvide only the translated code."
        else:
            return code