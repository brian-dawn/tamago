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

## With Poetry

Tell poetry to use a python version that matches the pyproject.toml file:

    poetry env use $(tamago find)/bin/python

Now you can do the normal poetry things:

    poetry shell
    poetry install

## Features

Build Python versions 3.8, 3.9, 3.10, and 3.11 on your system with one command.

We explicitely do not care about patch versions and always attempt to work with the latest available.

## Planned Features

- Use MUSL Python builds
