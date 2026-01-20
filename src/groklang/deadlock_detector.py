from typing import List, Dict, Any, Set
from .decorator_processor import LlmService
import re

class DeadlockDetector:
    def __init__(self, llm_service: LlmService):
        self.llm_service = llm_service

    def analyze_code(self, code: str) -> Dict[str, Any]:
        # Static analysis first
        static_risks = self._static_deadlock_analysis(code)
        ai_risks = self._ai_deadlock_analysis(code)
        
        # Combine results
        overall_risk = self._combine_risks(static_risks, ai_risks)
        recommendations = static_risks['recommendations'] + ai_risks.get('recommendations', [])
        
        return {
            'risk_level': overall_risk,
            'recommendations': recommendations,
            'static_analysis': static_risks,
            'ai_analysis': ai_risks
        }

    def _static_deadlock_analysis(self, code: str) -> Dict[str, Any]:
        """Static analysis for deadlock patterns"""
        risks = []
        recommendations = []
        
        # Check for nested locks
        if re.search(r'lock.*lock', code, re.IGNORECASE):
            risks.append("Potential nested locking")
            recommendations.append("Avoid nested locks; use timeouts")
        
        # Check for channel operations without timeouts
        if 'receive' in code and 'timeout' not in code:
            risks.append("Unbounded channel receives")
            recommendations.append("Add timeouts to channel operations")
        
        # Check for circular dependencies in actors
        actor_sends = re.findall(r'send\s*\(\s*(\w+)', code)
        if len(set(actor_sends)) > 1:  # Multiple sends, potential cycle
            risks.append("Potential circular actor communication")
            recommendations.append("Model actor dependencies as DAG")
        
        risk_level = 'high' if len(risks) > 2 else 'medium' if risks else 'low'
        
        return {
            'risk_level': risk_level,
            'issues': risks,
            'recommendations': recommendations
        }

    def _combine_risks(self, static: Dict, ai: Dict) -> str:
        """Combine static and AI risk levels"""
        levels = {'low': 1, 'medium': 2, 'high': 3}
        static_level = levels.get(static['risk_level'], 1)
        ai_level = levels.get(ai.get('risk_level', 'low'), 1)
        combined = max(static_level, ai_level)
        return {1: 'low', 2: 'medium', 3: 'high'}[combined]

    def _ai_deadlock_analysis(self, code: str) -> Dict[str, Any]:
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