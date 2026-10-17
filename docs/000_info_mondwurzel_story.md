# Mondwurzel: Der Handel des Waldalchemisten

*Spieldesign-Spezifikation für ein textbasiertes Handelsspiel in der Ich-Perspektive über Sammeln, Zauberkunst und Trankbrauen, aufgebaut wie Dope Wars. Alle Namen, Orte, Kreaturen und Gegenstände sind frei erfunden. Technische Bezeichner (Schlüssel in den Datenstrukturen, Abschnitt 17) bleiben bewusst englisch, damit die Engine sie direkt übernehmen kann; alle Texte, die der Spieler sieht, sind deutsch.*

## 1. Idee

Du bist ein junger Heckenzauberer in der Dornmark, einer Region aus uralten Wäldern, Mooren, Bergen und Hafenstädten. Du schuldest den **Laternenwächtern**, einer Gilde von Geldverleihern, die säumige Zahler nicht freundlich behandelt, **5.500 Kronen**. Du hast **30 Tage**, um aus einer Umhängetasche, einem verbeulten Kessel und einem Kopf voll halb vergessener Kunde ein Vermögen zu machen.

Jeden Tag reist du an einen neuen Ort, sammelst Zutaten (Pilze, Blätter, Steine, Knochen, Federn, Giftdrüsen), kaufst und verkaufst auf dem örtlichen Markt und braust Tränke, die für ein Vielfaches ihrer Bestandteile den Besitzer wechseln. Die Preise schwanken wild. Zufallsereignisse helfen oder schaden. Die Zinsen auf deine Schulden wachsen täglich.

Der Kniff gegenüber der klassischen Formel: **Wissen ist eine zweite Währung.** Zutaten sind anfangs Rätsel. Je mehr du über ihre verborgenen Eigenschaften lernst, desto wertvollere Tränke kannst du brauen und desto besser nutzt du Preisschwankungen, die andere nicht verstehen.

**Ton:** derb, trocken, Märchen mit Zähnen. Nur Text. Zweite Person („Du erreichst das Trübwasser-Moor. Die Luft schmeckt nach Kupfer und Fäulnis.“).

## 2. Kernschleife (ein Tag)

1. Zeige die **Statusleiste**: Tag X/30, Ort, Gold, Schulden, Bank, LP, Gepäck belegt/gesamt, Wissen, verbleibendes Tageslicht.
2. Zeige den **Markt**: Zutaten und Tränke, die heute angeboten werden, mit Preisen.
3. Der Spieler hat **2 Tageslichtpunkte** pro Tag. Diese Aktionen kosten je 1 Tageslicht: **Sammeln**, **Brauen**, **Erforschen**. Marktaktionen (Kaufen, Verkaufen, Ablegen) und Dienste (Verleiher, Bank, Heiler, Ausrüster, Weiser) sind kostenlos.
4. Der Spieler kann jederzeit **Reisen**. Die Reise beendet den Tag: Datum erhöhen, verderbliche Waren verderben lassen, Zufallsereignis würfeln, Schuldzinsen anwenden, Markt am Zielort erzeugen, Tageslicht auf 2 zurücksetzen.
5. An Tag 30 endet das Spiel. Die Endwertung steht in Abschnitt 15.

## 3. Startzustand

| Wert | Start |
| --- | --- |
| Gold | 2.000 Kronen |
| Schulden | 5.500 Kronen (bei den Laternenwächtern) |
| Bank | 0 |
| LP (Lebenspunkte) | 100 / 100 |
| Gepäckplätze | 100 |
| Wissen | 1 (Skala 0–10) |
| Tageslicht pro Tag | 2 |
| Stab | Weidenrute (Angriff 1) |
| Robe | Geflickte Reiserobe (Verteidigung 0) |
| Kessel | Kesselflickers Kessel (Ausbeute 3) |
| Bekannte Rezepte | Heiltrunk, Klarsicht-Tonikum |
| Bekannte Zutaten | Tauhut-Pilz, Frostfarnblatt, Mottenflügelstaub, Wolfszahn, Dämmerbeere |
| Startort | Disteltal |
| Tage | 30 |

## 4. Orte (8)

