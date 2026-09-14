# Next TO DO:
- Join the Championship before applying for Sponsor
- When I want to add a championship race result, and I choose that I was "not on points" it understands it as the first result after the positions with points. So if only down to the 3rd position gets points/charisma, it understands that I did 4th. The problem is: sometimes I want to add other drivers to the list of results that are not on the points. In that case, it should look for the lowest position that is not on the points and is not yet taken. So if I add the first 5, and I am not on points, it will assign me 6th. Or if I add 10 drivers but dont configure the 6th position, instead of giving me the 12th position it would give me the 6th (provided the 6th does not get rewards)
- dataset_gui does not allow me to configure item types, it still seems to have the racing example hardcoded. It is also missing the different configurations possible for costs and probably many more.
- Make the python script be a GUI where the changes are shown in a mockup of the main program. We start showing the Dashboard, then add an inventory tab, choose its name, then add a dealer tab, choose its name, add items, modify their costs...always showing a menubox when there is a limited set of options to choose from (e.g: item requires objects_ids, get a list of the existing ones to choose from). Add events, add quests...

- Social failure should not be on the csv it is right now, it should be a possible result from a social event, in the same way there are things to earn from the results of race events. Also having this there means other games need that as well. Please redo.
- make run should also use the ./dataset folder

# Stuff needed:
- Test weirdness:
  - Can I buy more of the same?
- Fight mode
  - Fight mode to get a job
  - Fight mode to get a Sponsor
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- If race same day as job, no money OR fired
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Dashboard picture can change depending on what it has (helmet, suit...)
- Sponsor gives you suit, carisma, a trail for the season...
- SPonsoring also gives a boost in Charisma
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
