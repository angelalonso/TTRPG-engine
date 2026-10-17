# Moonroot: A Forest Alchemist's Trade

*Game design spec for a text-based, first-person trading game about foraging, wizardry and potion-brewing, structured like Dope Wars. All names, places, creatures and items are original inventions.*

## 1. Pitch

You are a young hedge-wizard in the Thornmarch, a region of old forests, fens, mountains and harbor towns. You owe **5,500 crowns** to the Lanternkeepers, a guild of lenders who do not take kindly to late payment. You have **30 days** to turn a satchel, a battered kettle and a head full of half-remembered lore into a fortune.

Each day you travel to a new place, forage for ingredients (mushrooms, leaves, stones, bones, feathers, venom glands), buy and sell goods at the local market, and brew potions that sell for far more than their parts. Prices swing wildly. Random events help or hurt. Interest on your debt compounds daily.

The twist on the classic formula: **knowledge is a second currency.** Ingredients start as mysteries. The more you learn about their hidden properties, the more valuable potions you can brew, and the more you can exploit price swings that others do not understand.

**Tone:** gritty, wry, fairy-tale-with-teeth. Text only. Second person (“You arrive in Murkwater Fen. The air tastes of copper and rot.”).

## 2. Core Loop (one day)

1. Show the **status bar**: Day X/30, Location, Gold, Debt, Bank, HP, Pack used/total, Lore, Daylight remaining.
2. Show the **market**: ingredients and potions on offer today with prices.
3. The player has **2 Daylight points** per day. Each of these actions costs 1 Daylight: **Forage**, **Brew**, **Study**. Market actions (Buy, Sell, Drop) and Service visits (Lender, Bank, Healer, Outfitter, Sage) are free.
4. The player may **Travel** at any time. Travel ends the day: advance the date, spoil perishables, roll a random event, apply debt interest, generate the destination's market, restore Daylight to 2.
5. On Day 30 the game ends. Final score is in Section 12.

## 3. Starting State

| Stat | Start |
| --- | --- |
| Gold | 2,000 crowns |
| Debt | 5,500 crowns (to the Lanternkeepers) |
| Bank | 0 |
| HP | 100 / 100 |
| Pack space | 100 slots |
| Lore | 1 (scale 0–10) |
| Daylight per day | 2 |
| Staff | Willow Switch (Attack 1) |
| Robe | Patched Traveling Robe (Defense 0) |
| Cauldron | Tinker's Kettle (batch yield 3) |
| Known recipes | Mending Draught, Clear-Head Tonic |
| Known ingredients | Dewcap Mushroom, Frostfern Leaf, Moth Wing Dust, Wolf Fang, Duskberry |
| Start location | Thistledown |
| Days | 30 |

## 4. Locations (8)

The player may travel to any location except the current one. Each has a **danger rating** (1–5 skulls), a **biome** (which drives foraging), and **price personality**: cheap items ×0.7, expensive items ×1.3, everything else ×1.0.

| Location | Biome | Danger | Cheap goods | Expensive goods | Services |
| --- | --- | --- | --- | --- | --- |
| Thistledown | Meadow | 1 | Dewcap Mushroom, Frostfern Leaf | Drake Scale Shard | Herbalist (healer), Kettle Hire |
| Murkwater Fen | Swamp | 3 | Marsh Adder Gland, Duskberry | Mending Draught | Fen Witch (sells rumors) |
| Highcairn | Mountain | 3 | Cold Quartz, Stormglass Shard | Emberheart Tincture | Outfitter (staffs, robes) |
| Saltmere Harbor | Coast | 3 | Moth Wing Dust | Raven Skull | **Guild Vault (Bank)**, Outfitter |
| Silverbirch Academy | Orchard and library | 1 | Glimmer Moss | Stormstep Philtre | **Sage (recipes and lore)**, Healer, **Bank**, Cauldron and Satchel shop |
| Emberfall Crater | Volcanic | 4 | Cinder Geode, Ashbloom Fungus | Frostwalker's Brew | Hot Springs (healer) |
| Gloamwood | Deep forest | 4 | Elderwood Amber, Raven Skull | Clear-Head Tonic | Hireling Camp (porters, apprentice) |
| Lantern Hollow | Smugglers' town | 5 | Drake Scale Shard | Glimmer Moss | **The Lender**, Black Market |

## 5. Goods