Der Spieler kann zu jedem Ort außer dem aktuellen reisen. Jeder Ort hat eine **Gefahrenstufe** (1–5 Schädel), ein **Biom** (bestimmt das Sammeln) und eine **Preispersönlichkeit**: günstige Waren ×0,7, teure Waren ×1,3, alles andere ×1,0.

| Ort | Biom | Gefahr | Günstige Waren | Teure Waren | Dienste |
| --- | --- | --- | --- | --- | --- |
| Disteltal | Wiese | 1 | Tauhut-Pilz, Frostfarnblatt | Drachenschuppensplitter | Kräuterfrau (Heilerin), Kesselverleih |
| Trübwasser-Moor | Sumpf | 3 | Sumpfotter-Drüse, Dämmerbeere | Heiltrunk | Moorhexe (verkauft Gerüchte) |
| Hochstein | Gebirge | 3 | Kaltquarz, Sturmglasscherbe | Glutherz-Tinktur | Ausrüster (Stäbe, Roben) |
| Salzmoor-Hafen | Küste | 3 | Mottenflügelstaub | Rabenschädel | **Gildentresor (Bank)**, Ausrüster |
| Silberbirken-Akademie | Obstgarten und Bibliothek | 1 | Glimmermoos | Sturmschritt-Philtre | **Weiser (Rezepte und Kunde)**, Heiler, **Bank**, Kessel- und Taschenladen |
| Glutfall-Krater | Vulkan | 4 | Aschengeode, Aschenblüten-Pilz | Frostwandler-Gebräu | Heiße Quellen (Heilung) |
| Dämmerwald | Tiefer Wald | 4 | Ältestenholz-Bernstein, Rabenschädel | Klarsicht-Tonikum | Söldnerlager (Träger, Lehrling) |
| Laternenmulde | Schmugglerstadt | 5 | Drachenschuppensplitter | Glimmermoos | **Der Verleiher**, Schwarzmarkt |

## 5. Waren

Es gibt zwei Arten handelbarer Waren: **Zutaten** (gesammelt oder gekauft) und **Tränke** (gebraut oder gekauft). Jede Ware hat eine Preisspanne; der Tagespreis an einem Ort ist eine zufällige ganze Zahl in dieser Spanne, multipliziert mit dem Faktor der Preispersönlichkeit und eventuellen Ereignisfaktoren. Kauf- und Verkaufspreis sind identisch (wie im Original). Jede Einheit belegt 1 Gepäckplatz.

**Verfügbarkeit:** Bei der Ankunft ist jede Zutat mit etwa 70 % Wahrscheinlichkeit verfügbar, jeder Trank mit etwa 60 %. Der Spieler sieht und verkauft nur Waren, die im aktuellen Markt gelistet sind.

### 5.1 Essenzen

Jede Zutat trägt verborgene **Essenz**werte. Es gibt sechs Essenzen:

| Essenz (Schlüssel) | Bedeutung |
| --- | --- |
| Vital (vital) | Leben, Wachstum, Heilung |
| Glut (ember) | Hitze, Feuer, Leidenschaft |
| Frost (frost) | Kälte, Stille, Haltbarkeit |
| Schatten (umbra) | Dunkelheit, Heimlichkeit, Verfall |
| Sturm (storm) | Blitz, Wind, Tempo |
| Geist (mind) | Gedächtnis, Träume, Klarheit |

### 5.2 Zutaten

„Verdirbt“ heißt: Die Ware ist verderblich (siehe Abschnitt 7). Flüchtigkeit (Flü) erhöht die Chance auf einen Braumissgeschick (Abschnitt 8). Preisspannen in Kronen.

