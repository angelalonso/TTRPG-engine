# Mondwurzel – Datensatz

Dieser Datensatz implementiert das in [`docs/000_info_mondwurzel_story.md`](../docs/000_info_mondwurzel_story.md)
beschriebene Händler-/Alchemie-Spiel **ausschließlich mit Datensatz-Dateien** (CSV + HTML + Bilder),
ohne jede Änderung am Rust-Engine-Code oder am Frontend. Struktur und Konventionen folgen `gtr2career`
als Vorbild (`config.csv`, `player.csv`, `objects.csv`, `events.csv`, `condition_groups.csv`,
`conditions.csv`, `requirement_bindings.csv`, `effects.csv`, `effect_bindings.csv`,
`numeric_modifiers.csv`, `html/`, `img/`, `saves/`).

Validiert mit `cargo run --bin validate_dataset -- ../mondwurzel`: **0 Fehler, 1 harmlose Warnung**
(`objects.csv` Spalte `image_path` wird von der aktuellen Engine-Version geladen, aber nicht
ausgewertet – derselbe informative Hinweis erscheint auch bei `gtr2career`, das dieselbe Spalte nutzt).

## Spielkern (umgesetzt)

- **8 Orte** (Disteltal, Trübwasser-Moor, Hochstein, Salzmoor-Hafen, Silberbirken-Akademie,
  Glutfall-Krater, Dämmerwald, Laternenmulde) als `current_location` (0–7) mit Reise-Events
  (`travel_0` … `travel_7`), die ortsabhängig gesperrt sind (man kann nicht zum aktuellen
  Ort "reisen").
- **15 Zutaten** und **11 Tränke** als handelbare `objects.csv`-Einträge (kaufbar/verkaufbar,
  feste Preise in Kronen).
- **Sammeln** (`sammeln_0` … `sammeln_7`), ein Event pro Ort, das die laut Story wichtigsten
  Zutaten dieses Biomes gewährt und 1 Tageslicht sowie HP (abhängig von der Gefahrenstufe) kostet.
- **Brauen** (`brauen_pot_*`), ein Event pro Trank mit Rezept, das die passenden Zutaten
  verbraucht, 1 Tageslicht kostet und 3 Einheiten des Tranks gewährt.
- **Erforschen** (+1 Wissen), **Heiler-Dienst** (HP voll, ortsgebunden), **Bank**
  (Einzahlen/Abheben in zwei Beträgen, ortsgebunden auf Salzmoor-Hafen/Silberbirken-Akademie),
  **Kreditgeber/Schuldenrückzahlung** (zwei Beträge) und **Startausrüstung** (einmalige Gewährung
  von Stab, Robe, Kessel).
- **Ortsgebundene Läden**: Konservierungssalz (Orte 2/3), Tragetasche/Kessel-Upgrades
  (Ort 4 – Silberbirken-Akademie), Wissens-Items (Orte 1, 4, 6, 7), jeweils über
  `requirement_bindings.csv` mit `operation=acquire` gesperrt – getestet und bestätigt über
  `GET /transactions` (`can_acquire=false` mit korrektem `acquire_reason`, bis man am
  richtigen Ort ist).
- **Schulden** (`debt`), die bei jeder Reise um 2 % verzinst werden (siehe Vereinfachungen).
- **Charakterwerte**: `gold`, `debt`, `bank`, `hp` (0–100), `knowledge` (0–10),
  `daylight` (0–2, pro Tag zurückgesetzt durch Reisen), `day` (1–30), `bag_capacity`,
  `current_location`.

## Wichtige Engine-Erkenntnisse (warum einige Dinge anders umgesetzt sind)

Beim Testen mit dem `game_api`-Server und durch Lesen von `src-tauri/src/lib.rs` wurden
reale Engine-Grenzen entdeckt, die bestimmte Spiel-Mechaniken aus der Spezifikation
**datensatz-seitig unmöglich** machen:

1. **`requirement_group`-Spalte auf `events.csv`/`objects.csv` ist NICHT die generische
   Freischalt-Logik** – sie dient nur der Ausrüstungs-Exklusivität. Die echte, funktionierende
   Freischalt-Logik ist `requirement_bindings.csv` (+ `condition_groups.csv`/`conditions.csv`),
   mit `operation` = `event_start` (für Events) bzw. `acquire` (für Käufe).
2. **Nur `event.type == "work"`, `"trade"` oder `"finance"`** werden im generischen
   Aktivitäten-UI angezeigt (`activityBelongsToTab` in `src/App.tsx`). Alle Mondwurzel-Events
   nutzen daher `type="work"`.
