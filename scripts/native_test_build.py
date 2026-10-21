"""Build one exact workspace libtest target through Cargo's JSON protocol."""

import json
import subprocess

from workspace_test_artifacts import test_artifacts


def build_test_binary(package, target, kind):
    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"],
            text=True,
        )
    )
    owners = [
        row["id"]
        for row in metadata["packages"]
        if row["id"] in metadata["workspace_members"] and row["name"] == package
    ]
    if len(owners) != 1:
        raise ValueError(f"expected one workspace package: {package}")
    selector = ["--lib"] if kind == "lib" else ["--" + kind, target]
    result = subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "-p",
            package,
            *selector,
            "--no-run",
            "--message-format=json-render-diagnostics",
        ],
        stdout=subprocess.PIPE,
        text=True,
    )
    messages = [json.loads(line) for line in result.stdout.splitlines()]
    for message in messages:
        if message.get("reason") == "compiler-message":
            print(message["message"].get("rendered", ""), end="", flush=True)
    result.check_returncode()
    if [
        row.get("success") for row in messages if row.get("reason") == "build-finished"
    ] != [True]:
        raise ValueError("test build requires one successful Cargo finish")
    key = (owners[0], target, kind)
    return test_artifacts(messages, [key])[key]