| Zutat | Art | Preisspanne | Essenzen | Flü | Verdirbt | Beschreibung |
| --- | --- | --- | --- | --- | --- | --- |
| Tauhut-Pilz | Pilz | 8 – 30 | Vital 2, Geist 1 | 0 | Ja | Blass, feucht, riecht nach Regen. |
| Frostfarnblatt | Blatt | 15 – 60 | Frost 2, Vital 1 | 0 | Ja | Silberne Wedel, die in der Sonne kalt bleiben. |
| Mottenflügelstaub | Tier | 25 – 90 | Geist 1, Sturm 1 | 0 | Nein | Pudrige Schuppen von Lampenmotten. |
| Wolfszahn | Kadaver | 100 – 350 | Vital 2, Schatten 1 | 1 | Nein | Vergilbt, immer noch scharf. |
| Dämmerbeere | Beere | 40 – 150 | Schatten 2, Geist 1 | 1 | Ja | Schwarze Frucht. Färbt alles. |
| Aschenblüten-Pilz | Pilz | 20 – 70 | Glut 2, Schatten 1 | 1 | Ja | Wächst nur auf verbranntem Boden. |
| Glimmermoos | Moos | 30 – 110 | Geist 2, Vital 1 | 0 | Ja | Schwach leuchtend. |
| Donnerdistel | Kraut | 60 – 220 | Sturm 2, Glut 1 | 2 | Ja | Knistert beim Pflücken. |
| Kaltquarz | Stein | 90 – 300 | Frost 2, Sturm 1 | 0 | Nein | Bereifter Fels aus hohen Spalten. |
| Aschengeode | Stein | 150 – 500 | Glut 3 | 2 | Nein | Warm anzufassen, summt leise. |
| Rabenschädel | Kadaver | 300 – 900 | Schatten 2, Geist 2 | 1 | Nein | Heißt es, er erinnere sich an seine letzte Mahlzeit. |
| Sumpfotter-Drüse | Kadaver | 800 – 2.500 | Schatten 3, Sturm 1 | 3 | Ja | Gefährlich zu tragen. |
| Sturmglasscherbe | Stein | 1.500 – 4.000 | Sturm 3, Geist 1 | 2 | Nein | Von Blitzen verschmolzenes Glas. |
| Ältestenholz-Bernstein | Harz | 3.000 – 7.000 | Vital 3, Geist 2 | 0 | Nein | Uraltes Baumharz mit Dingen darin. |
| Drachenschuppensplitter | Kadaver | 6.000 – 14.000 | Glut 3, Vital 2 | 2 | Nein | Von einem Dornrückendrachen abgeworfen. Sehr wertvoll, zieht Ärger an. |

**Wissensstände des Spielers** je Zutat: **Unbekannt** (angezeigt mit vager Bezeichnung wie „ein blasser, feuchter Pilz“, Essenzen verborgen) und **Bekannt** (echter Name und Essenzen sichtbar). Abschnitt 9 erklärt, wie der Spieler lernt.

### 5.3 Tränke

| Trank | Rezeptanforderung (Summe der Essenzen der verwendeten Zutaten) | Preisspanne | Kampfnutzen |
| --- | --- | --- | --- |
| Heiltrunk | Vital ≥ 3 | 40 – 120 | Heilt 40 LP |
| Klarsicht-Tonikum | Geist ≥ 3 | 80 – 260 | Heilt einen Statuseffekt, +1 Fluchtchance |
| Frostwandler-Gebräu | Frost ≥ 3 | 150 – 450 | Wurf: 30 Schaden, Gegner setzt eine Runde aus (40 %) |
| Glutherz-Tinktur | Glut ≥ 3 | 150 – 450 | Wurf: 40 Schaden |
| Schattenschleier-Elixier | Schatten ≥ 4 | 500 – 1.500 | Garantierte Flucht |
| Sturmschritt-Philtre | Sturm ≥ 4 | 700 – 2.000 | Die nächste Reise überspringt den Gefahrenwurf |
| Sehertrank | Geist ≥ 4 und Schatten ≥ 2 | 1.500 – 4.500 | Zeigt die Ereignisart am nächsten Zielort im Voraus |
| Hirschkraft | Vital ≥ 5 und Glut ≥ 2 | 2.500 – 7.000 | Heilt alle LP, +10 max. LP dauerhaft |
| Hagelsturm-Trank | Frost ≥ 3 und Sturm ≥ 3 | 3.000 – 9.000 | Wurf: 80 Schaden |
| Elixier der Langen Jahre | Vital ≥ 5 und Geist ≥ 4 | 12.000 – 30.000 | Selten benutzt. Verkauft sich für viel Geld. |
| Trüber Schlamm | Fehlgebrautes | 3 – 10 | Wertlos |

