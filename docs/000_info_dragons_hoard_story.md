# Dragon's Hoard: A Dungeon Trader

*Game design spec for a text-based, first-person trading game in the Dungeons & Dragons setting, modeled on the structure of Dope Wars / Drug Wars.*

## 1. Pitch

You are a smuggler-adventurer in the Sword Coast with a debt to the Zhentarim-style "Black Network" and **30 days** to turn it into a fortune. Each day you travel to a city, see that city's prices for contraband and rare goods, then buy low and sell high. Between cities you face random events: bandits, goblin ambushes, customs inspectors, dragon sightings, wild magic surges. Pay off your debt, survive, and end with as much gold as possible.

**Tone:** gritty, funny, tavern-tale. Text only. All second person ("You arrive in Skullport..."). The player is the first-person protagonist.

*IP note: place names are Forgotten Realms. If you need a legally generic version, swap them for invented names; the mechanics do not change.*

## 2. Core Loop (one "day")

1. Show **status bar**: Day X/30, Location, Gold, Debt, Bank, HP, Pack space used/total.
2. Show **market**: the goods for sale in this city today with current prices (not every good is available every day).
3. Player may **Buy**, **Sell**, **Drop**, visit **Special Location** (Lender, Bank, Blacksmith, Healer), or **Travel**.
4. On Travel: advance the day, roll for a **random event**, apply debt interest, generate new prices at the destination.
5. On Day 30: forced end. Final score = Gold + Bank - Debt (see Section 11).

## 3. Starting State

| Stat | Start |
| --- | --- |
| Gold | 2,000 gp |
| Debt | 5,500 gp (to the Lender) |
| Bank | 0 gp |
| HP | 100 / 100 |
| Pack space | 100 slots (Bag of Holding upgrades later) |
| Weapon | Rusty Shortsword (Attack 1) |
| Armor | Leather Jerkin (Defense 0) |
| Hirelings | 0 |
| Start city | Phandalin |
| Days | 30 |

## 4. Locations (8 cities)

The player can travel to any city except the current one. Each city has a **price personality** (which goods tend to be cheap or expensive there) and a **danger rating**, which scales the chance of a hostile event.

| City | Danger | Cheap goods | Expensive goods | Special |
| --- | --- | --- | --- | --- |
| Phandalin | Low | Moonpetal Elixir, Trinkets | Dragon Scales | Starting town, Healer |
| Neverwinter | Medium | Goblin Firepowder | Wyvern Venom | Blacksmith |
| Waterdeep | Medium | Wild Magic Dust | Trinkets | **Bank** (Temple Vault) |
| Baldur's Gate | Medium | Elven Wine | Firepowder | Bank |
| Silverymoon | Low | Spell Scrolls | Wild Magic Dust | Healer, Wizard (Bag of Holding) |
| Mirabar | Medium | Dwarven Steel Ingots | Moonpetal Elixir | Blacksmith |
| Luskan | High | Wyvern Venom | Elven Wine | Hireling tavern |
| Skullport | Very High | Dragon Scales | Spell Scrolls | **The Lender**, black market |

## 5. Goods (the "drugs")

All goods are fantasy contraband or high-value rarities. Each has a price **range**; the day's price in each city is a random value inside the range, then modified by city personality and events.

| Good | Price range (gp) | Notes |
| --- | --- | --- |
| Dragon Scales | 15,000 – 29,000 | The big money. Heavy risk on events. |
| Wyvern Venom | 5,000 – 14,000 | High value, high swing. |
| Wild Magic Dust | 1,000 – 4,500 | Surges can trigger events. |
| Spell Scrolls (stolen) | 600 – 2,200 | Mid-tier. |
| Moonpetal Elixir | 300 – 900 | Reliable mid-low. |
| Elven Wine | 100 – 400 | Bulk-friendly. |
| Goblin Firepowder | 70 – 250 | Cheap, volatile. |
| Stolen Trinkets | 11 – 60 | Junk; good for beginners. |

**Availability:** on each arrival, each good is available with \~75% probability. Unavailable goods are not shown (player can still sell goods they own only if the city lists the good, as in the original game).

## 6. Price Mechanics

