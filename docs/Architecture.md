# Architecture

## Goals

Our goals are to parse and analyze large codebases. Unique constraints driving the design of this project are that we need to:

- Parse large codebases quickly
- Respond to many changes to these codebases correctly and effeciently
- Parse many different languages with a common framework
- Expose an easy to use API and understanding of the codebase to end users

However, we don't need to accurately type check a codebase. We can assume the syntax is valid and typing is correct prior to any of our operations.

## Design

Pink has 4 layers:

- CST: The syntax nodes 1:1 corresponding to tree-sitter nodes
- AST: A higher level representation of the CST, focusing on relevant nodes
- Analyzer: An engine to compute references/dependencies between AST nodes and update that representation in response to external and internal changes
- Bindings: A user friendly wrapper around the previous layers

## Implementation

To achieve this, we rely on code generation to scale across languages quickly

### CST

We generate code for this layer using tree-sitter's node-types.json found in each language.
See ../codegen-sdk-cst-generator for more.
