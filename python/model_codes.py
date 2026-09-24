#!/usr/bin/env python3
"""Populate `modelCode` on every model in configs/*.json.

A model code identifies a model across providers: `org/model` or
`org/model:variant`. The org is the training organization's id (Hugging Face
or GitHub style, lowercased, e.g. `zai-org`, `moonshotai`); the model id is
kebab-case, case sensitive, dots allowed; the optional variant is a lowercase
serving qualifier (`fast`, `free`, `high`, `256k`, ...).

Strategy: split provider ids into org/model (handling bedrock, fireworks,
ollama, synthetic and Venice id schemes), normalize the org via an alias
table, infer orgs for bare ids from model-family prefixes, strip serving
suffixes into the variant slot, then canonicalize the model-id casing against
a weighted consensus (Hugging Face repo ids win, then official API configs).
Ambiguous ids must be listed in OVERRIDES; the script errors on collisions.

Run with --check to validate without writing.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

RE_ORG = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
RE_MODEL = re.compile(r"^[A-Za-z0-9]+(?:[.-][A-Za-z0-9]+)*$")
RE_VARIANT = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")

# Hosts whose ids mirror upstream naming, and how much their casing counts.
# huggingface.json carries literal upstream repo ids and wins outright; the
# official provider configs (45) outrank aggregators (20).
WEIGHTS = {
    "huggingface": 1000,
    "chutes": 50,
    "baseten": 50,
    "doubleword": 50,
    "ionet": 50,
    "nebius": 50,
    "synthetic": 50,
    "atlascloud": 50,
    "togetherai": 20,
    "openrouter": 20,
    "vercel": 20,
    "openai": 45,
    "chatgpt": 45,
    "anthropic": 45,
    "gemini": 45,
    "vertexai": 45,
    "xai": 45,
    "mistral": 45,
    "cohere": 45,
    "moonshotai": 45,
    "moonshotai-cn": 45,
    "minimax": 45,
    "minimax-cn": 45,
    "zai": 45,
    "zai-coding": 45,
    "zhipu": 45,
    "zhipu-coding": 45,
    "deepseek": 45,
    "alibaba-sg": 45,
    "alibaba-us": 45,
    "xiaomimimo": 45,
    "perplexity": 45,
    "voyageai": 45,
    "kimi-code": 45,
}

ORG_ALIASES = {
    "z-ai": "zai-org",
    "zai": "zai-org",
    "x-ai": "xai-org",
    "spacexai": "xai-org",
    "deepseek": "deepseek-ai",
    "alibaba": "qwen",
    "minimax": "minimaxai",
    "xiaomi": "xiaomimimo",
    "meituan": "meituan-longcat",
    "liquidai": "liquid",
    "meta": "meta-llama",
    "stepfun": "stepfun-ai",
    "bytedance": "bytedance-seed",
    "mistral": "mistralai",
    "perplexity": "perplexity-ai",
}

# Bare-id leading prefixes that name the hosting org, not the model. Only
# stripped when the remainder matches a model family (the family's org wins).
ORG_PREFIXES = [
    "meituan-longcat",
    "meta-llama",
    "deepseek-ai",
    "zai-org",
    "z-ai",
    "zai",
    "moonshotai",
    "minimaxai",
    "xiaomimimo",
    "mistralai",
    "stepfun",
    "bytedance",
    "tencent",
    "baidu",
    "cohere",
    "nvidia",
    "xiaomi",
    "google",
    "openai",
    "aihub",
    "aihubmix",
    "qwen",
    "x-ai",
]

# Model-family prefix -> (training org, strip prefix from model id). Order
# matters: longest / most specific first.
FAMILY = [
    ("doubao-deepseek", "deepseek-ai", True),
    ("minimax", "minimaxai", False),
    ("kimi", "moonshotai", False),
    ("k2.", "moonshotai", False),
    ("k3", "moonshotai", False),
    ("glm", "zai-org", False),
    ("deepseek", "deepseek-ai", False),
    ("qwen", "qwen", False),
    ("gemini", "google", False),
    ("gemma", "google", False),
    ("claude", "anthropic", False),
    ("gpt-", "openai", False),
    ("o1", "openai", False),
    ("o3", "openai", False),
    ("o4", "openai", False),
    ("grok", "xai-org", False),
    ("ministral", "mistralai", False),
    ("pixtral", "mistralai", False),
    ("magistral", "mistralai", False),
    ("voxtral", "mistralai", False),
    ("codestral", "mistralai", False),
    ("devstral", "mistralai", False),
    ("mixtral", "mistralai", False),
    ("open-mistral", "mistralai", False),
    ("mistral", "mistralai", False),
    ("llama", "meta-llama", False),
    ("nemotron", "nvidia", False),
    ("hy3", "tencent", False),
    ("doubao", "bytedance-seed", False),
    ("seed-", "bytedance-seed", False),
    ("seedoss", "bytedance-seed", False),
    ("ernie", "baidu", False),
    ("step-", "stepfun-ai", False),
    ("stepfun", "stepfun-ai", False),
    ("mimo", "xiaomimimo", False),
    ("longcat", "meituan-longcat", False),
    ("command-", "cohere", False),
    ("c4ai", "cohere", False),
    ("aya-", "cohere", False),
    ("north-", "cohere", False),
    ("sonar", "perplexity-ai", False),
    ("phi-", "microsoft", False),
    ("jina", "jinaai", False),
    ("lfm", "liquid", False),
    ("muse", "meta-llama", False),
    ("mercury", "inception", False),
    ("inkling", "thinkingmachines", False),
    ("kat-", "kwaipilot", False),
    ("dots-", "dots-studio", False),
    ("ling-", "inclusionai", False),
    ("nova-", "amazon", False),
    ("apertus", "swiss-ai", False),
    ("hermes", "nousresearch", False),
    ("cosmos", "nvidia", False),
    ("solar", "upstage", False),
    ("fugu", "sakana", False),
    ("namazu", "sakana", False),
    ("allam", "sdaia", False),
    ("whisper", "openai", False),
    ("compound", "groq", False),
    ("cogito", "deepcogito", False),
    ("voyage", "voyageai", False),
    ("granite", "ibm-granite", False),
    ("nex-n2", "nex-agi", False),
    ("laguna", "poolside", False),
    ("unslopnemo", "thedrummer", False),
    ("orpheus", "canopylabs", False),
    ("rnj", "essentialai", False),
    ("aion", "aion-labs", False),
    ("prompt-guard", "meta-llama", False),
    ("interfaze", "interfaze", False),
    ("relace", "relace", False),
    ("sakana", "sakana", False),
    ("big-pickle", "opencode", False),
    ("ox-", "ox", False),
    ("agnes", "agnes", False),
    ("venice", "venice", False),
    ("saba", "mistralai", False),
    ("aya", "cohere", False),
    ("text-embedding", "openai", False),
    ("codex", "openai", False),
    ("cohere-embed", "cohere", False),
    ("model-router", "microsoft", False),
    ("intellect", "primeintellect", False),
]

DEFAULT_ORG = {
    "openai": "openai",
    "chatgpt": "openai",
    "anthropic": "anthropic",
    "gemini": "google",
    "vertexai": "google",
    "zai": "zai-org",
    "zai-coding": "zai-org",
    "zhipu": "zai-org",
    "zhipu-coding": "zai-org",
    "moonshotai": "moonshotai",
    "moonshotai-cn": "moonshotai",
    "minimax": "minimaxai",
    "minimax-cn": "minimaxai",
    "deepseek": "deepseek-ai",
    "cohere": "cohere",
    "mistral": "mistralai",
    "xai": "xai-org",
    "perplexity": "perplexity-ai",
    "voyageai": "voyageai",
    "xiaomimimo": "xiaomimimo",
    "groq": "groq",
    "kimi-code": "moonshotai",
    "aihubmix": "aihubmix",
    "alibaba-sg": "qwen",
    "alibaba-us": "qwen",
    "venice": "venice",
}

# Serving qualifiers stripped into the variant when the base model exists.
VARIANT_SUFFIXES = {
    "fast",
    "free",
    "highspeed",
    "tee",
    "cloud",
    "think",
    "nothink",
    "search",
    "lightning",
    "ultraspeed",
    "tput",
    "nvfp4",
    "fp8",
    "256k",
    "128b",
    "int4-mixed-ar",
    "flex",
    "short",
    "disc",
    "instant",
    "versatile",
    "high",
    "low",
    "us",
}

# Normalized keys that are not the literal upstream model id, mapping to the
# canonical key used by the authoritative sources.
KEY_ALIASES = {
    "claude-opus4-5": "claude-opus-4-5",
    "claude-opus4-6": "claude-opus-4-6",
    "claude-opus4-7": "claude-opus-4-7",
    "claude-opus4-8": "claude-opus-4-8",
    "claude-4-5-sonnet": "claude-sonnet-4-5",
    "claude-4-6-sonnet": "claude-sonnet-4-6",
    "kimi-k2": "kimi-k2-instruct",
    "kimi-k2-0905": "kimi-k2-instruct-0905",
    "seed-1-6-flash": "doubao-seed-1-6-flash",
    "seed-2-0-code": "doubao-seed-2-0-code",
    "seed-2-0-mini": "doubao-seed-2-0-mini",
    "seed-1-8": "doubao-seed-1-8",
    "seed-2-1-turbo": "doubao-seed-2-1-turbo",
    "llama-3-3-70b": "llama-3-3-70b-instruct",
    "nova-2-lite-v1": "nova-2-lite",
    "nova-lite-v1": "nova-lite",
    "nova-micro-v1": "nova-micro",
    "nova-pro-v1": "nova-pro",
    "north-mini-code": "north-mini-code-1-0",
    "nemotron-3-5-lightning": "nemotron-lightning-3-5-30b-a3b",
    "nemotron-3-ultra": "nemotron-3-ultra-550b-a55b",
    "nemotron-3-super": "nemotron-3-super-120b-a12b",
    "nemotron-3-nano-30b": "nemotron-3-nano-30b-a3b",
    "k2-6-code-preview": "kimi-k2-6-code-preview",
    "qwen-3-6-plus": "qwen3-6-plus",
    "qwen-3-6-max-preview": "qwen3-6-max-preview",
    "qwen-3-7-plus": "qwen3-7-plus",
    "qwen-3-7-max": "qwen3-7-max",
    "qwen-3-8-max": "qwen3-8-max",
    "qwen-3-8-flash": "qwen3-8-flash",
    "qwen-3-8-27b": "qwen3-8-27b",
    "qwen-3-8-2-4t-a95b": "qwen3-8-2-4t-a95b",
    "qwen-3-32b": "qwen3-32b",
}

# Explicit (provider, id) -> (org, model, variant) for ids the generic rules
# cannot resolve unambiguously.
OVERRIDES = {
    ("kimi-code", "k3"): ("moonshotai", "Kimi-K3", None),
    ("kimi-code", "k3-256k"): ("moonshotai", "Kimi-K3", "256k"),
    ("kimi-code", "kimi-for-coding"): ("moonshotai", "kimi-for-coding", None),
    ("kimi-code", "kimi-for-coding-highspeed"): ("moonshotai", "kimi-for-coding", "highspeed"),
    ("llamafile", "LLaMA_CPP"): ("mozilla-ai", "llamafile", None),
    ("synthetic", "syn:large:text"): ("synthetic", "syn-large-text", None),
    ("synthetic", "syn:large:vision"): ("synthetic", "syn-large-vision", None),
    ("synthetic", "syn:small:text"): ("synthetic", "syn-small-text", None),
    ("synthetic", "syn:small:vision"): ("synthetic", "syn-small-vision", None),
    ("synthetic", "hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4"): (
        "nvidia",
        "NVIDIA-Nemotron-3-Super-120B-A12B",
        "nvfp4",
    ),
    ("aihubmix", "anthropic-opus-4-6"): ("anthropic", "claude-opus-4-6", None),
    ("aihubmix", "glm-5.2-fast-preview"): ("zai-org", "GLM-5.2", "fast-preview"),
    ("aihubmix", "ox-alpha"): ("ox", "ox-alpha", None),
    ("opencode-go", "ox-alpha-free"): ("ox", "ox-alpha", "free"),
    ("aihubmix", "agnes-2.5-flash"): ("agnes", "agnes-2.5-flash", None),
    ("aihubmix", "agnes-2.5-pro"): ("agnes", "agnes-2.5-pro", None),
    ("aihubmix", "agnes-2.5-pro-alpha"): ("agnes", "agnes-2.5-pro", "alpha"),
    ("aihubmix", "kat-dev"): ("kwaipilot", "kat-dev", None),
    ("aihubmix", "auto"): ("aihubmix", "auto", None),
    ("opencode-zen", "big-pickle"): ("opencode", "big-pickle", None),
    ("groq", "allam-2-7b"): ("sdaia", "allam-2-7b", None),
    ("venice", "venice-uncensored-1-2"): ("venice", "venice-uncensored", "1-2"),
    ("venice", "venice-uncensored-role-play"): ("venice", "venice-uncensored", "role-play"),
    ("venice", "gemma-4-uncensored"): ("venice", "gemma-4-uncensored", None),
    ("venice", "olafangensan-glm-4.7-flash-heretic"): ("olafangensan", "glm-4.7-flash-heretic", None),
    ("venice", "kimi-k3-fast-api"): ("moonshotai", "Kimi-K3", "fast"),
    ("venice", "mercury-2-5"): ("inception", "mercury-2.5", None),
    ("venice", "qwen3-next-80b"): ("qwen", "qwen3-next-80b-a3b-instruct", None),
    ("venice", "qwen3-vl-235b-a22b"): ("qwen", "qwen3-vl-235b-a22b-instruct", None),
    ("venice", "llama-3.2-3b"): ("meta-llama", "llama-3.2-3b", None),
    ("ionet", "meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8"): (
        "meta-llama",
        "Llama-4-Maverick-17B-128E-Instruct",
        "fp8",
    ),
    ("azure", "llama-4-maverick-17b-128e-instruct-fp8"): (
        "meta-llama",
        "Llama-4-Maverick-17B-128E-Instruct",
        "fp8",
    ),
    ("cortecs", "cosmos3-super-reasoner"): ("nvidia", "cosmos3-super-reasoner", None),
    ("chutes", "google/gemma-4-31B-turbo-TEE"): ("google", "gemma-4-31B-turbo", "tee"),
    ("ollama-cloud", "mistral-large-3:675b"): ("mistralai", "mistral-large-3", "675b"),
}

VENICE_ORG_PREFIXES = [
    "aion-labs-",
    "olafangensan-",
    "zai-org-",
    "openai-",
    "google-",
    "z-ai-",
]


def key_of(model: str) -> str:
    return model.lower().replace(".", "-").replace("_", "-")


def split_org_model(raw: str) -> tuple[str | None, str]:
    if "/" in raw and not raw.startswith("accounts/"):
        org, _, model = raw.partition("/")
        if org and model and "/" not in model:
            return org, model
    return None, raw


def decode(provider: str, mid: str) -> tuple[str | None, str, list[str]]:
    """Decode a provider model id into a raw (org, model, forced variants)."""
    if provider in ("bedrock", "bedrock-eu"):
        body = re.sub(r"^(?:us|eu)\.anthropic\.", "", mid)
        body = re.sub(r"-v1(?::0)?$", "", body)
        return "anthropic", body, []
    if provider == "fireworks":
        body = re.sub(r"^accounts/fireworks/models/", "", mid)
        body = re.sub(r"(\d)p(\d)", r"\1.\2", body)
        return (*split_org_model(body), [])
    if provider == "synthetic":
        if mid.startswith("hf:"):
            return (*split_org_model(mid[len("hf:") :]), [])
        return None, mid, []
    if provider == "ollama-cloud":
        body, sep, tag = mid.partition(":")
        if sep:
            return (*split_org_model(f"{body}-{tag}"), [])
        return (*split_org_model(mid), [])
    if provider == "venice":
        for prefix in VENICE_ORG_PREFIXES:
            if mid.startswith(prefix):
                org = prefix.rstrip("-").lower()
                return ORG_ALIASES.get(org, org), mid[len(prefix) :], []
        return None, mid, []
    body, sep, tag = mid.rpartition(":")
    if sep and re.fullmatch(r"[a-z0-9-]+", tag):
        return (*split_org_model(body), [tag])
    return (*split_org_model(mid), [])


def family_match(model: str) -> tuple[str, str, bool] | None:
    """Match a model id against the family table.
    Returns (matched_prefix, org, strip_flag)."""
    key = key_of(model)
    for fam, org, strip in FAMILY:
        if key.startswith(key_of(fam)):
            return fam, org, strip
    return None


def resolve(provider: str, org: str | None, model: str, flags: list) -> tuple[str | None, str]:
    """Resolve the org and possibly strip an embedded hosting prefix.
    Returns (org, model)."""
    if org is not None:
        org = org.lower()
        return ORG_ALIASES.get(org, org), model
    for prefix in ORG_PREFIXES:
        if model.lower().startswith(prefix.lower() + "-") and len(model) > len(prefix) + 1:
            rest = model[len(prefix) + 1 :]
            match = family_match(rest)
            if match is not None:
                return match[1], rest
    match = family_match(model)
    if match is None and model.lower().startswith("coding-"):
        match = family_match(model[len("coding-") :])
    if match is not None:
        fam, org, strip = match
        if strip:
            return org, model.split("-", 1)[1]
        return org, model
    default = DEFAULT_ORG.get(provider)
    if default:
        return default, model
    flags.append((provider, model, "no org found"))
    return None, model


def strip_dates_and_versions(model: str, raw_keys: set[str]) -> tuple[str, bool]:
    """Strip API-alias date/version suffixes when the bare form exists.
    Returns (model, stripped_flag)."""
    for pattern in (r"-\d{8}$", r"-\d{2}-\d{4}$", r"-\d{4}-\d{2}-\d{2}$", r"-\d{6}$"):
        candidate = re.sub(pattern, "", model)
        if candidate == model or not candidate:
            continue
        if key_of(candidate) in raw_keys:
            return candidate, True
    return model, False


def extract_variant(model: str, raw_keys: set[str]) -> tuple[str, list[str]]:
    """Strip serving suffixes whose base exists; returns (model, tokens)."""

    def suffix_loop(m: str) -> tuple[str, list[str]]:
        tokens: list[str] = []
        current = m
        for _ in range(3):
            if "-" not in current:
                break
            base, _, suffix = current.rpartition("-")
            if suffix.lower() not in VARIANT_SUFFIXES:
                break
            if key_of(base) not in raw_keys:
                break
            tokens.append(suffix.lower())
            current = base
        return current, tokens

    # Leading aihubmix coding-plan prefix: `coding-<model>` -> variant `coding`.
    if model.lower().startswith("coding-"):
        base, tokens = suffix_loop(model[len("coding-") :])
        if key_of(base) in raw_keys:
            return base, ["coding"] + list(reversed(tokens))
    current, tokens = suffix_loop(model)
    if current.lower().startswith("coding-"):
        rest = current[len("coding-") :]
        if key_of(rest) in raw_keys:
            tokens.append("coding")
            current = rest
    tokens.reverse()
    return current, tokens


def fuzzy_match(model: str, canonical_keys: set[str]) -> str | None:
    """Match a flattened model id (dots lost) against canonical keys."""
    key = key_of(model)
    for m in re.finditer(r"\d+", key):
        start, end = m.span()
        if end - start < 2:
            continue
        for cut in range(start + 1, end):
            candidate = key[:cut] + "-" + key[cut:]
            if candidate in canonical_keys:
                return candidate
    for m in re.finditer(r"[a-z]\d", key):
        cut = m.end() - 1
        candidate = key[:cut] + "-" + key[cut:]
        if candidate in canonical_keys:
            return candidate
    return None


def compute_codes(entries: list[dict], casing: dict[str, dict[str, int]]) -> None:
    for e in entries:
        raw_key = key_of(e["model"])
        key = KEY_ALIASES.get(raw_key, raw_key)
        if e["provider"] in ("venice", "ollama-cloud"):
            fuzzy = fuzzy_match(e["model"], set(casing.keys()))
            if fuzzy is not None:
                key = fuzzy
        forms = casing.get(key)
        if forms:
            canonical = sorted(forms.items(), key=lambda kv: (kv[1][1], -kv[1][0], kv[0]))[0][0]
        else:
            canonical = e["model"]
        e["key"] = key
        e["canonical"] = canonical
        e["code_str"] = f"{e['org']}/{canonical}" + (f":{'-'.join(e['tokens'])}" if e["tokens"] else "")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--configs", type=Path, default=Path(__file__).resolve().parent.parent / "configs")
    parser.add_argument("--check", action="store_true", help="validate only, do not write")
    parser.add_argument("--dump", metavar="PROVIDER", help="print resolved model codes for one provider and exit")
    args = parser.parse_args()

    configs: list[tuple[Path, str, dict]] = []
    for path in sorted(args.configs.glob("*.json")):
        provider = json.loads(path.read_text())
        configs.append((path, provider.get("id", path.stem), provider))

    flags: list[tuple[str, str, str]] = []

    # Pass 1: decode every id into (org_raw, model_raw); build the raw key set.
    entries = []
    for path, provider, data in configs:
        for model in data.get("models", []):
            mid = model["id"]
            if (provider, mid) in OVERRIDES:
                org, name, variant = OVERRIDES[(provider, mid)]
                entries.append(
                    {
                        "provider": provider,
                        "mid": mid,
                        "org": org,
                        "model": name,
                        "tokens": [variant] if variant else [],
                        "forced": [],
                        "override": True,
                    }
                )
                continue
            org, model, forced = decode(provider, mid)
            entries.append(
                {
                    "provider": provider,
                    "mid": mid,
                    "org": org,
                    "model": model,
                    "tokens": [],
                    "forced": forced,
                    "override": False,
                }
            )
    raw_keys = {key_of(e["model"]) for e in entries}

    # Pass 2: strip API date/version suffixes.
    for e in entries:
        if e["override"]:
            continue
        e["model"], _ = strip_dates_and_versions(e["model"], raw_keys)

    # Pass 3: resolve orgs.
    for e in entries:
        if e["override"]:
            continue
        org, model = resolve(e["provider"], e["org"], e["model"], flags)
        e["org"], e["model"] = org, model

    # Pass 4: strip serving suffixes into the variant slot.
    for e in entries:
        if e["override"]:
            continue
        model, tokens = extract_variant(e["model"], raw_keys)
        if e["provider"] == "chutes" and model.lower().endswith("-tee") and not tokens:
            tokens, model = ["tee"], model[: -len("-tee")]
        e["model"], e["tokens"] = model, tokens + e["forced"]

    # Pass 5: canonical casing consensus (tiered: HF > official API > others),
    # then assemble codes. Intra-provider same-code entries are notes: they are
    # the same model offered through different endpoints.
    for _ in range(5):
        casing: dict[str, dict[str, tuple[int, int]]] = {}
        for e in entries:
            form = e["model"]
            weight = WEIGHTS.get(e["provider"], 1)
            tier = 0 if e["provider"] == "huggingface" else 1 if weight >= 45 else 2
            key = key_of(form)
            casing.setdefault(key, {})
            total, min_tier = casing[key].get(form, (0, 2))
            casing[key][form] = (total + weight, min(min_tier, tier))
        compute_codes(entries, casing)

    # Validate + collision check.
    errors: list[str] = []
    notes: list[str] = []
    seen: dict[tuple[str, str], list[str]] = {}
    for e in entries:
        if e["org"] is None or not RE_ORG.match(e["org"]):
            errors.append(f"{e['provider']}: {e['mid']!r}: bad org {e['org']!r}")
        if not RE_MODEL.match(e["canonical"]):
            errors.append(f"{e['provider']}: {e['mid']!r}: bad model id {e['canonical']!r}")
        if e["tokens"] and not RE_VARIANT.match("-".join(e["tokens"])):
            errors.append(f"{e['provider']}: {e['mid']!r}: bad variant {'-'.join(e['tokens'])!r}")
        seen.setdefault((e["provider"], e["code_str"]), []).append(e["mid"])

    for (provider, code), ids in sorted(seen.items()):
        if len(ids) > 1:
            notes.append(f"{provider}: same model offered twice: {code}: {ids}")

    if flags:
        for flag in flags:
            errors.append(f"flag: {flag[0]}: {flag[1]!r} ({flag[2]})")

    if errors:
        print(f"{len(errors)} problem(s):")
        for err in errors:
            print(f"  - {err}")
        return 1
    if notes:
        print(f"{len(notes)} note(s):")
        for note in notes:
            print(f"  - {note}")

    if args.check:
        print("check ok")
        return 0

    if args.dump:
        for e in entries:
            if e["provider"] == args.dump:
                print(f"  {e['mid']!r:70} -> {e['code_str']}")
        return 0

    for path, provider, data in configs:
        by_mid = {e["mid"]: e["code_str"] for e in entries if e["provider"] == provider}
        rebuilt = []
        for model in data.get("models", []):
            item = {}
            for k, v in model.items():
                item[k] = v
                if k == "id":
                    item["modelCode"] = by_mid[model["id"]]
            rebuilt.append(item)
        data["models"] = rebuilt
        path.write_text(json.dumps(data, indent=2) + "\n")

    total = len(entries)
    with_variant = sum(1 for e in entries if e["tokens"])
    print(f"wrote modelCode for {total} models ({with_variant} with variant)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())