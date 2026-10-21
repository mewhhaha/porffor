"""Measured scheduling data; unknown IDs retain the runner's fallback policy."""

import json
from pathlib import Path


def load_weights():
    data = json.loads(
        Path(__file__).with_name("native-partition-costs.json").read_text()
    )
    if data["schema"] != 1:
        raise ValueError("unknown native partition cost schema")
    weights = {}
    for observation in data["observations"]:
        factor = observation["units_per_second"]
        if type(factor) is not int or factor <= 0:
            raise ValueError("invalid scheduling weight scale")
        for name, seconds in observation["seconds"].items():
            if type(seconds) is not int or seconds <= 0 or name in weights:
                raise ValueError("invalid or duplicate measured native cost")
            weights[name] = factor * seconds
    for name, estimate in data["estimates"].items():
        value = estimate["weight"]
        if type(value) is not int or value <= 0 or name in weights:
            raise ValueError("invalid or duplicate estimated native cost")
        weights[name] = value
    return weights


WEIGHTS = load_weights()


def measured_weight(name):
    return WEIGHTS.get(name)
