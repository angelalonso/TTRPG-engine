# Next TO DO:

- Dataset editor should be able to clean up (confirmation from the user required) references that are missing on the CSV (e.g.: requirement of an event is an object that does not exist on the objects csv)

---
- This program is divided in two general parts:
  - The core program
  - The dataset and plugins


The dataset defines what kind of game the core program becomes. It is a bunch of CSVs that define what the player has, what objects it can acquire or use, and what events happen.
- Player characteristics can affect acquiring objects (in an example, the amount of budget affects if the player can acquire the object).
- There is also a Player "bag" that defines which objects the player has. This also can affect other things (e.g.: you can only buy a certain object if you already have another object or a given amount of them)
- Objects can have requirements to acquire (e.g.: be only acquired through a successful event). Some objects can be sold, some can be loaned/rented... 
- Events can be a lot of things, from getting a job that pais regularly a sum to a fight where you can have positive and negative consequences.
- Quests are groups of Events, like a Championship is a group of Races that puts together the points after each race and gives the player a trophy that cannot be sold, but can make it easier to get a sponsor.

So, with that, I want you to investigate the code of this repository as follows:
- folder dataset includes dataset and plugins for acar racing career game. You can use it to get more use examples about player, events, objects, quests...
- folders dataset_tutorial, dataset_w_more and risiko_beim_schreiben must be ignored
- The rest are the code for the program itself

, after investigating I want you to put together a list of tasks that reach the following goals:
- the core program is agnostic from the type of game the dataset and plugins define. It should work for a car racing career but also for a pony stable or a cooking game. Identify terms, variable names... that refer to things that are car-racing related (like championship or garage) and turn them to a generic term (like using quest instead of championship
- the requirements and consequences of objects and events should be flexible. I want to be able to, for instance, define a new event where the amount of objects of one kind makes it easier for the player to succeed. To know the possibilities, we need a list of what can be done right now.
- dataset_editor should look closer to what the core program looks like and guide the user through. When I set an event, I want to see what possibilities for requirements I have and choose one, or tailor a new requirement and check htat it works. Each step should have explanation about what every part does.

With that I want you to put together a specific list of tasks in a file called next_development.txt.
That list will be passed on to cheaper AI models to take the tasks one by one.
---

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

# TO TEST
- Does quit game work?
- objects clickable?
- object buttons with photo?

