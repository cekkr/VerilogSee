from typing import Dict, List, Tuple, Optional, Union, Any
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

# Token types
class TokenType(Enum):
    TYPE = "TYPE"
    IDENTIFIER = "IDENTIFIER"
    OPERATOR = "OPERATOR"
    DELIMITER = "DELIMITER"
    KEYWORD = "KEYWORD"
    NUMBER = "NUMBER"

@dataclass
class Token:
    type: TokenType
    value: str
    line: int
    column: int

# Parser base class
class Parser:
    def __init__(self):
        self.context = {
            'variables': {},
            'functions': {},
            'sequential_calls': [],
            'call_counter': 0,
            'in_always_block': False
        }
        
    def parse_code(self, code: str) -> str:
        """Parse pseudo-C code and convert to Verilog"""
        lines = code.strip().split('\n')
        tokens = self.tokenize(lines)
        ast = self.build_ast(tokens)
        verilog = self.generate_verilog(ast)
        return verilog
    
    def tokenize(self, lines: List[str]) -> List[Token]:
        """Convert code lines into tokens"""
        tokens = []
        
        for line_idx, line in enumerate(lines):
            if not line.strip():
                continue
                
            col_idx = 0
            while col_idx < len(line):
                # Skip whitespace
                if line[col_idx].isspace():
                    col_idx += 1
                    continue
                
                # Check for type declarations
                if self._check_type_declaration(line, col_idx):
                    type_str, new_idx = self._extract_type(line, col_idx)
                    tokens.append(Token(TokenType.TYPE, type_str, line_idx, col_idx))
                    col_idx = new_idx
                    continue
                
                # Check for keywords
                keyword_match = self._check_keyword(line, col_idx)
                if keyword_match:
                    keyword, new_idx = keyword_match
                    tokens.append(Token(TokenType.KEYWORD, keyword, line_idx, col_idx))
                    col_idx = new_idx
                    continue
                
                # Check for identifiers
                if line[col_idx].isalpha() or line[col_idx] == '_':
                    ident, new_idx = self._extract_identifier(line, col_idx)
                    tokens.append(Token(TokenType.IDENTIFIER, ident, line_idx, col_idx))
                    col_idx = new_idx
                    continue
                
                # Check for operators
                if line[col_idx] in "=+-*/&|^<>":
                    # Handle multi-character operators
                    if col_idx + 1 < len(line) and line[col_idx:col_idx+2] in ["<=", ">=", "==", "!="]:
                        tokens.append(Token(TokenType.OPERATOR, line[col_idx:col_idx+2], line_idx, col_idx))
                        col_idx += 2
                    else:
                        tokens.append(Token(TokenType.OPERATOR, line[col_idx], line_idx, col_idx))
                        col_idx += 1
                    continue
                
                # Check for delimiters
                if line[col_idx] in "{}();,":
                    tokens.append(Token(TokenType.DELIMITER, line[col_idx], line_idx, col_idx))
                    col_idx += 1
                    continue
                
                # Check for numbers
                if line[col_idx].isdigit():
                    num, new_idx = self._extract_number(line, col_idx)
                    tokens.append(Token(TokenType.NUMBER, num, line_idx, col_idx))
                    col_idx = new_idx
                    continue
                
                # Unrecognized character
                col_idx += 1
        
        return tokens
    
    def _check_type_declaration(self, line: str, col_idx: int) -> bool:
        """Check if position is at start of a type declaration"""
        type_prefixes = ["int", "signed", "unsigned"]
        for prefix in type_prefixes:
            if line[col_idx:].startswith(prefix):
                # Make sure it's followed by a delimiter or is a complete token
                next_idx = col_idx + len(prefix)
                if next_idx >= len(line) or line[next_idx].isspace() or line[next_idx] in ".::":
                    return True
        return False
    
    def _extract_type(self, line: str, col_idx: int) -> Tuple[str, int]:
        """Extract a complete type declaration"""
        # Possible parts: base_type.width::sync_mode
        start_idx = col_idx
        while col_idx < len(line) and not line[col_idx].isspace() and line[col_idx] not in ";,(){}":
            col_idx += 1
        return line[start_idx:col_idx], col_idx
    
    def _check_keyword(self, line: str, col_idx: int) -> Optional[Tuple[str, int]]:
        """Check if position is at a keyword"""
        keywords = ["function", "if", "else", "for", "while", "return"]
        for keyword in keywords:
            if line[col_idx:].startswith(keyword):
                # Make sure it's a complete token
                next_idx = col_idx + len(keyword)
                if next_idx >= len(line) or line[next_idx].isspace() or line[next_idx] in "({;:":
                    return keyword, next_idx
        return None
    
    def _extract_identifier(self, line: str, col_idx: int) -> Tuple[str, int]:
        """Extract a complete identifier"""
        start_idx = col_idx
        while col_idx < len(line) and (line[col_idx].isalnum() or line[col_idx] == '_'):
            col_idx += 1
        return line[start_idx:col_idx], col_idx
    
    def _extract_number(self, line: str, col_idx: int) -> Tuple[str, int]:
        """Extract a complete number"""
        start_idx = col_idx
        while col_idx < len(line) and line[col_idx].isdigit():
            col_idx += 1
        return line[start_idx:col_idx], col_idx
    
    def build_ast(self, tokens: List[Token]) -> Dict:
        """Build abstract syntax tree from tokens"""
        ast = {
            'variables': {},
            'functions': {},
            'statements': []
        }
        
        i = 0
        while i < len(tokens):
            # Variable declaration
            if tokens[i].type == TokenType.TYPE and i + 1 < len(tokens) and tokens[i+1].type == TokenType.IDENTIFIER:
                type_info = TypeInfo.parse(tokens[i].value)
                var_name = tokens[i+1].value
                ast['variables'][var_name] = Variable(var_name, type_info)
                self.context['variables'][var_name] = Variable(var_name, type_info)
                i += 3  # Skip type, name, semicolon
                continue
            
            # Function declaration
            if tokens[i].type == TokenType.KEYWORD and tokens[i].value == "function":
                i, function = self._parse_function(tokens, i)
                ast['functions'][function.name] = function
                self.context['functions'][function.name] = function
                continue
            
            # Assignment statement
            if tokens[i].type == TokenType.IDENTIFIER and i + 1 < len(tokens) and tokens[i+1].type == TokenType.OPERATOR and tokens[i+1].value == "=":
                var_name = tokens[i].value
                expr_tokens = []
                i += 2  # Skip identifier and equals
                while i < len(tokens) and tokens[i].type != TokenType.DELIMITER:
                    expr_tokens.append(tokens[i])
                    i += 1
                
                expr = self._tokens_to_expression(expr_tokens)
                ast['statements'].append(('assignment', var_name, expr))
                
                i += 1  # Skip semicolon
                continue
            
            # Skip unrecognized token
            i += 1
        
        return ast
    
    def _parse_function(self, tokens: List[Token], start_idx: int) -> Tuple[int, Function]:
        """Parse a function declaration and body"""
        i = start_idx
        sync_mode = SyncMode.CONCURRENT
        
        # Check for sync mode
        if i + 2 < len(tokens) and tokens[i+1].type == TokenType.DELIMITER and tokens[i+1].value == "::" and tokens[i+2].type == TokenType.IDENTIFIER:
            sync_str = tokens[i+2].value
            sync_mode = SyncMode(sync_str)
            i += 3
        else:
            i += 1
        
        # Parse return type if present
        return_type = None
        if i < len(tokens) and tokens[i].type == TokenType.TYPE:
            return_type = TypeInfo.parse(tokens[i].value)
            i += 1
        
        # Function name
        if i < len(tokens) and tokens[i].type == TokenType.IDENTIFIER:
            func_name = tokens[i].value
            i += 1
        else:
            raise ValueError("Expected function name")
        
        # Parameter list
        parameters = []
        if i < len(tokens) and tokens[i].type == TokenType.DELIMITER and tokens[i].value == "(":
            i += 1
            
            # Parse parameters
            while i < len(tokens) and not (tokens[i].type == TokenType.DELIMITER and tokens[i].value == ")"):
                if tokens[i].type == TokenType.TYPE:
                    param_type = TypeInfo.parse(tokens[i].value)
                    i += 1
                    
                    if i < len(tokens) and tokens[i].type == TokenType.IDENTIFIER:
                        param_name = tokens[i].value
                        parameters.append(Variable(param_name, param_type))
                        i += 1
                    
                    # Skip comma if present
                    if i < len(tokens) and tokens[i].type == TokenType.DELIMITER and tokens[i].value == ",":
                        i += 1
                else:
                    i += 1
            
            i += 1  # Skip closing paren
        
        # Function body
        statements = []
        if i < len(tokens) and tokens[i].type == TokenType.DELIMITER and tokens[i].value == "{":
            i += 1
            
            # Parse function body
            while i < len(tokens) and not (tokens[i].type == TokenType.DELIMITER and tokens[i].value == "}"):
                if tokens[i].type == TokenType.IDENTIFIER and i + 1 < len(tokens) and tokens[i+1].type == TokenType.OPERATOR and tokens[i+1].value == "=":
                    var_name = tokens[i].value
                    expr_tokens = []
                    i += 2  # Skip identifier and equals
                    
                    while i < len(tokens) and not (tokens[i].type == TokenType.DELIMITER and tokens[i].value == ";"):
                        expr_tokens.append(tokens[i])
                        i += 1
                    
                    expr = self._tokens_to_expression(expr_tokens)
                    statements.append(('assignment', var_name, expr))
                    
                    i += 1  # Skip semicolon
                else:
                    i += 1
            
            i += 1  # Skip closing brace
        
        function = Function(func_name, parameters, return_type, statements, sync_mode)
        return i, function
    
    def _tokens_to_expression(self, tokens: List[Token]) -> str:
        """Convert tokens to an expression string"""
        expr_parts = []
        
        for token in tokens:
            if token.type == TokenType.IDENTIFIER:
                expr_parts.append(token.value)
            elif token.type == TokenType.OPERATOR:
                expr_parts.append(token.value)
            elif token.type == TokenType.NUMBER:
                expr_parts.append(token.value)
            elif token.type == TokenType.DELIMITER and token.value in "()":
                expr_parts.append(token.value)
        
        return " ".join(expr_parts)
    
    def generate_verilog(self, ast: Dict) -> str:
        """Generate Verilog code from AST"""
        verilog_code = []
        
        # Module header
        verilog_code.append("module main(")
        verilog_code.append("    input clk,")
        verilog_code.append("    input rst")
        verilog_code.append(");")
        verilog_code.append("")
        
        # Variable declarations
        for var_name, var in ast['variables'].items():
            verilog_code.append(self._generate_variable_declaration(var))
        verilog_code.append("")
        
        # Always block for main logic
        verilog_code.append("    always @(posedge clk or posedge rst) begin")
        verilog_code.append("        if (rst) begin")
        
        # Reset logic
        for var_name in ast['variables']:
            verilog_code.append(f"            {var_name} <= 0;")
        
        verilog_code.append("        end else begin")
        
        # Main logic
        self.context['in_always_block'] = True
        for stmt_type, *args in ast['statements']:
            if stmt_type == 'assignment':
                var_name, expr = args
                converted_expr = self._convert_expression(expr)
                verilog_code.append(f"            {var_name} <= {converted_expr};")
        
        self.context['in_always_block'] = False
        
        verilog_code.append("        end")
        verilog_code.append("    end")
        verilog_code.append("")
        
        # Generate function modules
        for func_name, func in ast['functions'].items():
            verilog_code.extend(self._generate_function_module(func))
        
        # Generate sequential function instances
        for func, instance_name, args in self.context.get('sequential_calls', []):
            verilog_code.extend(self._generate_sequential_instance(func, instance_name, args))
        
        verilog_code.append("endmodule")
        
        return "\n".join(verilog_code)
    
    def _generate_variable_declaration(self, var: Variable) -> str:
        """Generate Verilog variable declaration"""
        type_str = "signed " if var.type_info.base_type == "signed" else ""
        return f"    reg {type_str}[{var.type_info.width-1}:0] {var.name};"
    
    def _convert_expression(self, expr: str) -> str:
        """Convert expression, including function calls"""
        # Simple expression conversion for now
        return expr
    
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
        for stmt_type, *args in func.statements:
            if stmt_type == 'assignment':
                var_name, expr = args
                converted_expr = self._convert_expression(expr)
                module_code.append(f"            {var_name} <= {converted_expr};")
        
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

# Example usage
if __name__ == "__main__":
    # Create parser
    parser = Parser()
    
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
    verilog_code = parser.parse_code(pseudo_c_code)
    print(verilog_code)