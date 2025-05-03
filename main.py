import re
from typing import Dict, List, Tuple, Optional, Union, Callable, Any
from dataclasses import dataclass
from enum import Enum
from abc import ABC, abstractmethod

class SyncMode(Enum):
    CONCURRENT = "concurrent"
    SEQUENTIAL = "sequential"

@dataclass
class TypeInfo:
    base_type: str  # int, signed, etc.
    width: int
    sync_mode: Optional[SyncMode] = None
    
    @classmethod
    def parse(cls, type_str: str) -> 'TypeInfo':
        """Parse a type string like 'int.32::concurrent'"""
        parts = type_str.split("::")
        type_part = parts[0]
        sync_mode = SyncMode(parts[1]) if len(parts) > 1 else None
        
        # Parse base type and width
        if "." in type_part:
            base_type, width = type_part.split(".")
            width = int(width)
        else:
            base_type = type_part
            width = 32  # Default width
        
        return cls(base_type, width, sync_mode)

@dataclass
class Variable:
    name: str
    type_info: TypeInfo

@dataclass
class Function:
    name: str
    parameters: List[Variable]
    return_type: Optional[TypeInfo]
    statements: List[str]
    default_sync_mode: SyncMode = SyncMode.CONCURRENT

# Base Rule class for conversion rules
class ConversionRule(ABC):
    @abstractmethod
    def pattern(self) -> str:
        """Return regex pattern to match"""
        pass
    
    @abstractmethod
    def convert(self, match: re.Match, context: Dict[str, Any]) -> str:
        """Convert matched text to Verilog"""
        pass

# Grammar and Conversion Rules
class VariableDeclarationRule(ConversionRule):
    def pattern(self) -> str:
        return r'(\w+(?:\.\d+)?(?:::\w+)?)\s+(\w+);'
    
    def convert(self, match: re.Match, context: Dict[str, Any]) -> str:
        type_str, var_name = match.groups()
        type_info = TypeInfo.parse(type_str)
        context['variables'][var_name] = Variable(var_name, type_info)
        return None  # Declaration doesn't generate output

class FunctionDeclarationRule(ConversionRule):
    def pattern(self) -> str:
        return r'function(?:::(\w+))?\s*(?:(\w+(?:\.\d+)?(?:::\w+)?))?\s*(\w+)\((.*?)\)\s*\{'
    
    def convert(self, match: re.Match, context: Dict[str, Any]) -> str:
        sync_mode_str, return_type_str, name, params_str = match.groups()
        
        # Parse sync mode from function declaration or default to concurrent
        default_sync = SyncMode(sync_mode_str) if sync_mode_str else SyncMode.CONCURRENT
        
        # Parse return type if present
        return_type = TypeInfo.parse(return_type_str) if return_type_str else None
        
        # Parse parameters
        parameters = []
        if params_str.strip():
            for param in params_str.split(','):
                param = param.strip()
                if param:
                    parts = param.split()
                    if len(parts) >= 2:
                        type_info = TypeInfo.parse(parts[0])
                        var_name = ' '.join(parts[1:])
                        parameters.append(Variable(var_name, type_info))
        
        function = Function(name, parameters, return_type, [], default_sync)
        context['current_function'] = function
        context['functions'][name] = function
        return None

class AssignmentRule(ConversionRule):
    def pattern(self) -> str:
        return r'(\w+)\s*=\s*([^;]+);'
    
    def convert(self, match: re.Match, context: Dict[str, Any]) -> str:
        var_name, expr = match.groups()
        
        # Convert expression to Verilog
        converted_expr = self._convert_expression(expr, context)
        
        # Determine if non-blocking assignment is needed
        if context.get('in_always_block', False):
            return f"{var_name} <= {converted_expr};"
        else:
            return f"assign {var_name} = {converted_expr};"

    def _convert_expression(self, expr: str, context: Dict[str, Any]) -> str:
        """Convert expression, including function calls"""
        # Pattern for function calls
        func_call_pattern = r'(\w+)\((.*?)\)'
        
        def replace_func_call(match):
            func_name, args = match.groups()
            if func_name in context.get('functions', {}):
                func = context['functions'][func_name]
                return self._generate_function_call(func, args, context)
            return match.group(0)
        
        return re.sub(func_call_pattern, replace_func_call, expr)
    
    def _generate_function_call(self, func: Function, args: str, context: Dict[str, Any]) -> str:
        """Generate Verilog for function call based on sync modes"""
        arg_parts = args.split(',') if args else []
        
        # Check if any parameter has specific sync mode
        has_sequential = any(param.type_info.sync_mode == SyncMode.SEQUENTIAL 
                           for param in func.parameters if len(arg_parts) > func.parameters.index(param))
        
        # Use default function sync mode if parameter doesn't specify
        effective_sync_mode = SyncMode.SEQUENTIAL if has_sequential else func.default_sync_mode
        
        if effective_sync_mode == SyncMode.CONCURRENT:
            return f"{func.name}({args})"
        else:
            # For sequential calls, we need to instantiate the module
            instance_name = f"{func.name}_inst_{context.get('call_counter', 0)}"
            context['sequential_calls'].append((func, instance_name, args))
            return f"{instance_name}_result"

