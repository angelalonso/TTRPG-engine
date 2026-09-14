# Next TO DO:
- The image shown on dashboard is not displayed somehow. It should look on player.csv for an entry on the picture_file variable (should be the path to an image on <dataset_name>/img/) fnd if not found it should default to <main_folder>/img/player.jpeg

- The Sponsor-related actions need the following workflow:
User clicks on it
Popup comes up with a description of the sponsor. At the bottom there are two buttons, one to "Try luck with Sponsor", one to close the popup and go back.
If we click on "Try luck with Sponsor" we get another pop up for the so called "Fight mode". If we win, we get the sponsor, if not, we dont get the sponsor, and get a penalty (for now, -5 stamina, -1 charisma). The penalties are configurable through CSV like the rest of the games.
Now here is how the fight mode works:
- The player gets a set of "weapons" and the opponent too. For the player, some of these weapons are objects and they will only be listed if the player has them in inventory. Maybe call them tools instead of weapons. Also both get a number of points of resistance. Each weapon has a success rate, a positive result and a negative result.
- The fight can have several modes, but for now we only have an attack-defense mode: On each turn, the Player chooses to use a tool from its list, then the computer calculates the result of it, and to that result the Opponent tries to defend itself. If the result "before defence" is the maximum possible, the opponent cannot defend and the player wins. Otherwise, the points to substract from the opponent's resistance are calculated and the fight goes into the next turn for the player to chooose a tool to use. If the result "before defence" is 10% or less from the maximum possible for that tool, the player's resistance gets the full hit. First one to lose all its resistance loses.
- On the example of getting a sponsor, the tools the user have are: current user's Trophies (please create these as objects, make them sellable but dont make them "buyable" from the dealer), Charisma, business proposal, and lower cost. Please think about a race driver in their starting years to calculate the odds of him getting a sponsor.

- Getting the sponsor/Applying for it shows no popup. Dashboard shows Race gear: Not ready - buy all required gear. No eligible cars.
- Sponsored Gloves, shoes, track suit show "Loaned sponsor car; returned on day 366." but these objects are not loaned. also they should last longer than the regular objects (sponsor tracksuit should last 50% longer than a regular tracksuit) . Also rename helm by helmet.
- Social failure should not be on the csv it is right now, it should be a possible result from a social event, in the same way there are things to earn from the results of race events. Also having this there means other games need that as well. Please redo.
^
- make run should also use the ./dataset folder

# Stuff needed:
- Test weirdness:
  - Can I buy more of the same?
- Fight mode
  - Fight mode to get a job
  - Fight mode to get a Sponsor
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- If race same day as job, no money OR fired
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Dashboard picture can change depending on what it has (helmet, suit...)
- Sponsor gives you suit, carisma, a trail for the season...
- SPonsoring also gives a boost in Charisma
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
