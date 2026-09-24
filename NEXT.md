# Next TO DO:
- When I enter a race and I have racing gear (as I should) it asks me if I want to enter with any of them, helmet, gloves...and the car I have. Please only offer that for the cars that the player owns, even when there are no requirements.
- On a non-championship race or track day, when I have to enter the result, I have the textbox to enter my result and right to the right I have "Damage from this event", which should go below.
- Also remember that after each race and track day, a new set of tires is needed.
- Moreover, cosmetic repairs never mean that a car cannot be used for a race/trackday. Instead, entering an event with cosmetic means a -2 on charisma (unless that means going negative, in that case, minimum is 0)
- loan does not seem to work fine. Every 30 days the user has to pay 1000, and after 22 iterations (30x22 days) the loan is paid. Loans require the user to have a job, and you cannot have more than 2 loans at the same time. Again, make this configurable on the CSV and take the variables from the code.
- The ford fiesta has all its costs with a message like: "ford_fiesta_st150_tires (missing cost definition): Ready" can you correct those missing cost definitions?

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
