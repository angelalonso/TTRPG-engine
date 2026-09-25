# Next TO DO:
- I also want to make sure the look and feel is consistent, so the HTMLs buttons for instance should look like the buttons for the tags on the application.
- When I go to an open race or trackday, I want an option to Rent a Car for the day. The available cars should show on a menu after I click on renting the car (one for each car you see in the CSVs). The rent costs 1/25 from the car's price and still the license and race gear requirements apply, but not the costs associated BEFORE racing. Damage will be paid for automatically.

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