3. **`stamina_cost` muss `0` sein** auf allen Events: Da Mondwurzel bewusst keine
   `resource_role_recovery`-Zuordnung hat (um gtr2career's hartcodierte nächtliche
   Ausdauer-Erholung zu vermeiden), würde die UI den Start-Button sonst dauerhaft deaktivieren
   (sie verweist auf eine nicht existente Charakteristik `"stamina"`).
4. **`consume_object`/`remove_object`/`grant_object`-Effekte verbieten die Menge 0 und
   schlagen hart fehl (Fehler, kein stilles No-Op), wenn der Spieler nicht genug vom
   Zielobjekt besitzt** – auch mit `numeric_modifiers.csv`-Ausdrücken (`min`/`max`/
   `inventory_count(...)`) lässt sich das nicht umgehen, da eine auf 0 geklemmte Menge
   ebenfalls einen Fehler auslöst. **Konsequenz:** Die geplante Mechanik "verderbliche
   Zutaten verfallen beim Reisen" (ein `consume_object`-Effekt, der bedingungslos an jedes
   Reise-Event gebunden wäre) ist mit Bordmitteln des Datensatzes **nicht umsetzbar** und
   wurde komplett entfernt.
5. **`perform_event` löst nur den Trigger `"event_started"` aus, nie `"event_completed"`,
   und zwar unbedingt, unabhängig vom internen Erfolg/Fehlschlag-Wurf.** Deshalb sind
   Sammeln/Brauen/Erforschen in diesem Datensatz **deterministisch** (kein Zufalls-/
   Missgeschick-System möglich).

## Vereinfachungen gegenüber der Spezifikation (`000_info_mondwurzel_story.md`)

| Thema | Spezifikation | Umsetzung hier | Grund |
| --- | --- | --- | --- |
| Rezepte | Essenz-Summen (6 verborgene Essenzwerte pro Zutat, Schwellenwerte pro Trank) | Feste Zutat-Kombinationen pro Trank (siehe unten) | Die Engine hat keine datensatz-definierbare Summen-/Schwellenwert-Logik für Rezepte; `requirement_bindings`/`conditions` prüfen nur Bestand, Charakterwerte und Ort. |
| Verborgenes Wissen | Essenzen sind anfangs unbekannt, werden durch Wissens-Items/Analyse aufgedeckt | Keine verborgene Information – alle Rezepte sind von Anfang an klar | Es gibt keinen datensatz-seitigen Mechanismus für "verborgene" Objekteigenschaften, die sich durch Spielfortschritt aufdecken. |
| Braumissgeschicke / Flüchtigkeit | Zufälliger Fehlschlag beim Brauen je nach Flüchtigkeit der Zutaten | Brauen ist immer erfolgreich (deterministisch) | `perform_event` feuert Effekte unbedingt vor jedem Erfolg/Fehlschlag-Wurf; es gibt keine datensatz-seitige Möglichkeit, Effekte an das interne Ergebnis zu binden. |
| Verderbliche Zutaten | Zutaten verderben nach X Tagen bzw. beim Reisen | **Komplett entfernt** – keine Verderblichkeit | `consume_object`/`remove_object` schlagen hart fehl, wenn die Zielmenge 0 ist oder der Spieler das Objekt nicht besitzt; ein unbedingt an jedes Reise-Event gebundener Verfalls-Effekt würde das Spiel bei jeder Reise ohne die betroffene Zutat zum Absturz bringen. Auch `numeric_modifiers`-Klemmung auf `min(1, inventory_count(...))` hilft nicht, da 0 ebenfalls verboten ist. |
| Marktpreise | Pro Ort tägliche Zufallszahl im Preisband, multipliziert mit Preispersönlichkeit (günstig ×0,7 / teuer ×1,3) und Verfügbarkeits-Würfen (~70 %/60 %) | Ein fester Preis pro Ware (Mittelwert der Preisspanne), überall gleich, immer verfügbar | `objects.csv`-Preise sind statisch; es gibt keine datensatz-seitige Zufalls- oder Orts-Preislogik. |
| Sammeln | 2–5 gewichtete Würfe aus einer Orts-Tabelle mit ~39 Einträgen total | Ein deterministisches Sammeln-Event pro Ort, das die 2–3 laut Tabelle wichtigsten Zutaten dieses Ortes in fester Menge (2/1/1) gewährt | Vereinfachte Annäherung an "durchschnittlicher Sammelertrag" ohne RNG-Unterstützung der Engine für gewichtete Tabellen. |
| Gefahr beim Sammeln | Variable, ereignisabhängige Risiken | Fixer HP-Verlust = Gefahrenstufe des Ortes × 2 | Einfache deterministische Näherung. |
| Zinsen auf Schulden | Tägliche Verzinsung | 2 % Zinsen, ausgelöst bei jeder Reise (da Reisen in dieser Umsetzung den Tag beendet) | Es gibt keinen automatischen Tag-Wechsel-Trigger im Datensatz; Reisen ist der einzige Event-Typ, der den Tag vorantreibt. |
| Kesselverleih/-Upgrades, Konservierungssalz | Mechanisch relevant (Brauqualität, Haltbarkeit) | Rein kosmetische Sammelobjekte ohne Spielmechanik-Effekt | Direkte Folge der oben beschriebenen Engine-Grenzen (Punkt 4 und 5). |
| Tag/Uhr | Eingebettet in ein eigenes Kalendersystem | Eigene Charakteristik `day` (1–30), unabhängig vom internen Engine-Kalender (`calendar_day`) | Einfachste robuste Lösung innerhalb der Datensatz-Grenzen; keine automatische 30-Tage-Spielende-Erzwingung in der Tauri-UI (nur ein Headless-Testskript könnte das separat auswerten). |
| Essenz-/Flüchtigkeitswerte der Zutaten | Sechs zusätzliche numerische Spalten pro Zutat | Nicht als Spalten abgebildet, nur in den HTML-Beschreibungen erwähnt (Flavortext) | `objects.csv` hat kein Schema für beliebige Zusatzattribute, die von Rezept-Validierung gelesen werden könnten – die Rezepte sind stattdessen direkt als feste Zutatenlisten in `requirement_bindings`/`effects` kodiert. |