- Daily price = random integer in the good's range, then multiplied by the city's personality modifier (cheap ×0.7, normal ×1.0, expensive ×1.3).
- **Price events** (see Section 8) can multiply a price by ×0.25 (crash) or ×3–×5 (spike).
- Buy and sell price are the same (as in the original). No haggling in v1; optional v2 feature: a Charisma-based haggle roll.
- Strategy loop: find a cheap city, buy, travel to an expensive one, sell.

## 7. Inventory ("The Pack")

- Each unit of any good takes 1 slot. Base capacity 100.
- **Upgrades:** Wizard in Silverymoon sells a *Bag of Holding* (+50 slots, 8,000 gp) and a *Portable Hole* (+100 slots, 20,000 gp). Hirelings (pack mule) add +20 slots each.
- Player may **Drop** goods anywhere. Goods lost in events are removed from inventory.

## 8. Random Events

On each travel, roll the event table. Probability per trip is about 60% that *something* happens (higher in dangerous cities). Event types:

### 8.1 Price events (market shocks)

| Event text | Effect |
| --- | --- |
| "A dragon burns the Dwarven mines! Dragon Scales flood the market." | Dragon Scales price ×0.25 in this city |
| "The Royal Wizard bans Wild Magic Dust. Traders panic." | Wild Magic Dust ×4 |
| "Plague in the Elven quarter. Moonpetal Elixir is in demand." | Moonpetal ×3 |
| "A caravan of Trinkets arrives from the east." | Trinkets ×0.3 |
| "Orc raid on the Firepowder mills." | Firepowder ×5 |

### 8.2 Found items

| Event | Effect |
| --- | --- |
| "You find a dead adventurer's pack." | +gold (50–800) or +random goods (3–15 units) |
| "A friendly gnome gives you a pouch." | +1 random good (5–20 units) |
| "You find a magical ring on the road." | Random buff (see Section 9) |

### 8.3 Hostile encounters (trigger **Combat**, Section 10)

| Encounter | Strength | Threat |
| --- | --- | --- |
| Goblin ambush | Weak | Steal some goods |
| Bandit gang | Medium | Steal gold + goods |
| City Watch inspection | Medium | Fine or confiscate goods |
| Orc warband | Strong | Heavy damage |
| Owlbear | Strong | Heavy damage, no loot |
| Young dragon (rare, ≤3%) | Boss | Lose \~50% of everything unless you flee or pay |

### 8.4 Pressure events

| Event | Effect |
| --- | --- |
| "The Lender's thugs find you." | If debt > 0 and Day > 10: forced combat or pay 20% of debt |
| "Wild magic surge!" | Random effect: swap two of your goods quantities, heal 30 HP, lose 30 HP, teleport to a random city |
| "Cursed coin in your purse." | Lose 5% of gold |

## 9. Buffs and Items

| Item | Source | Effect |
| --- | --- | --- |
| Ring of Free Passage | Found | Avoid next City Watch inspection |
| Potion of Healing | Healer (200 gp) | +50 HP |
| Boots of Speed | Event | Next trip skips danger roll |
| Cloak of Elvenkind | Blacksmith/Wizard (6,000 gp) | -50% chance of hostile events |
| Longsword / Magic Sword | Blacksmith | Attack 3 / 6 |
| Chain mail / Plate | Blacksmith | Defense 3 / 6 |
| Pack mule hireling | Luskan tavern (1,500 gp) | +20 pack slots |
| Mercenary hireling | Luskan tavern (3,000 gp) | +4 combat power, takes 10% of profits at game end |

## 10. Combat (simple)

Combat is a short text sequence, like the shootouts in Dope Wars.

1. Show enemy: name, HP, attack.
2. Player picks **Fight**, **Flee**, or **Pay** (bribe, if applicable).
3. **Fight:** player damage = 5 + Weapon Attack + d6; enemy damage = enemy attack + d6 − Defense (min 1). Repeat per round until someone is at 0 HP.
4. **Flee:** success chance 50% (+20% with Boots of Speed). On failure, enemy gets a free attack.
5. **Pay:** costs 10–30% of current gold; ends encounter safely.
6. **Victory:** loot (gold or goods). **Defeat (HP 0):** game over unless the player has a *Potion of Healing* (auto-use) or the Healer scene triggers; otherwise "You wake up in a temple. They took everything." → lose 90% of gold and goods, HP set to 20, continue (this keeps the game forgiving; set a hardcore option for permadeath).

