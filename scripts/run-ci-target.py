#!/usr/bin/env python3
"""Run a named CI target with exact argument boundaries and Cargo's exit status."""
import argparse
import json
import os
from pathlib import Path


def command(name, release=False):
    targets = json.loads(Path(__file__).with_name("ci-targets.json").read_text())
    if name not in targets:
        raise ValueError(f"unknown CI target: {name}")
    arguments = targets[name]
    return ["cargo", arguments[0], *(["--release"] if release else []), *arguments[1:]]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("name")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--print-command", action="store_true")
    args = parser.parse_args()
    try:
        arguments = command(args.name, args.release)
    except ValueError as error:
        parser.error(str(error))
    if args.print_command:
        print(json.dumps(arguments))
    else:
        os.execvp(arguments[0], arguments)


if __name__ == "__main__":
    main()