Tränke sind die Waren mit der höchsten Marge. Das Gegenstück zu den „Drachenschuppen“: Elixier der Langen Jahre und Drachenschuppensplitter sind das Alles-oder-nichts-Risiko.

## 6. Preismechanik

- Tagespreis = zufällige ganze Zahl in der Spanne der Ware, × Faktor der Preispersönlichkeit (0,7 / 1,0 / 1,3).
- **Preisereignisse** (Abschnitt 11.1) wenden einen weiteren Faktor an (×0,25 Absturz, ×3 bis ×5 Spitze), nur für den aktuellen Besuch.
- Strategieschleife 1: Günstige Zutaten an einem Ort kaufen, anderswo teuer verkaufen.
- Strategieschleife 2: Kostenlose Zutaten sammeln, Tränke brauen, Tränke dort verkaufen, wo sie teuer sind.
- Strategieschleife 3: Erforschen, um höherwertige Rezepte und die seltenen Zutaten dafür freizuschalten.
- Kein Feilschen in v1. Optionales v2-Feature: Feilschwurf auf Basis von Wissen.

## 7. Das Gepäck und Verderben

- Jede Einheit belegt 1 Platz. Grundkapazität 100.
- **Erweiterungen:** Vielfachtasche (+50 Plätze, 8.000 Kronen), Anteil am Lastenwagen (+100 Plätze, 20.000 Kronen), Träger-Söldner (+20 Plätze).
- Der Spieler kann Waren überall **ablegen**. Bei Ereignissen verlorene Waren werden entfernt.
- **Verderben:** Bei jeder Reise verliert jeder Stapel **verderblicher** Zutaten (in 5.2 mit „Verdirbt“ markiert) 10 % seiner Einheiten, aufgerundet, mindestens 1, wenn der Stapel nicht leer ist (z. B. 12 → 10, 3 → 2). Ein Beutel **Konservierungssalz** (Ausrüster, 400 Kronen, hält 5 Tage) halbiert das. Tränke mit Frost-Essenz verderben nie. Nicht verderbliche Waren sind unberührt.

## 8. Brauen

Das Brauen ist das Herz des Spiels. Es kostet 1 Tageslicht und erfordert einen Kessel (Kesselflickers Kessel wird immer mitgeführt).

**Kesselstufen**

| Kessel | Kosten | Ausbeute (Tränke pro Brauvorgang) | Max. Zutatenplätze |
| --- | --- | --- | --- |
| Kesselflickers Kessel | Start | 3 | 3 |
| Kupferkessel | 6.000 | 5 | 4 |
| Silberkessel | 15.000 | 8 | 4 |

**Ablauf**

1. Der Spieler wählt 2 bis N Zutaten aus dem Gepäck (bis zu 2 derselben Zutat). Pro Brauvorgang wird genau eine Einheit jeder gewählten Zutat verbraucht.
2. Die Engine summiert jede Essenz über alle Zutaten.
3. Die Engine prüft jeden Trank von teuerstem bis günstigstem; der erste, dessen Anforderung erfüllt ist, ist das Ergebnis. Passt keiner, entsteht **Trüber Schlamm**.
4. **Missgeschick-Prüfung:** Missgeschickchance = 5 % × (Gesamtflüchtigkeit der Zutaten) − 3 % × Wissen, begrenzt auf 0–40 %. Bei einem Missgeschick ist der Sud verdorben (Zutaten verloren), der Spieler erleidet 10–30 Schaden („Der Kessel platzt!“) und es entsteht kein Trank.
5. Bei Erfolg erhält der Spieler *Ausbeute* Einheiten des Tranks. War das Rezept noch nicht bekannt, wird es **entdeckt** (+1 Wissen) und ein Jubeltext erscheint.
6. Jede verwendete Unbekannte Zutat wird **enthüllt** (wird Bekannt), egal ob der Sud gelingt, misslingt oder schiefgeht.
7. Der Spieler kann nur Kombinationen versuchen, die er noch nicht ausprobiert hat; das Spiel merkt sich jede versuchte Kombination im **Grimoire**.