### Feste Rezepte (anstelle von Essenz-Schwellenwerten)

| Trank | Zutaten |
| --- | --- |
| Heiltrunk (Vital≥3) | 2× Tauhut-Pilz |
| Klarsicht-Tonikum (Geist≥3) | 1× Mottenflügelstaub + 1× Glimmermoos |
| Frostwandler-Gebräu (Frost≥3) | 2× Frostfarnblatt |
| Glutherz-Tinktur (Glut≥3) | 1× Aschengeode |
| Schattenschleier-Elixier (Schatten≥4) | 1× Sumpfotter-Drüse + 1× Dämmerbeere |
| Sturmschritt-Philtre (Sturm≥4) | 2× Donnerdistel |
| Sehertrank (Geist≥4 & Schatten≥2) | 2× Rabenschädel |
| Hirschkraft-Essenz (Vital≥5 & Glut≥2) | 1× Drachenschuppensplitter + 1× Ältestenholz-Bernstein |
| Hagelsturm-Trank (Frost≥3 & Sturm≥3) | 2× Kaltquarz + 1× Sturmglasscherbe |
| Elixier der Langen Jahre (Vital≥5 & Geist≥4) | 2× Ältestenholz-Bernstein |
| Trüber Schlamm | *(kein Rezept – nur als Flavor-Objekt vorhanden, nicht brau- oder kaufbar über ein Event)* |

## Bilder

Alle Bilder in `img/` (inklusive `player.jpeg`) sind **neutrale Platzhalter** (einfarbige
256×256-JPEGs), wie vom Nutzer gewünscht ("leave blank placeholders for now"). Sie können
später durch echte Artworks ersetzt werden, ohne Dateinamen oder `objects.csv`-Referenzen
zu ändern.

## Testen

Manuell verifiziert über den `game_api`-HTTP-Server
(`DATASET_PATH=../mondwurzel cargo run --bin game_api`):

- Startausrüstung (einmalig, zweiter Versuch korrekt abgelehnt).
- Sammeln (korrekte Zutaten-Gewährung, Tageslicht-/HP-Kosten).
- Brauen (korrekter Zutatenverbrauch + Trank-Gewährung, kein Fehlschlag mehr nach Entfernen
  der Verderblichkeits-Effekte).
- Reisen (Ortswechsel, Tag+1, Tageslicht-Reset, 2 % Schuldzinsen); Reise zum aktuellen Ort
  korrekt abgelehnt.
- Erforschen, Heiler-Dienst (inkl. Ablehnung am falschen Ort), Bank (inkl. Ablehnung am
  falschen Ort).
- Ortsgebundene Käufe (`GET /transactions`): `can_acquire=false` mit korrektem
  `acquire_reason` am falschen Ort, `can_acquire` wird nach Ortswechsel korrekt neu bewertet.

`cargo run --bin validate_dataset -- ../mondwurzel` → 0 Fehler, 1 harmlose Warnung
(siehe oben).
