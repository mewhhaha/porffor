"""Check or refresh Intl source identities before expensive Rust verification."""
import argparse
from pathlib import Path
import subprocess
import sys


# Named-zone identities feed country membership and DateTime; DateTime feeds
# locale query provenance. Duration consumes the generated List identity and
# manifest. Keep dependent recipes after their actual producers.
RECIPES = (
    'generate-intl-keyword-aliases.py',
    'generate-intl-collator-data.py',
    'generate-intl-named-time-zones.py',
    'generate-intl-locale-time-zones-profile.py',
    'generate-intl-datetime-identity.py',
    'generate-intl-locale-calendars-profile.py',
    'generate-intl-locale-hour-cycles-profile.py',
    'generate-intl-locale-text-profile.py',
    'generate-intl-locale-week-profile.py',
    'generate-intl-display-relative-identity.py',
    'generate-intl-list-identity.py',
    'generate-intl-duration-identity.py',
    'generate-intl-locale-collations-identity.py',
    'generate-intl-locale-numbering-systems-identity.py',
    'generate-intl-segmenter-identity.py',
)


def refresh(root, recipes=RECIPES, write=False):
    results = []
    for name in recipes:
        command = [sys.executable, '-B', str(root / 'scripts' / name)]
        if write:
            checked = subprocess.run([*command, '--check'], cwd=root,
                                     text=True, capture_output=True)
            if checked.returncode == 0:
                # Some generators rewrite identical output. Keep current
                # timestamps so Cargo does not rebuild unchanged consumers.
                results.append((name, checked))
                continue
        else:
            command.append('--check')
        child = subprocess.run(command, cwd=root, text=True, capture_output=True)
        results.append((name, child))
        # A failed producer cannot provide valid input to dependent writers.
        if write and child.returncode:
            break
    return results


def report(results):
    for name, child in results:
        print(('FAIL ' if child.returncode else 'PASS ') + name)
        if child.returncode:
            print(child.stdout, end='', file=sys.stderr)
            print(child.stderr, end='', file=sys.stderr)
    return any(child.returncode for _, child in results)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path,
                        default=Path(__file__).resolve().parents[1])
    parser.add_argument('--write', action='store_true',
                        help='Regenerate stale identities, then verify every recipe again.')
    args = parser.parse_args()
    root = args.repository.resolve()
    if report(refresh(root, write=args.write)):
        return 1
    if args.write:
        return int(report(refresh(root)))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
