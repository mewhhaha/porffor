"""Strict libtest inventory and terminal-summary framing for native runners."""

import re


def test_names(output):
    return [
        line.removesuffix(": test")
        for line in output.splitlines()
        if line.endswith(": test")
    ]


def summary_counts(output):
    return re.findall(
        r"^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;", output, re.M
    )


def parse_inventory(output: str, *, allow_empty=False, sort=True) -> list[str]:
    names = test_names(output)
    totals = re.findall(r"(?m)^(\d+) tests?, (\d+) benchmarks?$", output)
    if len(totals) != 1 or int(totals[0][0]) != len(names) or int(totals[0][1]) != 0:
        raise ValueError(
            "libtest inventory is incomplete or contains unexpected benchmarks"
        )
    if (not names and not allow_empty) or len(set(names)) != len(names):
        raise ValueError("libtest inventory must be nonempty and contain unique names")
    return sorted(names) if sort else names


def result_summaries(output):
    return re.findall(
        r"(?m)^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
        r"(\d+) measured; (\d+) filtered out; finished in .+$",
        output,
    )


def passed_exactly_one(output: str, total: int) -> bool:
    return result_summaries(output) == [("ok", "1", "0", "0", "0", str(total - 1))]
