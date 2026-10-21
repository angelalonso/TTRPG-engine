# Next TO DO:
Content:
- Use 180 Motor Club reference for most of the events as organizer, give it a logo.
- Please add the possibility to have images also for events and championships, like objects do.
- Please make the HTML of Events follow the same pattern as cars and other objects. Make the border silver-grey instead of Red, and instead of having the maintenance costs, put the event details below the picture. Then on the right side just write some lines about the event
- Please make the HTML of Championships follow the same pattern as cars and other objects. Make the border golden-yellow instead of Red, and instead of having the maintenance costs, put the entry and rewards details below the picture. Then on the right side just put the championship rounds (and results as they produce)
- Please make the HTML of licences, insurance, companion and trailer follow the same pattern as cars and other objects. 
- Please make the HTML of Jobs and Ventures follow the same pattern as cars and other objects. Make the border green instead of Red, and instead of having the maintenance costs, put the details of each one. Then on the right side just put some explanation
Functionality:
- No dataset migration is defined from revision 34 to 35 <- I see this message when trying to migrate a saved game. Is there a way around it? otherwise we can just reset the counter of migrations and I will just delete that savegame.
Playtest:
- Whenever the playtest interacts with the game, the probabilities should be set to 1 on things like getting a job or a sponsor (not on sickness and other random stuff). The idea is to make the playtest stable enough for the system to identify what is the best choice.
- The playtest reinforncement training should have negative points for getting too close to running out of stamina (lets start with 10 points or less) or running out of money (lets start with less than 950 after 30 days since the game starts with 0 but a pending payment of 950 by the end of the month).
- I got this error when running playtest on a different machine, is this a true error? it does not look like it.
/home/aaf/Software/Dev/TTRPG-engine/.venv/lib/python3.12/site-packages/torch/cuda/__init__.py:484: UserWarning: Found GPU0 NVIDIA GeForce GTX 1050 which is of compute capability (CC) 6.1.   
The following list shows the CCs this version of PyTorch was built for and the hardware CCs it supports:
- 7.5 which supports hardware CC >=7.5,<8.0
- 8.0 which supports hardware CC >=8.0,<9.0 except {8.7}
- 8.6 which supports hardware CC >=8.6,<9.0 except {8.7}
- 9.0 which supports hardware CC >=9.0,<10.0
- 10.0 which supports hardware CC >=10.0,<11.0 except {10.1}
- 12.0 which supports hardware CC >=12.0,<13.0
Your installed torch==2.14.1+cu130 does not include kernels for this GPU. Reinstall the same version against a CUDA build that does, e.g.:
  For CUDA 12.6 use pip install torch==2.14.1 --index-url https://download.pytorch.org/whl/cu126
  _warn_unsupported_code(d, device_cc, code_ccs)
/home/aaf/Software/Dev/TTRPG-engine/.venv/lib/python3.12/site-packages/torch/cuda/__init__.py:602: UserWarning:
NVIDIA GeForce GTX 1050 with CUDA capability sm_61 is not compatible with the current PyTorch installation.
The current PyTorch install supports CUDA capabilities sm_75 sm_80 sm_86 sm_90 sm_100 sm_120.
If you want to use the NVIDIA GeForce GTX 1050 GPU with PyTorch, please check the instructions at https://pytorch.org/get-started/locally/



- make all objects ok on level 0
  - Licence National B for amateur races, none for track days
  - Does NOT Require a trailer
  - Sponsors of a higher level, they can use their own trailer
- move on to level 1
  - Still Licence National B
  - HH200 and Formula Ford
    - Formula Ford's removal of wings
  - Championship, test days, track days
  - Requires a trailer
  - Sponsors of a higher level, they can use their own trailer

- The dataset editor needs some changes:


# Stuff needed:
- make all objects ok on level 0
  - Licence National B for amateur races, none for track days
  - Does NOT Require a trailer
  - Sponsors of a higher level, they can use their own trailer
- move on to level 1
  - Still Licence National B
  - HH200 and Formula Ford
    - Formula Ford's removal of wings
  - Championship, test days, track days
  - Requires a trailer
  - Sponsors of a higher level, they can use their own trailer
