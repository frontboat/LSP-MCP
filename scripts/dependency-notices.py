"""Collect upstream notices for the crates and Rust library in a bridge binary."""

import json
import shutil
import subprocess
import sys
from collections import defaultdict
from pathlib import Path


def output(*command):
    return subprocess.check_output(command, text=True).strip()


destination = Path(sys.argv[1])
crates = {}
for target in sys.argv[2:]:
    tree = output("cargo", "tree", "--locked", "--target", target, "-e", "normal",
                  "--prefix", "none", "--format", "{p}")
    names = {tuple(line.split()[:2]) for line in tree.splitlines()}
    metadata = json.loads(output("cargo", "metadata", "--locked", "--format-version", "1",
                                 "--filter-platform", target))
    for package in metadata["packages"]:
        key = (package["name"], "v" + package["version"])
        if key in names and package["source"]:
            crates[" ".join(key)] = Path(package["manifest_path"]).parent

notices = defaultdict(list)
for crate, root in sorted(crates.items()):
    files = sorted(path for path in root.iterdir() if path.is_file() and
                   path.name.lower().startswith(("license", "licence", "copying", "notice", "copyright")))
    if not files:
        sys.exit(f"No upstream license files found for {crate}")
    for path in files:
        notices[path.read_text(encoding="utf-8").strip()].append(crate + " / " + path.name)

with (destination / "THIRD-PARTY-NOTICES.txt").open("w", encoding="utf-8") as out:
    out.write(Path("LICENSE").read_text(encoding="utf-8").strip() + "\n")
    out.write("\nRust standard library notices accompany this file in RUST-LIBRARY-LICENSES.html.\n")
    for notice, origins in notices.items():
        out.write("\n" + "=" * 72 + "\n" + "\n".join(origins) + "\n\n" + notice + "\n")

sysroot = Path(output("rustc", "--print", "sysroot"))
shutil.copyfile(sysroot / "share/doc/rust/COPYRIGHT-library.html",
                destination / "RUST-LIBRARY-LICENSES.html")
