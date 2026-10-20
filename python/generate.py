#!/usr/bin/env python3
"""Generate a static catalog.json from the provider configs in ../configs.

Mirrors OpenCode's model catalog schema: a JSON array of providers. The
generated artifact is what the user hosts; the Rust client fetches it from
CATALOG_URL.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

RE_MODEL_CODE = re.compile(
    r"^[a-z0-9]+(?:-[a-z0-9]+)*"  # org: lowercase kebab-case
    r"/[A-Za-z0-9]+(?:[.-][A-Za-z0-9]+)*"  # model: kebab-case, dots allowed
    r"(?::[a-z0-9]+(?:-[a-z0-9]+)*)?$"  # optional variant: lowercase kebab-case
)

RE_TIME = re.compile(r"^(?:[01][0-9]|2[0-3]):[0-5][0-9]$")

PEAK_EXEMPTIONS = {"weekends", "holidays-cn", "holidays-us"}
PEAK_KEYS = {"timezone", "periods", "exempt"}

REQUIRED_MODEL_FIELDS = {
    "id",
    "modelCode",
    "name",
    "reasoning",
    "attachment",
    "limit",
    "cost",
}


def known_timezones() -> set:
    try:
        from zoneinfo import available_timezones
    except ImportError:
        return set()
    return available_timezones()


ZONEINFO_NAMES = known_timezones()


def validate_timezone(value: object, where: str) -> None:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{where}: 'peaks.timezone' must be a non-empty IANA name")
    if ZONEINFO_NAMES and value not in ZONEINFO_NAMES:
        raise ValueError(f"{where}: unknown peaks.timezone {value!r}")


def validate_peaks(peaks: object, where: str) -> None:
    if not isinstance(peaks, dict):
        raise ValueError(f"{where}: 'peaks' must be an object")
    unknown = set(peaks) - PEAK_KEYS
    if unknown:
        raise ValueError(f"{where}: unknown 'peaks' keys {sorted(unknown)}")

    if "timezone" in peaks:
        validate_timezone(peaks["timezone"], where)

    periods = peaks.get("periods", [])
    if not isinstance(periods, list):
        raise ValueError(f"{where}: 'peaks.periods' must be a list")
    for index, period in enumerate(periods):
        if not isinstance(period, dict):
            raise ValueError(f"{where}: peaks.periods[{index}] must be an object")
        unknown = set(period) - {"start", "end"}
        if unknown:
            raise ValueError(
                f"{where}: unknown peaks.periods[{index}] keys {sorted(unknown)}"
            )
        times = []
        for key in ("start", "end"):
            value = period.get(key)
            if not isinstance(value, str) or not RE_TIME.match(value):
                raise ValueError(
                    f"{where}: peaks.periods[{index}].{key} must be `HH:MM`,"
                    f" got {value!r}"
                )
            times.append(value)
        if times[0] == times[1]:
            raise ValueError(
                f"{where}: peaks.periods[{index}] must span a non-zero amount of time"
            )

    exempt = peaks.get("exempt", [])
    if not isinstance(exempt, list):
        raise ValueError(f"{where}: 'peaks.exempt' must be a list")
    for value in exempt:
        if not isinstance(value, str) or value not in PEAK_EXEMPTIONS:
            raise ValueError(
                f"{where}: peaks.exempt value {value!r} not in"
                f" {sorted(PEAK_EXEMPTIONS)}"
            )


def load_provider(path: Path) -> dict:
    with path.open() as fh:
        provider = json.load(fh)

    if not isinstance(provider, dict):
        raise ValueError(f"{path}: expected a JSON object")
    if "id" not in provider or "name" not in provider:
        raise ValueError(f"{path}: missing 'id' or 'name'")
    if not provider.get("doc"):
        raise ValueError(f"{path}: missing 'doc'")
    if "peaks" in provider:
        validate_peaks(provider["peaks"], f"{path}: provider")

    models = provider.get("models", [])
    if not isinstance(models, list):
        raise ValueError(f"{path}: 'models' must be a list")

    ids = set()
    for model in models:
        if not isinstance(model, dict):
            raise ValueError(f"{path}: each model must be an object")
        missing = REQUIRED_MODEL_FIELDS - set(model)
        if missing:
            raise ValueError(f"{path}: model {model.get('id')!r} missing {sorted(missing)}")
        code = model["modelCode"]
        if not isinstance(code, str) or not RE_MODEL_CODE.match(code):
            raise ValueError(f"{path}: model {model['id']!r} has invalid modelCode {code!r}")
        if model["id"] in ids:
            raise ValueError(f"{path}: duplicate model id {model['id']!r}")
        ids.add(model["id"])
        if "peaks" in model:
            validate_peaks(model["peaks"], f"{path}: model {model['id']!r}")

    for default in ("default_large_model_id", "default_small_model_id"):
        value = provider.get(default)
        if value is not None and value not in ids:
            raise ValueError(f"{path}: {default} {value!r} not in models")

    return provider


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--configs",
        type=Path,
        default=Path(__file__).resolve().parent.parent / "configs",
        help="directory of provider config JSON files",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).resolve().parent.parent / "catalog.json",
        help="output catalog JSON path",
    )
    args = parser.parse_args()

    providers = []
    for path in sorted(args.configs.iterdir()):
        if not path.is_file():
            continue

        providers.append(load_provider(path))

    args.output.write_text(json.dumps(providers, indent=2) + "\n")
    print(f"wrote {len(providers)} providers to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