- move on to level 2
  - Licence National A
    - Caterham 320
    - Porsche 996 UK CUP
    - Radical UK?
  - Licence International C
    - HH300
    - Radical Australia?
    - Radical US?
  - Championship, test days, track days
  - Requires a trailer
  - Sponsors of a higher level, they can use their own trailer
- move on to level 3
  - Licence National A
    - BTCC
  - Licence International C
    - Commonwealth GT4 Works - TVR + Ginetta + Mosler
    - Races in Canada, US, Australia, Asia
  - First take on "Factory Driver"
- move on to level 4
  - N-GT
  - Licence International B
  - European Series
  - Off-season abroad
  - No Factory Drivers on this level
- move on to level 5
  - GT - Licence International B, and whatever comes
  - World Series
- level 7 would be hypercar - Licence International A
- level 8 would be F1 - Super Licence
- Requirements on race gear are for any version of the object. For instance, on gtr2career, the helmet requirement means that having either a generic helmet or bell_rs7_pro or arai_gp6_carbon is enough
- Compile for android
- Code works, refactor to make it agnostic fully
  - create a different game with the same engine, spot the racing hiccups.
- Document possible connections between objects and other elements like costs.
- Manager helps with sponsors
  - Communicate through messenger (make it specific to our game, create communication and companion for other games)
- Sponsoring also gives a boost in Charisma
- Dashboard picture can change depending on what it has (helmet, suit...)
- Different trailers needed for different cars
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Race results need two modes: automatic or user-entered. Default is user-entered
  - User-entered has the option of a random result that the user triggers.
- Different Starting points (background stories)
  - make the stories NOT boring
- Select which tournaments to show on calendar

# Rules for AI
I want to continue with the development of this program and I want to remind you of the rules:
- Ask me if you need the current content of any file.
- Always provide me full content of ANY file you consider needs changes.
- Functionality is agnostic, actual names of the objects and actions are stored in the /gtr2career folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.

# TO TEST
- Does quit game work?
- objects clickable?
- object buttons with photo?

# Balance review: prices and costs

The current `gtr2career` economy is internally usable, but several values make racing progression and everyday work feel unrealistic:

- **Cars and entry progression:** The £10,500 Ford Fiesta ST150 and £14,000 Honda Civic are reasonable starter prices, but the £46,500 Hot Hatch Cup cars are a large jump without a matching income tier. Add intermediate used-car options and condition-based purchase prices.
- **Race income:** Club championship rounds pay £500 for a £150 entry fee and national rounds pay £750 for a £250 entry fee. Open races pay roughly £2,600–£6,500 for £450–£850 entry fees. Reduce the guaranteed profitability of open races or add travel, tyres, fuel, and damage costs so prize money is not almost pure profit.
- **Championship prizes:** The configured £5,000/£3,000/£2,000 final prizes are much larger than the per-round rewards. Either increase round rewards modestly or reduce final prizes so the season payout is not disproportionately concentrated at the end.
- **Track days and social events:** Track days cost £250–£450 and provide no direct payout. Add a small paddock-cred range and a meaningful chance of wear/damage; social events costing £25–£30 should provide a clearer reputation benefit to justify the spend.
- **Race gear:** Basic helmet (£250), tracksuit (£300), gloves (£150), and racing boots (£180) are affordable compared with cars, while premium equipment (£420–£1,800) has very large jumps. Add one mid-tier option per equipment group and make replacement/lifetime rules consistent across generic and branded gear.
- **Insurance:** Annual insurance at £1,000/£3,000/£6,000 is expensive relative to the starter cars but cheap relative to high-end cars. Scale premiums from vehicle value and coverage limits, and make the third-party policy clearly insufficient for expensive vehicles.
- **Maintenance and repairs:** Generic maintenance (£2,500), engine rebuilds, gearboxes, and factory rebuilds are plausible for race cars, but the £150–£550 oil services and cosmetic repairs should be separated from mandatory race preparation. Add a predictable pre-race service budget so a player can plan rather than only react to random damage.
- **Consumables:** Tyre sets (£500–£2,400) and insurance costs do not consistently scale with vehicle price/class. Tie tyres, brakes, and insurance to vehicle value or performance class, and ensure the Hot Hatch, ST150, and Civic tiers have comparable operating-cost ratios.
- **Jobs and living income:** The 9–5 job (£250/week), harder job (£300/week), and part-time job (£125/week) are too low to support even the £10,500 starter car; the temporary job's £11 payout is an obvious outlier against the £100–£167 payouts of the other small jobs. Normalize all job pay to an hourly/daily basis and add living costs so work is meaningful without making racing impossible.
- **Loan affordability:** The £17,500 personal loan with £1,000 repayments every 30 days needs an explicit interest/fee and total term. Add a credit/eligibility gate, a repayment schedule, and a non-lethal default consequence before allowing a missed payment to end the run.
- **Sponsors:** Sponsor cash, monthly payments, repairs, and equipment should be priced against the target championship tier. Local sponsors currently offer £1,400–£2,500 up front and £140–£250 monthly, which can dominate early income without obligations or performance targets.

