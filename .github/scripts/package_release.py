"""Package the release executable with the assets it loads beside the executable."""

import argparse
import hashlib
import os
import re
import shutil
import tarfile
import tomllib
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PLATFORMS = {
    "x86_64-pc-windows-msvc": "windows-x64",
    "aarch64-apple-darwin": "macos-arm64",
    "x86_64-unknown-linux-gnu": "linux-x64",
}


def version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as manifest:
        value = tomllib.load(manifest)["package"]["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", value):
        raise ValueError(f"Invalid release version: {value}")
    if os.environ.get("GITHUB_REF_TYPE") == "tag":
        tag = os.environ["GITHUB_REF_NAME"]
        if tag != f"v{value}":
            raise ValueError(f"Tag {tag} does not match Cargo.toml version v{value}")
    return value


def package(target: str, binary: Path | None, output: Path) -> Path:
    release_version = version()
    windows = target.endswith("windows-msvc")
    executable = "gpui_idea.exe" if windows else "gpui_idea"
    binary = binary or ROOT / "target" / target / "release" / executable
    if not binary.is_file():
        raise FileNotFoundError(f"Release executable not found: {binary}")

    name = f"gpui_idea-v{release_version}-{PLATFORMS[target]}"
    # Each packaging attempt owns a new directory; never delete an existing path.
    output.mkdir(parents=True, exist_ok=True)
    staging = output / "staging" / name
    staging.mkdir(parents=True, exist_ok=False)
    executable_directory = staging
    destination = executable_directory / executable
    shutil.copy2(binary, destination)
    if not windows:
        destination.chmod(0o755)
    # The application keeps data/logs/tools beside the executable. Portable
    # packages preserve that layout without modifying a signed .app bundle.
    shutil.copytree(ROOT / "assets", executable_directory / "assets")

    if windows:
        archive = output / f"{name}.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as handle:
            for path in sorted(staging.rglob("*")):
                handle.write(path, path.relative_to(staging.parent))
    else:
        archive = output / f"{name}.tar.gz"
        with tarfile.open(archive, "w:gz") as handle:
            handle.add(staging, arcname=name)

    with archive.open("rb") as handle:
        digest = hashlib.file_digest(handle, "sha256").hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(
        f"{digest}  {archive.name}\n", encoding="utf-8"
    )
    print(archive)
    return archive


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check-version", action="store_true")
    parser.add_argument("--target", choices=PLATFORMS)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    arguments = parser.parse_args()
    if arguments.check_version:
        print(f"Release version: v{version()}")
    elif arguments.target:
        package(arguments.target, arguments.binary, arguments.output)
    else:
        parser.error("--check-version or --target is required")


if __name__ == "__main__":
    main()
