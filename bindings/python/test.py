import logging

logging.basicConfig(level=logging.DEBUG)

from codegen_sdk_pink import Codebase
from codegen_sdk_pink.java import JavaFile

codebase = Codebase("/tmp/zaproxy")
print(len(codebase.files))
for file in codebase.files:
        for function in file.functions:
            print(function.name)
            print(file.get_function(str(function.name)))
