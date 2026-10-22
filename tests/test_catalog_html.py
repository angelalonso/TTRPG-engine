import csv
from html.parser import HTMLParser
from pathlib import Path
import unittest


DATASET = Path(__file__).resolve().parents[1] / "gtr2career"


class Tags(HTMLParser):
    def __init__(self, html):
        super().__init__()
        self.tags = []
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        self.tags.append((tag, dict(attrs)))


def rows(filename):
    with (DATASET / filename).open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


class CatalogHtmlTests(unittest.TestCase):
    def test_cars_and_race_gear_start_with_picture_not_duplicate_title(self):
        objects = [
            row for row in rows("objects.csv")
            if row["type"] == "vehicle" or row["description_html"].startswith("./html/gear_")
        ]
        self.assertGreater(len(objects), 20)
        for row in objects:
            with self.subTest(id=row["id"]):
                html = (DATASET / row["description_html"]).read_text(encoding="utf-8")
                left = html.split("<main", 1)[1].split("</main>", 1)[0]
                tags = Tags(left).tags
                self.assertFalse(any(tag in ("h1", "header") for tag, _ in tags))
                self.assertFalse(any("header" in attrs.get("class", "").split() for _, attrs in tags))
                self.assertLess(left.index('class="catalog-image"'), left.index('class="catalog-details"'))

    def test_events_and_championships_have_images_and_organizer(self):
        entries = [
            (row, False) for row in rows("events.csv") if int(row["day_of_year"]) > 0
        ] + [
            (row, True) for row in rows("quests.csv") if row["type"] == "championship"
        ]
        self.assertGreater(len(entries), 100)
        for row, championship in entries:
            with self.subTest(id=row["id"]):
                self.assertTrue((DATASET / row["image_path"]).is_file())
                html = (DATASET / row["description_html"]).read_text(encoding="utf-8")
                left = html.split("<main", 1)[1].split("</main>", 1)[0]
                right = html.split("<aside", 1)[1].split("</aside>", 1)[0]
                self.assertIn("180 Motor Club", right)
                images = [attrs for tag, attrs in Tags(right).tags if tag == "img"]
                self.assertEqual(images[0]["src"], "../img/organizer_180_motor_club.svg")
                self.assertLess(left.index('class="catalog-image"'), left.index('class="catalog-details"'))
                if not championship:
                    self.assertIn('class="catalog-card silver"', html)
                    self.assertIn('class="catalog-side silver"', html)
                    self.assertIn("EVENT DETAILS", left)
                    self.assertNotIn("MAINTENANCE", left)


if __name__ == "__main__":
    unittest.main()
