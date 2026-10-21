"""Explicit ASCII resource counts, bounded before integer conversion."""


def positive_decimal(value, label):
    if (
        not isinstance(value, str)
        or not value.isascii()
        or not value.isdecimal()
        or len(value) > 20
        or int(value) == 0
    ):
        raise ValueError(f"{label} must be a positive decimal integer")
    return int(value)
