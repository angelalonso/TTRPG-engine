# Next TO DO:

- The races should show what requirements are not met (e.g: when the car does not have a pair of tires)
- On the results plugin, when data has been read from the results file, we need a button to reset like it did not read anything yet. Also once we have chosen one of the entries, the plugin saves the Driver Name (or it is sent to the main program, which saves it in cfg.yml as a list of names. Next time a results is read, it looks for that name and autoselects if it find it. Also once the data has been read, "Your finishing position" is greyed out and not used. Same goes for "other competitors, one per line..." text box.
- On that plugin, there is a non editable textbox saying "Ford Fiesta ST150 Championship - Club Series - Round 2: Pembrey" but above it there is a title saying the same, remove the textbox. Also change the title "Track ID (editable)" for "Track ID (double check this is the right file)
- Once joined, the Championship should show the current standings and which races have already happened.
- Insurances cannot be sold

# Stuff needed:
- Compile for windows
- Compile for android
- Code works, refactor to make it agnostic fully
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
- Functionality is agnostic, actual names of the objects and actions are stored on the /dataset folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.

# TO TEST
- Does quit game work?
- objects clickable?
- object buttons with photo?

