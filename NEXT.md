# Next TO DO:
- The config popup should have a button to load the selected dataset right below it, but also have a button to save chenges regarding the "Popup and Pause" categories. The events on the events tab should also have a way to only show what is on our Alarm list or show all


- Stamina system is defined as follows: 
Maximum of 100 points (as it is right now). 
Nightly recovery of +25 points. 
Every work day costs 30 points, but that is only days 1 to 5 of the week.
Every Job search costs 30 points as well. User must have 30 points or more to take part.
A Race day costs 20 points of Stamina (2 days of racing: 20+20)
The player needs to have enough stamina before an event starts or a work day begins. Otherwise it cannot participate or go to work.
Three days without going to work means the player gets fired. If the player is sick, that does not count as one of these three days.
- If Stamina goes down to 0, the player dies.
- Sickness does not mean Stamina is 0, but maybe just around 10, then around 50
- All championships gives objects called trophies. Those trophies have different levels (starting at level 1). Sponsoring for championships from level 2 onwards require at least one trophy from the previous level.
- There should be several different types of helmets, gloves, race boots...etc. All of them should be accepted on the requirements for things like a license. For instance, if a License requires having a helmet and gloves, then sponsored helmet and momo gloves would be accepted as well. Please rename the current helmet, gloves, etc to Generic Helmet, Generic Gloves...etc. For each one I want you to add real mid-price examples and expensive examples, like the Arai GP-6 Carbon helmet. MAke the prices realistic. Mid-price ads +1 to Charisma and Expensive adds +5. Each of these racing-gear Objects can only be bought once.
- Rename Charisma to Paddock Cred

- When a game is started, the user is asked for a Name, which will be shown on the Dasboard and title.
- The current popups for things like events should be configurable: Make it possible to mark events to be added to our alarms, and make it automatic that, when the player is enrolled on a championship, all related events are automatically added to our list of alarms. Then on the config the user should be able to choose which ones should trigger a pop up and "pause of time" among "Income", "Costs applied", "Event incoming" or "My Alarms". Regardless, all of them must get registered on the event log
- Money making should be classified by types as well: jobs, luck, sponsors...
- The Racing Market should only show the title and price of any objects we cannot buy. No images, no buy button. If the price is over our budget, the price should be in Red. If there are other requirements (License Advanced requires Basic License first) they should be put in red as well.
- The trophy objects should go in a third tab under "Garage", to the right of Garage and Driver's Room, and it should be called "Trophies". In code, this should be called "Achievements" but call it Trophies on the dataset csv. Also rename the sub-tab Garage to "Service Bay" please, to avoid naming confussion

# Stuff needed:
- Test weirdness:
  - Can I buy more of the same?
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- If race same day as job, no money OR fired
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Dashboard picture can change depending on what it has (helmet, suit...)
- Sponsoring also gives a boost in Charisma
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