## 11. Money: Lender and Bank

### The Lender (Loan Shark equivalent)

- Located in Skullport only.
- Starting debt: 5,500 gp. Player can pay any amount, or borrow more (max 20,000 total).
- **Interest: +10% per day** on remaining debt (compounds). This is the pressure clock.
- If debt > 50,000 at any time, the Lender sends an assassin: forced combat with a Strong enemy.

### The Bank (Temple Vault)

- Located in Waterdeep and Baldur's Gate.
- Deposit and withdraw gold. Earns 0% interest in v1 (optionally 5% in a "merchant" difficulty).
- Gold in the Bank is safe from thieves and the Cursed Coin event.

## 12. Winning, Losing, Endings

**Final score** (end of Day 30) = Gold + Bank + value of goods at current city prices − Debt.

| Score | Title |
| --- | --- |
| < 0 | **Debtor's Prison.** You are chained in a dungeon. |
| 0 – 10,000 | **Humble Peddler** |
| 10,000 – 50,000 | **Guild Merchant** |
| 50,000 – 200,000 | **Lord of the Trade Road** |
| 200,000+ | **Dragon-Hoard Baron** (secret ending: you buy a castle) |

Also keep a **High Score list** (top 10, saved locally), as in the original.

## 13. Screens (UI text spec)

**Main screen:**

```
Day 4/30      Location: Waterdeep
Gold: 3,210   Debt: 8,055   Bank: 0   HP: 87/100   Pack: 22/100

MARKET
 1. Dragon Scales        --
 2. Wyvern Venom       9,400
 3. Wild Magic Dust    1,150
 4. Spell Scrolls        --
 5. Moonpetal Elixir     540
 ...

YOUR PACK: Wild Magic Dust x12, Trinkets x10

[B]uy  [S]ell  [D]rop  [T]ravel  [V]isit Bank  [Q]uit
```

**Travel screen:** list 7 destinations with danger rating (skulls).

**Event screen:** one paragraph of flavor text, a result line, press any key to continue.

**End screen:** score, title, high-score entry.

## 14. Data Tables for the Engine

Suggested JSON-like structure (any engine can map it):

```
goods: [ {id, name, minPrice, maxPrice, weight:1, availability:0.75} ]
cities: [ {id, name, danger(1-5), priceMods:{goodId:multiplier}, services:[bank, lender, healer, blacksmith, wizard, tavern]} ]
events: [ {id, type: price|found|combat|pressure, text, weight, conditions, effect} ]
enemies: [ {id, name, hp, attack, lootTable, fleeChance} ]
items:   [ {id, name, cost, effect, slot: weapon|armor|trinket|hireling} ]
player:  {gold, debt, bank, hp, maxHp, packCapacity, inventory:{}, equipment:{}, day}
config:  {totalDays:30, startDebt:5500, interestRate:0.10, eventChance:0.60}
```

**Randomness rules:** seed per run for reproducibility; event probability = `eventChance × cityDanger/3`.

## 15. Balance Notes

- With 10% daily interest, 5,500 gp grows to \~96,000 gp over 30 days if never paid. Players should feel pressure to pay by day \~15.
- A skilled player should end around 100k–300k gp. Playtest and tune event frequency first, then price ranges.
- Trinkets and Firepowder are the safe early game. Dragon Scales are the "all-in" gamble.

## 16. Optional Expansions (v2)

- **Classes:** Rogue (better flee and haggle), Fighter (better combat), Wizard (reduced event chance, wild magic control).
- **Haggling mini-game** and **reputation** with each city.
- **Quests:** one-time optional events (rescue a noble, deliver a package) giving big gold.
- **Hardcore mode:** permadeath, 20 days, 15% interest.
- **Difficulty modes** adjusting starting debt and event danger.

## 17. Instructions for the Builder AI

> Build a text-based turn-driven trading game following this spec. Keep the code data-driven so goods, cities, events, enemies and items live in editable tables. Implement the core loop (Section 2), price generation (Section 6), event system (Section 8), simple combat (Section 10), Lender and Bank (Section 11), scoring and high scores (Section 12). Provide a keyboard-driven UI matching Section 13. Include a seeded random number generator and a config block for tuning.
