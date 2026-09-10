# Next TO DO:
- On Racing Market, default sort is by price. Make this adjustable (create generic functionality if not yet in code) and set in the related CSV
- Also on Racing market, under any of the submenus, the items should be shown as wide as the screen allows, isntead of the current width that allows for several columns.
- Those items should show also a thumbnail if the related image file is present (otherwise nothing). The thumbnail must be resized to fit the button.
- Some items that are bought are only available after X days. For now make all cars available after 5 days. make this also adjustable in the related CSV and add the needed code if not yet present.
- Add events that are sickness. 25% chance daily. When it happens, Stamina goes down to 0 for 4 days, then back to 50 for another 2 days and get +50 (in case some stamina was used before it wouldnt recover directly to 100) after those 2 days.
- From the races you added, I got one that showed this error when I tried to join: "Events can only be scheduled on days 5, 6, or 7 of the week". That should not be an error that the program needs to handle, instead you should have scheduled all races and trackdays on days 5,6 or 7 of the week. Please correct and Scheduled race that does not fit. Also dont be afraid to schedule several races on the same day.
- Championships/Quests should have their own tab to the right of races. After a Championship race I should be able to manually enter the results for each race (a table where I can mark my position and write names for the other players, but only down to the ones that get points) and the championship should show the current standing by adding those results.
- Create a Python script with GUI that guides me through the tutorial (but also creating the entries in the related CSV files). At the end of the script I should have all files needed in a folder called dataset_tutorial.
- Make it possible to "quit" an action of type job
- Probability of finding a job is not 100%

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