Ein Spieler, der ein Rezept noch nicht entdeckt hat, kann es trotzdem versehentlich brauen. Ein Spieler, der ein Rezept gelernt hat, sieht es im Grimoire mit dem Vermerk „Bekannt“ und kann die Schaltfläche **Rezeptvorschlag** nutzen, die bekannte Zutaten im Gepäck auflistet, die die Anforderung erfüllen.

## 9. Wissen und Kunde

Wissen (0–10) ist das Können des Zauberers.

| Wissenseffekt | Regel |
| --- | --- |
| Sammeln | Ertragsbonus +5 % pro Wissenspunkt |
| Brauen | Missgeschickchance um 3 % pro Wissenspunkt gesenkt |
| Schätzen | Ab Wissen ≥ 4 werden Hinweise auf die wahre Preisspanne bekannter Waren angezeigt („meist 800–2.500“) |
| Erkennen | Ab Wissen ≥ 6 werden beim Sammeln gefundene neue Zutaten automatisch erkannt |

**Wege, Wissen zu erlangen**

- **Erforschen (1 Tageslicht):** Eine Unbekannte Zutat aus dem Gepäck wählen, 1 Einheit aufwenden und eine Wissensprobe ablegen: Erfolgschance = 30 % + 10 % × Wissen. Bei Erfolg wird die Zutat enthüllt (+1 Wissen mit 50 % Chance). Bei Misserfolg ist die Einheit verloren.
- **Weiser (Silberbirken-Akademie):** Kauft und verkauft Rezeptseiten (ein bekanntes Rezept) und Feldnotizen (eine enthüllte Zutat). Preise 300 – 6.000 Kronen.
- **Moorhexe (Trübwasser-Moor):** Verkauft Gerüchte für 100 – 500 Kronen, die eine Essenz einer Unbekannten Zutat enthüllen.
- **Schwarzmarkt (Laternenmulde):** Verkauft seltene Rezepte zu hohen Preisen.
- **Entdeckung durch Brauen** (Abschnitt 8).
- **Fundstücke:** Zufallsereignisse können Feldnotizen, eine Rezeptseite oder einen Wissenspunkt bringen.

## 10. Sammeln

Sammeln kostet 1 Tageslicht. Der Spieler durchstreift das Gelände und die Engine würfelt auf der **Sammeltabelle** des Bioms.

- 2–5 Würfe (mehr bei höherem Wissen); jeder Wurf wählt nach Gewicht eine Zutat.
- Ein Sammelwurf hat pro Schädel Gefahr eine Chance von 15 %, stattdessen eine **Gefahr** auszulösen: Dornen (−5 LP), Bienenschwarm (−10 LP), Erdloch (1 zufällige Einheit verloren), eine seltene Sichtung (Stimmungstext, +1 Wissen).
- Ertrag pro Zutat: 1–4 Einheiten (×1,05 pro Wissenspunkt).

| Biom | Sammeltabelle (Gewichte) |
| --- | --- |
| Wiese | Tauhut 40, Frostfarn 30, Mottenflügelstaub 20, Dämmerbeere 10 |
| Sumpf | Dämmerbeere 35, Tauhut 20, Sumpfotter-Drüse 5, Rabenschädel 5, Glimmermoos 20, Wolfszahn 15 |
| Gebirge | Kaltquarz 30, Frostfarn 25, Sturmglasscherbe 5, Donnerdistel 20, Wolfszahn 20 |
| Küste | Mottenflügelstaub 30, Kaltquarz 15, Sturmglasscherbe 5, Tauhut 25, Rabenschädel 10, Dämmerbeere 15 |
| Obstgarten und Bibliothek | Glimmermoos 35, Tauhut 25, Dämmerbeere 15, Mottenflügelstaub 20, Ältestenholz-Bernstein 5 |
| Vulkan | Aschengeode 30, Aschenblüten-Pilz 40, Donnerdistel 10, Drachenschuppensplitter 3, Wolfszahn 17 |
| Tiefer Wald | Ältestenholz-Bernstein 8, Rabenschädel 15, Wolfszahn 20, Glimmermoos 20, Dämmerbeere 20, Tauhut 17 |
| Schmugglerstadt | (kein Sammeln; der Spieler kann die Gassen **durchstöbern** und findet nur Mottenflügelstaub oder Tauhut) |
