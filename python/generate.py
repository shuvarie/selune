#!/usr/bin/env python3
"""Generate a static catalog.json from the provider configs in ../configs.

Mirrors Catwalk's server output: a JSON array of providers. The generated
artifact is what the user hosts; the Rust client fetches it from CATALOG_URL.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Provider configs are read in this order so the output is stable.
ORDER = [
    "anthropic",
    "openai",
    "gemini",
    "deepseek",
    "openrouter",
]

REQUIRED_MODEL_FIELDS = {
    "id",
    "name",
    "cost_per_1m_in",
    "cost_per_1m_out",
    "cost_per_1m_in_cached",
    "cost_per_1m_out_cached",
    "context_window",
    "default_max_tokens",
    "can_reason",
    "supports_attachments",
}


def load_provider(path: Path) -> dict:
    with path.open() as fh:
        provider = json.load(fh)

    if not isinstance(provider, dict):
        raise ValueError(f"{path}: expected a JSON object")
    if "id" not in provider or "name" not in provider:
        raise ValueError(f"{path}: missing 'id' or 'name'")

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
        if model["id"] in ids:
            raise ValueError(f"{path}: duplicate model id {model['id']!r}")
        ids.add(model["id"])

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
    for name in ORDER:
        path = args.configs / f"{name}.json"
        if not path.exists():
            print(f"warning: missing config {path.name}, skipping", file=sys.stderr)
            continue
        providers.append(load_provider(path))

    args.output.write_text(json.dumps(providers, indent=2) + "\n")
    print(f"wrote {len(providers)} providers to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
