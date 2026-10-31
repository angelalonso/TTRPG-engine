1.- The game is based on days, which advance automatically, but the player can configure the speed or even pause it. 

2.- There are several goals for the player, and they are distributed in a progressive way.

3.- The goals are very rarely straightforward, and usually have requirements that the player has to deal with before even trying.

4.- As for the means towards those goals, there are different types of items that define the game itself. There are objects the Player can get, buy or sell, there are events that the player can take part on, there are quests which are combinations of events.

5.- There is a way to save and load a game.

# GTR2 Career
## 1.- Days-based game
There is a stable calendar on this dataset: the 1st of January is always Monday, regardless of the year.

## 2.- Progressive goals
- There are different types of objects, with different prices. At the beginning the player needs to secure income before investing on the objects it requires to race. There is a point when a job cannot finance the player's racing career anymore and it needs sponsors. Moreover, the sponsors have different levels, and a lower level sponsor will never want to pay the amount of money needed for a high level championship. There are also points of XP (experience) that the player can only win by taking part on events. As finding Sponsors becomes more and more difficult, the player can also opt for paying a manager (also there are different levels of managers) to increase the chances of getting a sponsor. On top of that, there are requirements for races and championships that the player has to gradually pay for, like a better trailer or a higher level racing licence.
- The rewards that the player gets from events are also increasingly higher depending on the level of the given event. 

## 3.- Requirements
- Track days require a car, but one can rent that car.
- Races require a car, which still can be rented, but they also require a racing licence, insurance and a helmet on all levels. From level 1 onwards they require a trailer, a tracksuit, gloves and race boots. Specifics on these can be seen on NEXT.md.
- Each Licence requires the previous one, e.g.: licence international C requires that the player already has the National A licence.
- There are two "special" requirements on events: Money and Stamina. The game simulates the investment of energy needed to try an action, e.g.: if you do two job interviews in one day, probably you will not have energy left to do a third interview.

## 4.- Items
- Objects can be of type "vehicle", "gloves", "racing boots", "helmet", "tracksuit", "trailer", "licence", "insurance", "companion" and "trophy". Each of them have their own characteristics.
- Events can be of type "track day", "test day", "meeting", "race" and "sponsor negotiation"
- Quests are Championships on this dataset, and they are normally a set of "Race" events plus maybe a "test day" and a "meeting" (meeting with fans that is). Once the player joins a Championship, all the related events are added to the Events calendar.
- Sponsor negotiation is an event that gets the player an economic reward. That reward can be direct (one-time payment, monthly salary) or indirect (the sponsor provides with a car for the championship, the sponsor pays for any repairs or provides the trailer...)

## 5.- Save and Load Game
- To avoid problems while we develop a dataset, the savegame file includes the current status and not the base data itself. For instance, it includes the payments the user will receive and why, but the original element that created those payments (e.g.: monthly payments by a sponsor) can be changed. This means loading a game retrieves also the data from the related dataset
