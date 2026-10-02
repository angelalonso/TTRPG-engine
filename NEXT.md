# Next TO DO:
- We need a plugin on dataset/ to negotiate and get sponsors. It should get as many a detail from the player as possible and internally decide what to do with those details (in a future iteration)
- Please modify risiko_beim_schreiben to work with hte current data structure. It was created on a previous version and probably is missing some details. Better yet, modify dataset_editor.py to identify this and propose the user how to "migrate" to the newer version (retaining as much data as possible and letting the user decide on what is missing)
- modify what is listed on racing_reference_words.txt to avoid using those racing terms on the program code.
- Dataset editor should use better its GUI. 
- It is good to use the right side for a previoew when we are changing the colors or the dashboard. For any other changes (Events, objects...) we definitely need to use that side of the GUI-window to navigate through the possible variables and values.
- regarding those values, those that are linked to others (e.g.: reference to a cost type) and mandatory should show a drop-down with all currently existing items of that type AND a button that sends us to the part of our program where we can add a new item of that type.



# Stuff needed:
- Compile for windows
- Compile for android
- Code works, refactor to make it agnostic fully
- Document possible connections between objects and other elements like costs.
- Manager helps with sponsors
  - Communicate through messenger (make it specific to our game, create communication and companion for other games)
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- Sponsoring also gives a boost in Charisma
- Dashboard picture can change depending on what it has (helmet, suit...)
- Different trailers needed for different cars
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Race results need two modes: automatic or user-entered. Default is user-entered
  - User-entered has the option of a random result that the user triggers.
- Different Starting points (background stories
  - make the stories NOT boring
- Select which tournaments to show on calendar
- calendar includes 364 days plus "Racer day"
  - lets add months too

# Rules for AI
I want to continue with the development of this program and I want to remind you of the rules:
- Ask me if you need the current content of any file.
- Always provide me full content of ANY file you consider needs changes.
- Functionality is agnostic, actual names of the objects and actions are stored on the /dataset folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.

# TO TEST
- Does quit game work?
- objects clickable?
- object buttons with photo?

