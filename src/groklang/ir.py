from typing import List

class IRInstruction:
    def __init__(self, opcode: str, args: List):
        self.opcode = opcode
        self.args = args

class IRBlock:
    def __init__(self, label: str):
        self.label = label
        self.instructions: List[IRInstruction] = []

class IRFunction:
    def __init__(self, name: str, params: List, blocks: List[IRBlock]):
        self.name = name
        self.params = params
        self.blocks = blocks