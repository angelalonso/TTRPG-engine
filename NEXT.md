# Next TO DO:
- I would like the playtest to be easier to configure. Consider the following use cases:
  - I want to control that the goal is to achieve a given championship or a number of championships of a given level.
  - I want to make sure the playtest does not repeat the same path twice (unless there are "luck elements that mess with the path", like getting sick at a ver bad moment)
  - I want to make sure every turn the playtest knows its possibilities (getting a job, applying for a sponsor, buying something...)
  - I want the playtest to build a list of actions it takes, that then the playtest can give a score for success or failure on its own

- Please create the championships and events/races described on HONDA_FIESTA_CHAMPS.txt (make sure you configure them correctly in terms of which ones require a ford fiesta, which ones a civic and which ones just any of those two.
- Please remove the existing formula ford championship and any related events.
- Currently the events include also jobs, and that is wrong, pelase troubleshoot and correct. As an example, I can see jobs on the Events/races list


# Stuff needed:
- Messages are OK, but warnings should be orange and bad news should be red
- Test weirdness:
  - Can I buy more of the same?
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- Sponsoring also gives a boost in Charisma
- If race same day as job, no money OR fired
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Dashboard picture can change depending on what it has (helmet, suit...)
- Different trailers needed for different cars
- Objects need more depth, more info, more columns with good defaults
- Race results need two modes: automatic or user-entered. Default is user-entered
  - User-entered has the option of a random result that the user triggers.

# Rules for AI
I want to continue with the development of this program and I want to remind you of the rules:
- Ask me if you need the current content of any file.
- Always provide me full content of ANY file you consider needs changes.
- Functionality is agnostic, actual names of the objects and actions are stored on the /dataset folder, which can be modified outside of the program. E.g.: We define objects in the code and how they behave, we define an object of type car called "renault" in the dataset.
