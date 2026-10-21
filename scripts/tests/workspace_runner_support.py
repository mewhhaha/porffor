"""Locate the execution closure for existing isolated runner policy tests."""
import ast


def runner_body(source):
    tree = ast.parse(source.read_text())
    return next(node.body for node in tree.body
                if isinstance(node, ast.FunctionDef) and node.name == "run")
