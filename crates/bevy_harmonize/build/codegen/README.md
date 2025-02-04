# Codegen

For local development, this directory is used for mod codegen.

The root cargo workspace will be used for mod building. This is nice because it allows for configuration-less language server integration.

## The Empty Crate

Cargo will refuse to compile a project if any cargo workspace wildcard matches nothing, hence we keep this directory in source version control.
