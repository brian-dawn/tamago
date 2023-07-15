# Tamago

TL;DR just use pyenv. This is just a playground.

## Usage

Install it:

    cargo install --path .

Build Python versions into a sandbox with the latest patch versions:

    tamago build

Activate auto selection:

    source $(tamago activate)

Now you can cd into a directory with a `.python-version` file and
it will auto select the relevant python version (ignoring the patch version).

## Features

Build Python versions 3.8, 3.9, 3.10, and 3.11 on your system with one command.

We explicitely do not care about patch versions and always attempt to work with the latest available.

## Planned Features

- Use MUSL Python builds
