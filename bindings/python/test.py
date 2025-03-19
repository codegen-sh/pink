import logging

logging.basicConfig(level=logging.INFO)

from codegen_sdk_pink import Codebase
from codegen_sdk_pink.java import JavaFile

codebase = Codebase("/tmp/core")
for file in codebase.files:
        for function in file.functions:
            if function.references:
                print(function.name)
                print(file.get_function(str(function.name)))
                print(function.references)
                exit(0)
