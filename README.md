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

## Peak and off-peak pricing

Some providers charge different rates during peak hours. `peaks` can be set on
a provider, where it applies to every model, or on a single model, where it
replaces the provider-wide schedule entirely, timezone included.

`timezone` is the IANA zone — `Asia/Shanghai`, `America/New_York`, `UTC` — in
which the whole schedule is written. It defaults to `UTC` and is omitted from
the JSON when it is UTC. Peak windows are times of day in that zone with
`start` inclusive and `end` exclusive; a window whose `end` is earlier than
its `start` wraps past midnight (for example `23:00`–`01:00`). `exempt` lists
the days on which the provider runs no peak/off-peak schedule at all:
`weekends`, `holidays-cn` or `holidays-us`.

```json
"peaks": {
  "timezone": "Asia/Shanghai",
  "periods": [
    { "start": "09:00", "end": "12:00" },
    { "start": "14:00", "end": "18:00" }
  ],
  "exempt": ["weekends", "holidays-cn"]
}
```

The schedule is evaluated with `Provider::peaks_for`, `Provider::is_peak`, or
directly with `Peaks::is_peak`; `Peaks::local` shows an instant in the
schedule's zone. Weekdays, holiday dates and windows are all resolved in
`timezone`, so a Chinese holiday starts at Shanghai midnight rather than UTC
midnight, and daylight saving transitions are handled for zones that observe
them. Chinese and US public holidays come from
[holidays2](https://github.com/chardoncs/holidays2) (2001–2030, and dates
outside that range are treated as non-holidays).

Peak windows and exemptions are separate: on an exempt day no time is peak,
even for a window that started the evening before.
