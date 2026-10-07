# Next TO DO:

- When I join a championship, the 3..2..1 plugin is called but this plugin should only be shown at the beginning of a new game.
- on championships, when the status is "pending" or "waiting for requirements" (I dont recall the exact words) it should show a different background color (dont reuse the dark mustard color, choose something else that means a warning, maybe dark grey that is lighter than the current background?)
- When I enter the race result plugin, I get: Race-results plugin failed: 2026-10-07 09:37:52,243 race_results INFO processing result event=race_snetterton_open_january interactive=False autodetect=True. I tried "retry plugin" and choose result file, but I got the same error. I cannot enter the results by hand either so I guess it has nothing to do with the results file but the plugin itself. Overriding in game works.


- The race_results plugin should use the same GUI system as sponsor_negotiation. I like the features that the .py_old file had, make sure no features are lost, including the option to manually select a results file
- Every try to get one of the jobs should cost a realistic amount of stamina, so that the user dos not just try 20 times a day until it works.
- On races, when I set the alarm on, I want the line to have a background color like races from a joined championship do (use a diffrent color though). The button should also show a tick icon instead of "Alarm on"
- The table with the records for the player should also include the amount of races so far, and not just the amount of pole positions, podiums and wins.


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
- Functionality is agnostic, actual names of the objects and actions are stored in the /gtr2career folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.

# TO TEST
- Does quit game work?
- objects clickable?
- object buttons with photo?
