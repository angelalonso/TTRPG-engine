# Next TO DO:
- When I buy a formula ford, I see "Pounds: 4" and a button to buy "1 pounds for £1800". I want the vehicle description in the garage to show only the costs that the object have defined. Feel free to modify dataset/* if needed. I want to be able to add up to 15 cost_* columns to dataset/objects.csv and the frontend should only show those that are defined. The python Editor should show amenu to choose among the entries defined in dataset/costs.csv.
- please explain what the point of dataset/cost_rule_conditions.csv is. What it is meant for and why it is not included on other CSV files 
- Event list needs to show days left to the event. 
  It also needs to be sorted by either name or days left. 
  It also needs a filter. 
  Also different colour if the even happens right now.
- Taking Part in an action closes the popup.
- Every action requires stamina. No stamina, cannot enter action.
- Stamina recovers by +1 every day when user took no actions. Being in a job means stamina usage everyday and some recovery on the 6th and 7th day of the week unless the user takes an action.
- Dates for races and trackdays are always 5th 6th and 7th day of the week. You can calculate day of the week as in day 1 is a monday and the rest is a progression. Find an efficient way to do this.
- Garage rent is linked to owning a car. No car, no rent.
- Objects can be sold as well. Selling value goes down 10% right after buying, then 10% every year with a minimum of 10% of original. All these variables must be on the csv and managed agnostically on the program itself. 
- Save and Load game - limited to Dataset used (save files on same foldeR?).
- make run should also use the ./dataset folder
- integrated sqlite 


# Stuff needed:
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
