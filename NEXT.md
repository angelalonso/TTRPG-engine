# Next TO DO:
- Selling an object needs a confirmation popup to avoid accidental pushing of the button.
- Save needs a confirmation message that it has happened (maybe a message with the file name?) Load game needs confirmation from the user that it can overwrite the current game status.
- We need another set of costs that are random, and calculated daily. These would be called Cosmetic fix, Bolt-on repair and Factory rebuild, each cost a different amount, have a different probability to happen "out of the blue" and show a different message to the user when they happen.
- After each race, the user should choose what kind of damages resulted from that race (maybe on the same popup where the user enters its result).
- Apart from the costs, Bolt-on repair means one day where the car cannot be used, and Factory Rebuild means 2 Weeks. These variables are modifiable and, as always, defined on a csv under dataset/:
- We also need a new kind of object, which is a license. Some races require a licence. To get a licence, one needs to pay a fee and sometimes already have the previous level of license. The "highest license" the player has should be shown on the Dashboard.
- The very first thing needed even before the most basic licence is an object of the same type: helm, tracksuit, gloves and shoes. Again, shown at the Dashboard.

- the settings, save and load buttons should also use icons that can be overwritten like the speed buttons
- We need a way to put several events together like a championship and have the final result of the championship be calculated.
- Some races need to require a specific car (or a list of them)

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
