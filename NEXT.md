# Next TO DO:
- Championships should have a button to see details instead of the full details shown upfront. They should just have the title and a "tag" if the player is enrolled on it or not
- The title of the game should be configurable and accept variables like "%player_name%'s Racing Career" (where player_name is a 
- The config popup should have a button to load the selected dataset right below it, but also have a button to save chenges regarding the "Popup and Pause" categories. The events on the events tab should also have a way to only show what is on our Alarm list or show all
- All championships gives objects called trophies. Those trophies have different levels (starting at level 1). Sponsoring for championships from level 2 onwards require at least one trophy from the previous level. Each of those trophies have the championship's name on it, the finishing position and a level to them. The current ones related to honda civic are level 1.
- For objects that cannot be bought (requirement or money missing) the format is good, but for those that can be bought I want the picture to the leftmost and then the price and button to buy should be to the right of the picture. Right now the picture is on top and the rest below it, and that looks wrong.
- Money making Should have sub-tabs (like garage has)



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
