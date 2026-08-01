import io
import os
import sys

from ruamel.yaml import YAML

ROLE_TYPE = "eink_display_firmware"


def main() -> int:
    path = sys.argv[1]
    version = os.environ["VERSION"]

    yaml = YAML()
    yaml.preserve_quotes = True
    yaml.width = 4096
    yaml.indent(mapping=2, sequence=4, offset=2)

    with open(path) as f:
        devices = yaml.load(f)

    updated = 0
    for device in devices:
        for role in device.get("roles", []):
            if role.get("type") == ROLE_TYPE:
                role["config"]["firmware_version"] = version
                updated += 1

    if updated == 0:
        print(f"no {ROLE_TYPE} roles found in {path}", file=sys.stderr)
        return 1

    buffer = io.StringIO()
    yaml.dump(devices, buffer)

    dedented = "\n".join(
        line[2:] if line.startswith("  ") else line
        for line in buffer.getvalue().split("\n")
    )

    with open(path, "w") as f:
        f.write(dedented)

    print(f"set firmware_version to {version} for {updated} displays")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
