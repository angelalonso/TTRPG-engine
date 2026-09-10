# Next TO DO:
- The buttons inside racing market, for cars, race gears and licenses, should look better when they are selected. They are OK looking until I click on them and they become "boxy"
- Add some races that do not belong in the championship. At least one every month. Rename the ones that belong to the championship to make it clear. Make the championship only for formula fords for now.
- Track days should also count as a race in terms of services needed (tires, oil change...)
- Damages of the three levels should be services as well (only to be paid for when there are damages)
- When a race belongs to a championship, and I have not joined, it should say which championship instead of just "quest"
- Add more finer details on how to create a game about wizards. Try to explain the different variables and types of actions, quests, objects, configs, costs, cost_rules, cost_rule_conditions, events and players with related examples. Put it on a file called TUTORIAL.md. 

- make run should also use the ./dataset folder

# Stuff needed:
- Test weirdness:
  - Can I buy more of the same?
- Fight mode
  - Fight mode to get a job
  - Fight mode to get a Sponsor
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- If race same day as job, no money OR fired
- Game builder (not dataset editor)
- Championship
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Day X of year Y (start  by 0)
- Show objects types in groups at the shop
  - Different helmets/suits/gloves... have different durability and charisma
- Dashboard picture can change depending on what it has (helmet, suit...)
- Sponsor gives you suit, carisma, a trail for the season...
- You can quit a job
- Probability of finding a job is not 100%
- Only show objects inside your budget first, add filter
- Different trailers needed for different cars
- Some objects cannot be sold, like a trailer "loaned" by the Sponsor
- SPonsoring also gives a boost in Charisma
- Popups should be enabled and disabled at will, same goes for auto-pause.
- Objects need more depth, more info, more columns with good defaults
- Race results need two modes: automatic or user-entered. Default is user-entered
  - User-entered has the option of a random result that the user triggers.
- Engine is agnostic from what kind of game it is
  - When menus are agnostic, a placeholder for their icons/emojis is also needed
- Messages are OK, but warnings should be orange and bad news should be red
- every object needs a placeholder for images. 

# Rules for AI
I want to continue with the development of this program and I want to remind you of the rules:
- Ask me if you need the current content of any file.
- Always provide me full content of ANY file you consider needs changes.
- Functionality is agnostic, actual names of the objects and actions are stored on the /dataset folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.