There are two kinds of tradeable goods: **Ingredients** (foraged or bought) and **Potions** (brewed or bought). Each good has a price range; the day's price in each location is a random integer in the range, then multiplied by the city's personality modifier and any event modifier. Buy and sell price are identical (as in the original). Each unit takes 1 pack slot.

**Availability:** on arrival, each ingredient is available with about 70% probability, each potion with about 60%. A player may only see and sell goods that are listed in the current market.

### 5.1 Essences

Every ingredient carries hidden **Essence** values. There are six essences:

| Essence | Meaning |
| --- | --- |
| Vital | Life, growth, healing |
| Ember | Heat, fire, passion |
| Frost | Cold, stillness, preservation |
| Umbra | Shadow, stealth, decay |
| Storm | Lightning, wind, speed |
| Mind | Memory, dreams, clarity |

### 5.2 Ingredients

“Spoils” means the good is perishable (see Section 7). Volatility (Vol) raises the chance of a brewing mishap (Section 8). Price ranges are in crowns.

| Ingredient | Type | Price range | Essences | Vol | Spoils | Flavor |
| --- | --- | --- | --- | --- | --- | --- |
| Dewcap Mushroom | Fungus | 8 – 30 | Vital 2, Mind 1 | 0 | Yes | Pale, damp, smells of rain. |
| Frostfern Leaf | Leaf | 15 – 60 | Frost 2, Vital 1 | 0 | Yes | Silver fronds that stay cold in the sun. |
| Moth Wing Dust | Animal | 25 – 90 | Mind 1, Storm 1 | 0 | No | Powdery scales from lamp-moths. |
| Wolf Fang | Carcass | 100 – 350 | Vital 2, Umbra 1 | 1 | No | Yellowed, still sharp. |
| Duskberry | Berry | 40 – 150 | Umbra 2, Mind 1 | 1 | Yes | Black fruit. Stains everything. |
| Ashbloom Fungus | Fungus | 20 – 70 | Ember 2, Umbra 1 | 1 | Yes | Grows only on burnt ground. |
| Glimmer Moss | Moss | 30 – 110 | Mind 2, Vital 1 | 0 | Yes | Faintly luminous. |
| Thunderthistle | Herb | 60 – 220 | Storm 2, Ember 1 | 2 | Yes | Crackles when picked. |
| Cold Quartz | Stone | 90 – 300 | Frost 2, Storm 1 | 0 | No | Frosted rock from high crevices. |
| Cinder Geode | Stone | 150 – 500 | Ember 3 | 2 | No | Warm to the touch, hums faintly. |
| Raven Skull | Carcass | 300 – 900 | Umbra 2, Mind 2 | 1 | No | Said to remember its last meal. |
| Marsh Adder Gland | Carcass | 800 – 2,500 | Umbra 3, Storm 1 | 3 | Yes | Dangerous to carry. |
| Stormglass Shard | Stone | 1,500 – 4,000 | Storm 3, Mind 1 | 2 | No | Glass fused by lightning strikes. |
| Elderwood Amber | Sap | 3,000 – 7,000 | Vital 3, Mind 2 | 0 | No | Ancient tree resin with things trapped in it. |
| Drake Scale Shard | Carcass | 6,000 – 14,000 | Ember 3, Vital 2 | 2 | No | Shed by a thornback drake. Worth a lot, attracts trouble. |

**Player knowledge states** for each ingredient: **Unknown** (shown by a vague label such as “a pale damp mushroom”, essences hidden) and **Known** (real name and essences shown). Section 9 explains how players learn.

### 5.3 Potions

| Potion | Recipe requirement (total essences of the ingredients used) | Price range | Combat use |
| --- | --- | --- | --- |
| Mending Draught | Vital ≥ 3 | 40 – 120 | Heals 40 HP |
| Clear-Head Tonic | Mind ≥ 3 | 80 – 260 | Cures a status effect, +1 flee chance |
| Frostwalker's Brew | Frost ≥ 3 | 150 – 450 | Thrown: 30 damage, enemy loses a turn (40%) |
| Emberheart Tincture | Ember ≥ 3 | 150 – 450 | Thrown: 40 damage |
| Shadowveil Elixir | Umbra ≥ 4 | 500 – 1,500 | Guaranteed flee |
| Stormstep Philtre | Storm ≥ 4 | 700 – 2,000 | Next travel skips the hostile-event roll |
| Seer's Draught | Mind ≥ 4 and Umbra ≥ 2 | 1,500 – 4,500 | Preview tomorrow's event type at the next destination |
| Stag's Vigor | Vital ≥ 5 and Ember ≥ 2 | 2,500 – 7,000 | Heals full HP, +10 max HP permanently |
| Hailstorm Draught | Frost ≥ 3 and Storm ≥ 3 | 3,000 – 9,000 | Thrown: 80 damage |
| Elixir of Long Years | Vital ≥ 5 and Mind ≥ 4 | 12,000 – 30,000 | Rarely used. Sells for big money. |
| Murky Sludge | Failed brew | 3 – 10 | Worthless |

