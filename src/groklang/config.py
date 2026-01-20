import os
import sys

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib

class Config:
    def __init__(self):
        self.ai_backend = "mock"
        self.ai_api_key = None
        self.ai_timeout = 5
        self.deadlock_detection = True
        self.load_config()

    def load_config(self):
        config_file = os.path.join(os.getcwd(), "grok.toml")
        if not os.path.exists(config_file):
            return  # Use defaults

        with open(config_file, "rb") as f:
            data = tomllib.load(f)

        if "ai" in data:
            ai_config = data["ai"]
            self.ai_backend = ai_config.get("backend", "mock")
            self.ai_api_key = ai_config.get("api_key", os.getenv("GROK_API_KEY"))
            self.ai_timeout = ai_config.get("timeout", 5)
        
        if "concurrency" in data:
            conc_config = data["concurrency"]
            self.deadlock_detection = conc_config.get("deadlock_detection", True)

config = Config()