# Balance review: probabilities and outcomes

- **Sponsor offers:** All six sponsor activities use a 5% base success probability, while owning a manager changes it to 99%. Replace this binary jump with a tiered formula, for example 10–20% without a manager and 35–70% with one, modified by paddock cred, championship tier, sponsor fit, and recent results.
- **Sponsor negotiation:** The negotiation actions range from 42% to 76% base success before charisma modifiers. Add diminishing returns and a failure cost/cooldown so repeatedly selecting the highest-probability action is not always optimal.
- **Jobs:** Job success rates range from 50% to 85% and are unrelated to stamina cost, pay, or player characteristics. Base them on job difficulty and charisma/experience, and make routine employment close to reliable while keeping specialist gigs risky.
- **Work obligations:** Missing work currently accumulates faults and can terminate a job. Add a warning/grace period, partial pay, and a probability of dismissal rather than making three missed workdays deterministic.
- **Crypto/venture:** The high-risk trade has only a 5% success chance for a £500 cost and £5,000 return. Add partial-loss outcomes and a probability band around the player's risk/charisma rather than a single near-lottery result.
- **Personal loan:** The loan currently has a 100% acceptance/success rate. Separate approval probability from payment success; approval should depend on budget, income, existing obligations, and possibly a manager/credit history.
- **Social events:** The only configured social failure result is 15%, with a fixed -5 charisma effect. Add success/neutral outcomes and make the result depend on charisma so social events are not a mostly fixed penalty.
- **Sickness:** The daily sickness probability of 0.001111111 is approximately 0.11% per day. Define a recovery/seasonality model and distinguish ordinary illness from race-week illness, where the cost is materially higher.
- **Race outcomes:** Race entry and championship events use deterministic success values of 1 because results are user-entered. Preserve that mode, but if automatic/random results are enabled, add class-based finish probabilities, reliability/DNF chances, and a small home-track or preparation modifier.
- **Damage and service costs:** Cost-rule probabilities should vary by vehicle age, mileage, damage state, and event intensity rather than applying mostly fixed repair categories. Publish the expected annual maintenance cost for each vehicle class.

# Balance review: race and championship prizes

Implemented in dataset revision 2:

- Competitive open races now pay a descending purse through 5th place. The
  existing winner purse is preserved; 2nd, 3rd, 4th, and 5th receive
  approximately 70%, 50%, 35%, and 25% of the winner's purse, rounded to
  practical cash amounts.
- Championship rounds now pay through 5th place. Club rounds use a smaller
  lower-order purse than national rounds, while preserving their existing
  winner, runner-up, and third-place values.
- Championship final rewards now pay through 10th place. The existing
  1st–3rd rewards remain unchanged, followed by 1,200/800/600/450/350/275/200
  for 4th–10th.
- Track days remain practice and preparation activities rather than prize
  events; they do not receive competitive race purses.
- The runtime now uses the configured `position_rewards` amount when a
  classified finishing position is submitted. A classified position without a
  configured prize receives no purse instead of incorrectly receiving the
  winner's flat `reward_pool`.

This is a gameplay abstraction of amateur club motorsport. Real amateur
events commonly combine modest cash or contingency awards with trophies,
credits, prizes, and recognition; many club championships pay little or no
cash outside the leading places. Paying lower positions here keeps finishing
outside the podium meaningful without implying that every club event is
commercially profitable. Future balancing should consider replacing some of
the lower cash awards with tyre/fuel credits, entry-fee rebates, paddock
cred, or equipment vouchers, especially in the club series.
