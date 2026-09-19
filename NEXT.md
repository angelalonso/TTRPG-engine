# Next TO DO:
- Daily jobs require the player to have enough stamina from day 1 to day 5. If that is not the case, the player is fired.
- Stamina should mean the user cannot do two full jobs simultaneously. 
- If Stamina goes down to 0, the player dies.
- Sickness does not mean Stamina is 0, but maybe just around 10, then around 50
- All championships gives objects called trophies. Those trophies have different levels (starting at level 1). Sponsoring for championships from level 2 onwards require at least one trophy from the previous level.
- When a game is started, the user is asked for a Name, which will be shown on the Dasboard and title.
- There should be several different types of helmets, gloves, race boots...etc. All of them should be accepted on the requirements for things like a license. For instance, if a License requires having a helmet and gloves, then sponsored helmet and momo gloves would be accepted as well. Please rename the current helmet, gloves, etc to Generic Helmet, Generic Gloves...etc. For each one I want you to add real mid-price examples and expensive examples, like the Arai GP-6 Carbon helmet. MAke the prices realistic. Mid-price ads +1 to Charisma and Expensive adds +5. Each of these racing-gear Objects can only be bought once.
- Rename Charisma to Paddock Cred
- Make it possible to add a calendar alarm to events, if enrolled on a championship, all related events are automatically added to alarms. Otherwise, choose on config which ones to get alarmed on: "Income", "Costs applied", "Event incoming". Regardless, all of them get registered on the event log

# Stuff needed:
- Test weirdness:
  - Can I buy more of the same?
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- If race same day as job, no money OR fired
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Dashboard picture can change depending on what it has (helmet, suit...)
- Sponsoring also gives a boost in Charisma
- Only show objects inside your budget first, add filter
- Different trailers needed for different cars
- Some objects cannot be sold, like a trailer "loaned" by the Sponsor
- Popups should be enabled and disabled at will, same goes for auto-pause.
- Objects need more depth, more info, more columns with good defaults
- Race results need two modes: automatic or user-entered. Default is user-entered
  - User-entered has the option of a random result that the user triggers.
- Messages are OK, but warnings should be orange and bad news should be red

# Rules for AI
I want to continue with the development of this program and I want to remind you of the rules:
- Ask me if you need the current content of any file.
- Always provide me full content of ANY file you consider needs changes.
- Functionality is agnostic, actual names of the objects and actions are stored on the /dataset folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.
