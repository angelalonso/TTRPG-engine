# Next TO DO:
- current championship standings should be shown when one clicks on it, maybe reuse the "not enrolled"/"joined" text and make it a button, put it a bit more to the left than currently, leaving some space on the right, and when one clicks on it, you get the current standings of the championship (if joined)
- When I have to enter a result on a race, my finishing position seems to be 1,2,3 or not on points, regardless of how the race is configured (maybe drivers down to the 6th position do get rewards) Please correct this.
- The ford fiesta has all its costs with a message like: "ford_fiesta_st150_tires (missing cost definition): Ready" can you correct those missing cost definitions?

- things like this 'message: "You accumulated three workday faults and were fired.".into(), ' should not be hardcoded, but those messages should be defined in CSVs
- moreover the logic behind workdays that are required and have a consequence if there is a fault or more...those shouldb e as generic as possible: when the player gets a job, it gets the obligation to "pay" X stamina, if it cannot pay, it gets a fault and on 3 faults it loses the job. The similar logic should fit a loan where the user gets 20000 moneys, then needs to pay 1000 every 30 days, 22 times, and if one month he doesnt pay, he dies. The details of how much, what the consequences are and for how long this goes on...those should be on CSVs
- I see some details on dataset/* are written in German. Please change to English only for those files under /dataset/.
- "not enough stamina for" -> should also go into CSV as in "Not enough resources of type X"

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