Potions are the high-margin goods. Dragon Scales equivalent: Elixir of Long Years and Drake Scale Shards are the all-in gambles.

## 6. Price Mechanics

- Daily price = random integer in the good's range, × city personality modifier (0.7 / 1.0 / 1.3).
- **Price events** (Section 10.1) apply a further multiplier (×0.25 crash, ×3 to ×5 spike) for the current visit only.
- Strategy loop 1: buy cheap ingredients in one place, sell dear elsewhere.
- Strategy loop 2: forage free ingredients, brew potions, sell potions where they are expensive.
- Strategy loop 3: study to unlock higher-tier recipes and the rare ingredients they need.
- No haggling in v1. Optional v2 feature: Lore-based haggle roll.

## 7. The Pack and Spoilage

- Each unit takes 1 slot. Base capacity 100.
- **Upgrades:** Satchel of Many Pockets (+50 slots, 8,000 crowns), Hammock Wagon Share (+100 slots, 20,000 crowns), Porter hireling (+20 slots).
- The player may **Drop** goods anywhere. Goods lost in events are removed.
- **Spoilage:** when the player travels, each stack of **perishable** ingredients (marked Spoils in 5.2) loses 10% of its units, rounded up, minimum 1 if the stack is non-empty (e.g. 12 → 10, 3 → 2). Having a **Preserving Salt** pouch (Outfitter, 400 crowns, lasts 5 days) halves this. Frost-essence potions never spoil. Non-perishable goods are unaffected.

## 8. Brewing