# Conversion engine with rule-based system
class PseudoCToVerilog:
    def __init__(self):
        self.rules: List[ConversionRule] = []
        self.context: Dict[str, Any] = {
            'variables': {},
            'functions': {},
            'sequential_calls': [],
            'call_counter': 0,
            'in_always_block': False
        }
        
        # Register default rules
        self.register_rule(VariableDeclarationRule())
        self.register_rule(FunctionDeclarationRule())
        self.register_rule(AssignmentRule())
    
    def register_rule(self, rule: ConversionRule):
        """Add a new conversion rule"""
        self.rules.append(rule)
    
    def parse_and_convert(self, code: str) -> str:
        """Parse pseudo-C code and convert to Verilog"""
        lines = code.strip().split('\n')
        
        # Process each line through rules
        processed_lines = []
        i = 0
        
        while i < len(lines):
            line = lines[i].strip()
            
            # Check if we're entering a function body
            if '{' in line and i > 0:
                prev_line = lines[i-1].strip()
                if 'function' in prev_line:
                    # Parse function body
                    body_lines, i = self._parse_block(lines, i)
                    if self.context.get('current_function'):
                        self.context['current_function'].statements = body_lines
                    continue
            
            # Try to match line with rules
            matched = False
            for rule in self.rules:
                match = re.match(rule.pattern(), line)
                if match:
                    result = rule.convert(match, self.context)
                    if result:
                        processed_lines.append(result)
                    matched = True
                    break
            
            if not matched:
                processed_lines.append(line)
            
            i += 1
        
        # Generate Verilog code
        return self._generate_verilog(processed_lines)
    
    def _parse_block(self, lines: List[str], start_idx: int) -> Tuple[List[str], int]:
        """Parse a block of code (function body, main, etc.)"""
        block_lines = []
        i = start_idx + 1
        brace_count = 1
        
        while i < len(lines) and brace_count > 0:
            line = lines[i].strip()
            if '{' in line:
                brace_count += line.count('{')
            if '}' in line:
                brace_count -= line.count('}')
            
            if brace_count > 0:
                block_lines.append(line)
            
            i += 1
        
        return block_lines, i - 1
    
    def _generate_verilog(self, processed_lines: List[str]) -> str:
        """Generate complete Verilog code"""
        verilog_code = []
        
        # Module header
        verilog_code.append("module main(")
        verilog_code.append("    input clk,")
        verilog_code.append("    input rst")
        verilog_code.append(");")
        verilog_code.append("")
        
        # Variable declarations
        for var_name, var in self.context['variables'].items():
            verilog_code.append(self._generate_variable_declaration(var))
        verilog_code.append("")
        
        # Always block for main logic
        verilog_code.append("    always @(posedge clk or posedge rst) begin")
        verilog_code.append("        if (rst) begin")
        
        # Reset logic
        for var_name in self.context['variables']:
            verilog_code.append(f"            {var_name} <= 0;")
        
        verilog_code.append("        end else begin")
        
        # Main logic
        self.context['in_always_block'] = True
        for line in processed_lines:
            if line and not line.startswith(('int', 'signed', 'function')):
                # Process through assignment rule if needed
                for rule in self.rules:
                    if isinstance(rule, AssignmentRule):
                        match = re.match(rule.pattern(), line)
                        if match:
                            converted = rule.convert(match, self.context)
                            if converted:
                                verilog_code.append(f"            {converted}")
                                break
                else:
                    verilog_code.append(f"            {line}")
        
        self.context['in_always_block'] = False
        
        verilog_code.append("        end")
        verilog_code.append("    end")
        verilog_code.append("")
        
        # Generate function modules
        for func_name, func in self.context['functions'].items():
            verilog_code.extend(self._generate_function_module(func))
        
        # Generate sequential function instances
        for func, instance_name, args in self.context['sequential_calls']:
            verilog_code.extend(self._generate_sequential_instance(func, instance_name, args))
        
        verilog_code.append("endmodule")
        
        return "\n".join(verilog_code)
    
    def _generate_variable_declaration(self, var: Variable) -> str:
        """Generate Verilog variable declaration"""
        type_str = "signed " if var.type_info.base_type == "signed" else ""
        return f"    reg {type_str}[{var.type_info.width-1}:0] {var.name};"
    
    def _generate_function_module(self, func: Function) -> List[str]:
        """Generate Verilog module for a function"""
        module_code = []
        
        # Module header
        ports = ["input clk", "input rst"]
        
        # Input ports
        for param in func.parameters:
            type_str = "signed " if param.type_info.base_type == "signed" else ""
            ports.append(f"input {type_str}[{param.type_info.width-1}:0] {param.name}")
        
        # Output port
        if func.return_type:
            type_str = "signed " if func.return_type.base_type == "signed" else ""
            ports.append(f"output reg {type_str}[{func.return_type.width-1}:0] result")
        
        module_code.append(f"module {func.name}(")
        module_code.append("    " + ",\n    ".join(ports))
        module_code.append(");")
        module_code.append("")
        
        # Function logic
        module_code.append("    always @(posedge clk or posedge rst) begin")
        module_code.append("        if (rst) begin")
        
        if func.return_type:
            module_code.append("            result <= 0;")
        
        module_code.append("        end else begin")
        
        # Convert function body
        for stmt in func.statements:
            # Reprocess through rules for function context
            converted = stmt
            for rule in self.rules:
                if isinstance(rule, AssignmentRule):
                    match = re.match(rule.pattern(), stmt)
                    if match:
                        self.context['in_always_block'] = True
                        converted = rule.convert(match, self.context)
                        self.context['in_always_block'] = False
                        break
            
            if converted:
                module_code.append(f"            {converted}")
        
        module_code.append("        end")
        module_code.append("    end")
        module_code.append("")
        module_code.append("endmodule")
        module_code.append("")
        
        return module_code
    
    def _generate_sequential_instance(self, func: Function, instance_name: str, args: str) -> List[str]:
        """Generate instance for sequential function call"""
        instance_code = []
        
        instance_code.append(f"    // Sequential function instance: {instance_name}")
        instance_code.append(f"    {func.name} {instance_name} (")
        instance_code.append("        .clk(clk),")
        instance_code.append("        .rst(rst),")
        
        # Connect parameters
        arg_list = args.split(',') if args else []
        for i, param in enumerate(func.parameters):
            arg_val = arg_list[i].strip() if i < len(arg_list) else '0'
            instance_code.append(f"        .{param.name}({arg_val}),")
        
        # Connect result
        if func.return_type:
            instance_code.append(f"        .result({instance_name}_result)")
        
        instance_code[-1] = instance_code[-1].rstrip(',')
        instance_code.append("    );")
        instance_code.append("")
        
        return instance_code

# Example custom rules
class BitwiseOperatorRule(ConversionRule):
    def pattern(self) -> str:
        return r'(\w+)\s*([&|^])\s*(\w+)'
    
    def convert(self, match: re.Match, context: Dict[str, Any]) -> str:
        left, op, right = match.groups()
        verilog_op = {
            '&': '&',
            '|': '|',
            '^': '^'
        }[op]
        return f"{left} {verilog_op} {right}"

# Example usage
if __name__ == "__main__":
    # Create converter and add custom rules
    converter = PseudoCToVerilog()
    converter.register_rule(BitwiseOperatorRule())
    
    # Example pseudo-C code
    pseudo_c_code = """
    int.32 a;
    signed.16 b;
    int counter;
    
    function::concurrent int.32 add(int.32 x, int.32::sequential y) {
        result = x + y;
    }
    
    function::sequential signed.8 multiply(int.8 x, int.8 y) {
        temp = x * y;
        result = temp;
    }
    
    a = 0;
    b = 5;
    counter = 0;
    
    a = add(b, counter);
    b = multiply(3, 4);
    
    counter = counter + 1;
    """
    
    # Convert to Verilog
    verilog_code = converter.parse_and_convert(pseudo_c_code)
    print(verilog_code)