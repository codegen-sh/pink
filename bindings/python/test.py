import logging

logging.basicConfig(level=logging.DEBUG)

from codegen_sdk_pink import Codebase

codebase = Codebase("/Users/ellen/workspace/scratch/codegen-sdk/src")
print(len(codebase.files))
for file in codebase.files:
    print(file.path)
    for function in file.functions:
        print(function.name)
        print(file.get_function(str(function.name)))
