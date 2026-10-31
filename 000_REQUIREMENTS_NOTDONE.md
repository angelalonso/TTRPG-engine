# Requirements not yet matched by the program

This document records requirements from `000_REQUIREMENTS_GUI.md`,
`000_REQUIREMENTS_LOGIC.md`, and `NEXT.md` that are not fully represented by
the current implementation. Dataset-specific content belongs in the dataset
folder; the engine should remain generic.

## GUI and configuration

- Intro-plugin content is still dataset/plugin-defined only in principle; the
  requirements document does not yet describe a complete default flow.
- The dashboard forecast now shows the required current/end budget and stamina
  table and exposes a validated 1–365 day horizon; a dataset default is
  provided by `forecast_days`.
- Asset prefixing is incomplete across legacy `gtr2career` event and object
  assets. Existing dataset references continue to support the current names.
- Dataset-defined event/calendar visibility selection is not implemented.
- The requirements screen supports configured relative paths and installation
  checks, but the dataset file format is still permissive rather than formally
  versioned.

## Gameplay

- Automatic/random race results do not yet model class-specific finish
  probabilities, reliability/DNF chances, home-track modifiers, or preparation
  modifiers.
- Work obligations use the current dataset rules; warning/grace-period
  behavior and partial pay are not a generic engine guarantee.
- Loan approval, repayment schedules, credit eligibility, and non-terminal
  default consequences are not fully generalized.
- Sponsor negotiation probability, cooldowns, fit modifiers, and manager tiers
  remain primarily dataset/plugin behavior rather than a complete generic
  rule contract.
- Track-day wear/damage, social-event success/neutral outcomes, sickness
  seasonality, and published expected annual maintenance costs are not
  universally configured.

## Content

- The requested 12-championship calendar, test days, preludes, race events,
  fee/prize tiers, and machine-readable export are now present in
  `gtr2career`.
- Several high-tier vehicle descriptions and visual assets remain generic or
  placeholder content.
- Vehicle weight and maximum-power metadata are now optional generic object
  fields (`weight_kg` and `max_power_hp`) and are populated for the current
  vehicle catalog.
- The machine-readable `gtr2career/championships.csv` export covers the
  requested championship calendar.
