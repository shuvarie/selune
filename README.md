# Selune

Selune ([Etymology link](https://forgottenrealms.fandom.com/wiki/Sel%C3%BBne)) is a simple registry for collecting providers and models supported by [Shuvarie](https://github.com/shuvarie/shuvarie).

The design of Selune is influenced by [Catwalk](https://github.com/charmbracelet/catwalk) and OpenCode Models.

## Model codes

Every model carries a `modelCode` — `org/model` or `org/model:variant` — that
identifies the same model across providers (for example
`zai-org/GLM-5.3-Flash:high`). The org is the training organization's id in
Hugging Face/GitHub style (lowercase kebab-case); the model id is kebab-case,
case sensitive, dots allowed; the optional variant is a lowercase serving
qualifier (`fast`, `free`, `high`, `256k`, ...). Codes are derived and kept in
sync by `python/model_codes.py` and validated by `python/generate.py`.