Brewing is the heart of the game. It costs 1 Daylight and requires a cauldron (the Tinker's Kettle is always carried).

**Cauldron tiers**

| Cauldron | Cost | Batch yield (potions per brew) | Max ingredient slots |
| --- | --- | --- | --- |
| Tinker's Kettle | start | 3 | 3 |
| Copper Cauldron | 6,000 | 5 | 4 |
| Silver Cauldron | 15,000 | 8 | 4 |

**Procedure**

1. The player chooses 2 to N ingredients from the pack (up to 2 of the same ingredient). Exactly one unit of each chosen ingredient is consumed per brew.
2. The engine sums each essence across the ingredients.
3. The engine checks every potion in order from highest to lowest price; the first whose requirement is satisfied is the result. If none match, the result is **Murky Sludge**.
4. **Mishap check:** mishapChance = 5% × (total Vol of ingredients) − 3% × Lore, clamped to 0–40%. On a mishap, the brew is ruined (ingredients lost), the player takes 10–30 damage (“The kettle bursts!”), and no potion is made.
5. On success, the player receives *batch yield* units of the potion. If the potion recipe was not already known, it is **discovered** (+1 Lore) and a celebratory text is shown.
6. Any Unknown ingredient used is **revealed** (becomes Known) whether the brew succeeds, fails or is a mishap.
7. Players may only attempt combinations that they have not previously tried; the game remembers every tried combination in a **Grimoire** screen.

A player who has not discovered a recipe may still brew it by accident. A player who has learned a recipe sees it in the Grimoire with a “Known” tag and can use the **Recipe Suggest** button, which lists known ingredients in the pack that meet the requirement.

## 9. Knowledge and Lore

Lore (0–10) is the wizard's skill.

| Lore effect | Rule |
| --- | --- |
| Foraging | Yield bonus +5% per Lore point |
| Brewing | Mishap chance reduced by 3% per Lore point |
| Appraisal | At Lore ≥ 4, show true price range hints (“usually 800–2,500”) on known goods |
| Identify | At Lore ≥ 6, new ingredients found by foraging are auto-identified |

**Ways to gain knowledge**

- **Study (1 Daylight):** pick an Unknown ingredient from the pack, spend 1 unit, and make a Lore check: success chance = 30% + 10% × Lore. On success the ingredient is revealed (+1 Lore with 50% chance). On failure the unit is wasted.
- **Sage (Silverbirch Academy):** buys and sells Recipe Pages (a known recipe) and Field Notes (an ingredient revealed). Prices 300 – 6,000 crowns.
- **Fen Witch (Murkwater Fen):** sells rumors for 100 – 500 crowns that reveal one essence of an Unknown ingredient.
- **Black Market (Lantern Hollow):** sells rare recipes at high prices.
- **Discovery by brewing** (Section 8).
- **Found items:** random events can award Field Notes, a recipe page, or a Lore point.

## 10. Foraging

Foraging costs 1 Daylight. The player walks the local terrain and the engine rolls on the biome's **forage table**.

- Roll 2–5 times (more with higher Lore), each roll picks an ingredient by weight.
- A forage roll has a 15% chance per skull of danger to cause **a hazard instead**: thorns (−5 HP), bee swarm (−10 HP), sinkhole (lose 1 random unit), a rare sighting (flavor text, +1 Lore).
- Yield per ingredient is 1–4 units (×1.05 per Lore point).

| Biome | Forage table (weights) |
| --- | --- |
| Meadow | Dewcap 40, Frostfern 30, Moth Wing Dust 20, Duskberry 10 |
| Swamp | Duskberry 35, Dewcap 20, Marsh Adder Gland 5, Raven Skull 5, Glimmer Moss 20, Wolf Fang 15 |
| Mountain | Cold Quartz 30, Frostfern 25, Stormglass Shard 5, Thunderthistle 20, Wolf Fang 20 |
| Coast | Moth Wing Dust 30, Cold Quartz 15, Stormglass Shard 5, Dewcap 25, Raven Skull 10, Duskberry 15 |
| Orchard and library | Glimmer Moss 35, Dewcap 25, Duskberry 15, Moth Wing Dust 20, Elderwood Amber 5 |
| Volcanic | Cinder Geode 30, Ashbloom 40, Thunderthistle 10, Drake Scale Shard 3, Wolf Fang 17 |
| Deep forest | Elderwood Amber 8, Raven Skull 15, Wolf Fang 20, Glimmer Moss 20, Duskberry 20, Dewcap 17 |
| Smugglers' town | (no foraging; the player may **Scavenge** the alleys for Moth Wing Dust or Dewcap only) |

## 11. Random Events

On each travel, there is a chance that something happens: **eventChance × danger / 3**, where eventChance = 0.60. Equipment such as the Cloak of Briars lowers it.

### 11.1 Price events (market shocks)

| Event text | Effect |
| --- | --- |
| “A drake burns the hillside. Drake Scale Shards litter the roads.” | Drake Scale Shard ×0.25 here |
| “The Archmage bans Stormglass. Collectors panic.” | Stormglass Shard ×4 |
| “A fever sweeps the market square. Mending Draught is in demand.” | Mending Draught ×3 |
| “A merchant's wagon overturns, spilling Moth Wing Dust everywhere.” | Moth Wing Dust ×0.3 |
| “Wildfire in the Ashbloom fields.” | Ashbloom Fungus ×5 |
| “A rival alchemist floods the market with Elixirs.” | Elixir of Long Years ×0.4 here |

### 11.2 Found items

| Event | Effect |
| --- | --- |
| “You find a dead traveler's satchel.” | +50 – 800 gold, or 3–15 units of a random ingredient |
| “A friendly hedgewitch gives you a pouch.” | +5–20 units of a random cheap ingredient |
| “You find an old page tucked in a hollow log.” | A random recipe page (reveals one recipe) |
| “A scholar's lost notebook.” | +1 Lore |

### 11.3 Hostile encounters (trigger Combat, Section 13)

| Encounter | Strength | Threat |
| --- | --- | --- |
| Hedge Brigands | Weak | Steal some goods |
| Highway Bandits | Medium | Steal gold and goods |
| Toll Warden Inspection | Medium | Fine or confiscate potions |
| Timber Wolves | Medium | Damage, no loot |
| Bramble Beast | Strong | Heavy damage |
| Marsh Wraith | Strong | Drains HP, no loot |
| Thornback Drake (rare, ≤3%) | Boss | Lose about 50% of everything unless you flee or pay |

### 11.4 Pressure events

| Event | Effect |
| --- | --- |
| “The Lanternkeepers' collectors find you.” | If debt > 0 and Day > 10: forced combat or pay 20% of debt |
| “Leyline surge!” | Random effect: two stacks in your pack swap quantities, heal 30 HP, lose 30 HP, teleport to a random location |
| “A pickpocket's curse clings to your purse.” | Lose 5% of gold |
| “Rot in your satchel.” | Extra spoilage: perishables lose a further 20% |

## 12. Equipment, Hirelings and Services

| Item | Source | Effect |
| --- | --- | --- |
| Oak Staff / Moonwood Staff | Outfitter | Attack 3 / 6 |
| Reinforced Robe / Warded Robe | Outfitter | Defense 3 / 6 |
| Cloak of Briars | Outfitter or Sage (6,000) | −50% hostile event chance |
| Preserving Salt | Outfitter (400) | Halves spoilage for 5 days |
| Herbalist's Salve | Healer (200) | +50 HP |
| Hedge Passport | Found | Avoids the next Toll Warden inspection |
| Seven-League Boots | Event | Next trip skips the danger roll |
| Copper / Silver Cauldron | Sage or Outfitter | See Section 8 |
| Porter hireling | Gloamwood camp (1,500) | +20 pack slots |
| Apprentice hireling | Gloamwood camp (3,000) | +1 Daylight per day, takes 10% of profits at game end |

## 13. Combat (simple)

1. Show the enemy: name, HP, attack.
2. Player picks **Fight**, **Cast** (use a combat potion from the pack), **Flee** or **Pay** (if applicable).
3. **Fight:** player damage = 5 + Staff Attack + d6; enemy damage = enemy attack + d6 − Robe Defense (min 1).
4. **Cast:** consumes one potion; applies its combat use (Section 5.3); the enemy then acts.
5. **Flee:** 50% success (+20% with Seven-League Boots, +10% with Clear-Head Tonic). On failure, the enemy gets a free attack.
6. **Pay:** costs 10–30% of current gold; ends the encounter safely.
7. **Victory:** loot (gold, or goods from the enemy's loot table, including carcass ingredients: Wolf Fang, Raven Skull, Marsh Adder Gland).
8. **Defeat (HP 0):** if the player has Mending Draught, auto-use one. Otherwise: “You wake in a roadside shrine. They took everything.” Lose 90% of gold and goods, HP set to 20, continue. A **Hardcore** option makes defeat permanent.

## 14. Money: The Lender and the Bank

**The Lender (Lantern Hollow only).** Starting debt 5,500 crowns. Player can pay any amount or borrow more (max 20,000 total). **Interest: +10% per day** on remaining debt, compounding. If debt exceeds 50,000, the Lanternkeepers send an enforcer: forced Strong combat.

**The Bank (Guild Vault in Saltmere Harbor, Silverbirch Academy).** Deposit and withdraw crowns. 0% interest in v1 (optionally 5% on an easier difficulty). Banked gold is safe from thieves, curses and enforcers.

## 15. Winning, Losing, Endings

Final score (end of Day 30) = Gold + Bank + value of goods at current market prices − Debt.

| Score | Title |
| --- | --- |
| < 0 | **Debtor's Cell.** You scrub cauldrons for the Lanternkeepers forever. |
| 0 – 10,000 | **Hedge Peddler** |
| 10,000 – 50,000 | **Guild Brewer** |
| 50,000 – 200,000 | **Mistress/Master of the Marsh Road** |
| 200,000+ | **Archalchemist of the Thornmarch** (secret ending: you buy a tower) |

Also show a **Lore Rank** on the end screen: Lore 0–2 Dabbler, 3–5 Adept, 6–8 Sage, 9–10 Grand Herbalist. Keep a High Score list (top 10, saved locally).

## 16. Screens (UI text spec)

**Main screen**

```
Day 4/30     Location: Murkwater Fen     Daylight: 2/2
Gold: 3,210  Debt: 8,055  Bank: 0  HP: 87/100  Pack: 22/100  Lore: 3

INGREDIENTS
 1. Dewcap Mushroom       18
 2. Duskberry             62
 3. a pale grey fungus    90   (unknown)
 4. Marsh Adder Gland   1,150

POTIONS
 5. Mending Draught      210
 6. Seer's Draught     3,300

YOUR PACK: Dewcap x12, Wolf Fang x4, Mending Draught x6

[B]uy [S]ell [D]rop [F]orage [R]Brew [Y]Study [G]rimoire [V]isit [T]ravel [Q]uit
```

**Brew screen:** list pack ingredients with numbers, player picks up to N, a preview shows known essence totals and mishap chance, \[C\]onfirm. **Grimoire screen:** known recipes, known ingredients and their essences, previously tried combinations. **Travel screen:** 7 destinations with danger skulls. **Event screen:** one paragraph of flavor text, a result line, press any key. **End screen:** score, title, Lore rank, high-score entry.

## 17. Data Tables for the Engine

Suggested JSON-like structure (any engine can map it):

```
essences: [vital, ember, frost, umbra, storm, mind]
ingredients: [ {id, name, type:fungus|leaf|stone|carcass|sap|animal|berry|moss|herb, minPrice, maxPrice, essences:{vital:2, mind:1}, volatility, perishable:bool, flavor, unknownLabel, startsKnown:bool} ]
potions: [ {id, name, minPrice, maxPrice, requirement:[{essence, min}], combatEffect, flavor} ]
cities: [ {id, name, biome, danger(1-5), priceMods:{goodId:multiplier}, services:[bank, lender, healer, outfitter, sage, camp, blackmarket, herbalist], forageTable:{ingredientId:weight}} ]
events: [ {id, type: price|found|combat|pressure, text, weight, conditions, effect} ]
enemies: [ {id, name, hp, attack, lootTable, fleeChance, canPay:bool} ]
items: [ {id, name, cost, effect, slot: staff|robe|cloak|cauldron|hireling|consumable} ]
player: {gold, debt, bank, hp, maxHp, packCapacity, inventory:{}, equipment:{}, lore, daylight, day, knownIngredients:[], knownRecipes:[], triedCombos:[]}
config: {totalDays:30, startDebt:5500, interestRate:0.10, eventChance:0.60, daylightPerDay:2, spoilRate:0.10, goodAvailability:{ingredient:0.7, potion:0.6}}
```

**Randomness rules:** seed per run for reproducibility. Event probability = `eventChance × cityDanger / 3`. Forage and brew mishap rolls use the same seeded generator.

**Brewing algorithm (pseudocode)**

```
sum = {}
for ing in chosen: for e, v in ing.essences: sum[e] += v
result = first potion (sorted by maxPrice desc) where all requirements satisfied by sum, else murkySludge
vol = sum(ing.volatility)
mishap = clamp(0.05*vol - 0.03*player.lore, 0, 0.40)
if rand() < mishap: consume ingredients, damage player, return
consume ingredients; add cauldron.yield units of result; reveal all chosen ingredients
if result not in knownRecipes: knownRecipes.add(result); player.lore += 1
```

## 18. Balance Notes

- With 10% daily interest, 5,500 crowns grows to about 96,000 over 30 days if never paid. Players should feel pressure to pay by about Day 15.
- A skilled player should end around 100k – 300k crowns. Playtest and tune event frequency first, then price ranges, then recipe thresholds.
- Early game: Dewcap, Frostfern, Moth Wing Dust and Mending Draught are the safe ramp. Drake Scale Shards and Elixir of Long Years are the all-in gamble.
- Check that every potion's expected sale price exceeds (sum of its minimal recipe ingredients' average cost ÷ cauldron yield) by at least 1.4× when sold in a favorable city, so brewing is always worth considering but never a dominant strategy.
- Knowledge should feel like an investment: Lore pays back through lower mishap rate, better yield and access to expensive recipes.

## 19. Optional Expansions (v2)

- **Classes:** Herbalist (better foraging and spoilage), Alchemist (better brewing and yield), Warlock (better combat, more Umbra recipes).
- **Haggling mini-game** and **reputation** with each location.
- **Quests:** one-time optional events (cure a lord's daughter, deliver a rare brew) for large rewards.
- **Seasonal modifiers:** autumn boosts fungi, winter boosts Frost essences.
- **Hardcore mode:** permadeath, 20 days, 15% interest.
- **Difficulty modes** adjusting starting debt and event danger.

## 20. Instructions for the Builder AI

> Build a text-based, turn-driven trading game following this spec. Keep the code data-driven so ingredients, potions, cities, events, enemies and items live in editable tables. Implement the core loop (Section 2), price generation (Section 6), spoilage (Section 7), brewing with essence matching and mishaps (Section 8), knowledge and Lore (Section 9), foraging by biome (Section 10), the event system (Section 11), simple combat including thrown potions (Section 13), Lender and Bank (Section 14), scoring and high scores (Section 15). Provide a keyboard-driven UI matching Section 16. Include a seeded random number generator and a config block for tuning. Treat all names as original; do not substitute any names from existing tabletop or video-game properties.

