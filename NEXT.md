# Next TO DO:
- Change playtest.json to mean "goal is to get 1 trpho of level 1"
- The racing market does not show the related HTML when I click on an object.
- Please integrate all features from dataset_editor.py into dataset_gui.py. In case of duplicated features, dataset_gui's have priority.
- Please make dataset_gui ask for which dataset to edit before starting. Give it the possibility to create a new empty dataset (And its folder)
- Please make the program save its configurations to a file called cfg.yml, and add an option to start in fullscreen mode.
- is event_outcomes.csv used? can it be deleted? how about dataset/event_results.csv ?

- Please advice on the best way to extend the program: I am thinking about a system of plugins written on python that do things that are specific for a given game: Fight system, Results injection, automatically reading results from another game... Is this doable under the current stack? can this program make calls to a python script?
- If it is doable and efficient, I would like the "race results input" to be the first plugin. Please "externalize" the results-reading system to a python script that does the same. Remember that the championship results depend on these results as well, and integrate the championship standings into that plugin too.

- Please create a performance tester to check where the program could be opimitzed.

- Please create a new CSV with different texts for the same "variables". This is needed for things like popups saying "you failed to get the job". The program should call the csv and randomly use one of the strings attached to a variable in that CSV, which would look like 'job_9-5_fail;"You didn't get the job";"They said they will call you";"Someone better looking than you landed the job". That CSV should have an undetermined number of columns. If you think its more efficient, you can also create .txt files, like job_9-5_fail.txt with one phrase on each line.

# Stuff needed:
- Compile for windows
- Compile for android
- Code works, refactor to make it agnostic fully
- Document possible connections between objects and other elements like costs.
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- Sponsoring also gives a boost in Charisma
- Dashboard picture can change depending on what it has (helmet, suit...)
- Different trailers needed for different cars
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Race results need two modes: automatic or user-entered. Default is user-entered
  - User-entered has the option of a random result that the user triggers.

# Rules for AI
I want to continue with the development of this program and I want to remind you of the rules:
- Ask me if you need the current content of any file.
- Always provide me full content of ANY file you consider needs changes.
- Functionality is agnostic, actual names of the objects and actions are stored on the /dataset folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.
