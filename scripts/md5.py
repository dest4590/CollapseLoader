#!/usr/bin/env python3
"""md5.py – Compute MD5 hash of a file (default: .jar)

Usage:
    python scripts/md5.py                  # interactive menu
    python scripts/md5.py <path-to-file>   # direct mode
"""

import hashlib
import sys
from pathlib import Path


def compute_md5(filepath: Path) -> str:
    h = hashlib.md5()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


def pick_file() -> Path | None:
    """Interactive file picker."""
    jar_files = list(Path(".").rglob("*.jar"))
    jar_files += list(Path("..").rglob("*.jar"))

    if not jar_files:
        print("No .jar files found in current or parent directory.")
        return None

    print("\nAvailable .jar files:\n")
    for i, f in enumerate(jar_files, 1):
        print(f"  {i}) {f}")

    print(f"\n  0) Cancel\n")
    try:
        choice = int(input("Select file: "))
    except (ValueError, EOFError):
        return None

    if choice == 0 or choice > len(jar_files):
        return None

    return jar_files[choice - 1]


def main():
    if len(sys.argv) > 1:
        filepath = Path(sys.argv[1]).resolve()
    else:
        picked = pick_file()
        if not picked:
            print("Cancelled.")
            sys.exit(0)
        filepath = picked.resolve()

    if not filepath.exists():
        print(f"File not found: {filepath}")
        sys.exit(1)

    print(f"\nComputing MD5 for {filepath.name}...")
    digest = compute_md5(filepath)
    print(f"MD5({filepath.name}) = {digest}")


if __name__ == "__main__":
    main()
