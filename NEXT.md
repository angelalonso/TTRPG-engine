# Next TO DO:

plugins - all
- We should have a standard test plugin that can be used as a base for newer plugins
- running any plugin with -inputlist gives back a list of the details it needs.
- plugins by default reuse the look and feel from the main program. This means reusing the same web definitions (probably CSS).

companion
- the program needs a new tab, but only visible when the player has a companion.
- In our racing example the companion is a manager, in other games it can be anything.
- About how a manager would work, please reuse as much as sensible for the generic characteristics of a companion of some sort.
- Having a manager means in this case that we get offers from sponsors instead of us having to go to them asking for sponsorship. 
  - When the sponsors propose, the probability of success is around 99% (only will not work if we hit the other 1% with a randomized "dice throw")
- Another future type of companion that may not be so complex would be a mechanic that costs X per month but then repairs cost up to 3000 less and maintenance of the cars does not require our interaction (tires are bought after every race, cosmetic repairs are automatically done for no extra money...)

plugin - sponsors negotiation
- We need a set of sponsors divided into tiers: local, regional, national, continental, world
- At first I would like you to create just 3 local sponsors: a repairman, a restaurant and a pub.
- We need to find a way to accomodate for those sponsors as a standard item that fits the current CSV structures. Probably we should just add any consequences (sponsored vehicle, extra money after races...)
- Depending on the level of the player's manager (companion), the player will get sponsor proposals. The higher the level, the better the deals are and the more important the sponsors are.
- The player can also do "cold calls" and initiate negotiations with a sponsor. 
- The Sponsors sub-tab shows current sponsor proposals as well as ponsors that we can propose to.
- Sponsor proposals have an expiration date.
- Click on either a cold call to a sponsor or the offer from a sponsor opens the sponsor negotiation plugin's GUI, which is a different window but looks like the main program(HTML, CSS...).
- The possible proposals sponsor and player have to agree about are:
  - Sponsor for an event, a quest or a year
  - Sponsor includes initial money, monthly payments or nothing
  - Sponsor includes racing gear (sponsored tracksuit, sponsored boots...etc) and/ or a specific car.
  - Sponsor includes maintenance and repairs of the car or not.
  - Sponsor includes entry fee for races and championships (for a given event, a quest, the whole year...)
  - Sponsor includes bonuses for race and/or championship results (down to podium only)
  - Sponsor includes penalties for incidents and DNFs
- On top of that, Sponsor will always cancel the sponsorship after 5 DNFs in a year
- There is also a percentage (very low) that the Sponsor cancels the sponsorship out of the blue without compensation.
- As for the negotiations the defining factors are defined strictly:
  - Podiums, Races won, Championships won AND the level of their races add points set the base.
  - On the other side, the amount of money involved and, in the case of a single race or championship, the charisma of it, defines if the sponsor will negotiate or reject directly.
  - Please check Sponsor_System.txt for a definition of how the system works up to this point.
  - This means at this point money and player's results should match
  - On top of that, there is a 15% percentage up and down where sponsor and player can propose newer values.
    - Whether the sponsor accepts depends on luck, having a manager 
    - Having a Manager lowers the amount of requirements the sponsor has.
  - Negotiations means one proposes and the other corrects, until one accepts or negotiations fail completels. Please look up generic negotiation dynamics in the internet and follow through.
  - To avoid having a random proposal out of the blue, when the player does a "cold call" (player initiates proposal) the system defaults to a conservative (+-5% of the ideal match) proposal that the player can change before sending.


dataset-editor:

- Lets forget about the extra python program to edit dataset and add a mode to the regular program (First screen, additional button in red "Editor mode")
- This mode will show every editable element (meaning their value comes from a CSV) as a Button, and when one clicks on it a popup (as large as the window) shows, where we can edit that element.
- For instance, on events we can edit their whole entry and dependencies (references on other CSVs) by clicking on any column in the entry
- Navigation elements (e.g: tabs and subtabs) should still be usable though, but maybe have an edit button on their side.
- Clicking on the top bar allows us to also modify colors, calendar, icons...
- When editing items that are part of a table (e.g: actions, events, sponsors...) we need a way to edit the whole related table (CSV) and navigate to those dependencies (costs for instance).
- Editing stuff that has references to existing items somewhere else (e.g.: types of event, requirements...) the user should see a dropdown menu to select one or more possible existing items or send us to the part of our program where we can add a new item of that type.
- Races/events should show the table on the right side, which is bigger, and then show options for what is selected on the left side. It needs a way to select several entries and modify them in batch if possible
- Editor on tables allows to hide columns
- The program needs a "Save and Exit" button present at all times

events & co:
- Championships and Events allows to filter by car that can participate (allow multiple selection)
- money making > ventures shows also returns that are not just monetary (e.g.: +100 in charisma). This should also be the case for any of the "money making" subtabs actually, sponsors included


other
- modify what is listed on racing_reference_words.txt to avoid using those racing terms on the program code.
- We need an additional type of objects in the Race Market: Insurances. We want at least three types: one where nothing is covered (terceras personas in Spanish), one where repairs up to 3000 euros are covered (and anything over that must be paid by the player) and the final one where everything is covered. They cost accordingly and having at least one of them is mandatory to race (same as with racing gear). 
- Events have a charisma too. This charisma (or cred on our racing game) means that winning it, or getting to the podium, gives the player ... charisma.
- Events results also include the possibility to add if we did pole position


# Stuff needed:
- Compile for windows
- Compile for android
- Code works, refactor to make it agnostic fully
- Document possible connections between objects and other elements like costs.
- Manager helps with sponsors
  - Communicate through messenger (make it specific to our game, create communication and companion for other games)
- Sponsor gives money for the whole season, always shorter than what is necessary for costs
- Sponsoring also gives a boost in Charisma
- Dashboard picture can change depending on what it has (helmet, suit...)
- Different trailers needed for different cars
- Events are mandatory, automatically enrolled (by championship) or free to enter
- Race results need two modes: automatic or user-entered. Default is user-entered
  - User-entered has the option of a random result that the user triggers.
- Different Starting points (background stories
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

