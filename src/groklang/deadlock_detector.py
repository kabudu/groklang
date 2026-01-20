from typing import List, Dict, Any
from .decorator_processor import LlmService

class DeadlockDetector:
    def __init__(self, llm_service: LlmService):
        self.llm_service = llm_service

    def analyze_code(self, code: str) -> Dict[str, Any]:
        """Use AI to detect potential deadlocks in code"""
        prompt = f"""
        Analyze the following GrokLang code for potential deadlocks in concurrency (actors, channels, threads):

        {code}

        Look for:
        - Circular waits
        - Resource starvation
        - Improper channel usage
        - Actor dependencies

        Provide a risk assessment (low/medium/high) and recommendations.
        """
        
        request = {
            'operation': 'analyze',
            'input': code,
            'parameters': {'analysis_type': 'deadlock'}
        }
        
        response = self.llm_service.call(request)
        return {
            'risk_level': response.get('output', 'unknown'),
            'recommendations': response.get('output', 'No analysis available')
        }