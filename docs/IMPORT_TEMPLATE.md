# Import Template

You can import activity lists into Hazardo using **Markdown**, **JSON**, or **CSV** format.

Go to **Settings > Import** and select your file.

---

## Markdown Format

The easiest way to create a list. Use this template:

```markdown
## Restaurant (restaurant)

| Item         | Time | Vibe   | Notes              |
| ------------ | ---- | ------ | ------------------ |
| Sushi Place  | PM   | Date   | Great omakase      |
| Burger Joint | AM   | Friend | Best smash burgers |
| Thai Express | PM   | Family | Kid-friendly       |

## Outdoor (outdoor)

| Item               | Time | Vibe   | Notes           |
| ------------------ | ---- | ------ | --------------- |
| Hiking Mont-Royal  | AM   | Friend | 1h loop trail   |
| Picnic at the park | PM   | Date   | Bring a blanket |
| Beach day          | AM   | Family | Sunscreen!      |

## Movies (movies)

| Item            | Time  | Vibe   | Notes               |
| --------------- | ----- | ------ | ------------------- |
| Cinema night    | Night | Date   |                     |
| Marvel marathon | PM    | Friend | All day event       |
| Disney movie    | PM    | Family | Kids pick the movie |
```

### Rules

- `## Category Name (icon)` — starts a new category. The icon in parentheses is optional.
- Table must have 4 columns: **Item**, **Time**, **Vibe**, **Notes**
- **Time** values: `AM`, `PM`, `Night`, `Mixed`
- **Vibe** values: `Friend`, `Date`, `Family`, `Mixed`
- Notes are optional (leave empty)

---

## JSON Format

```json
[
  {
    "user": "your_name",
    "categories": [
      {
        "name": "Restaurant",
        "icon": "restaurant",
        "items": [
          {
            "name": "Sushi Place",
            "time_pref": "PM",
            "vibe_pref": "Date",
            "notes": "Great omakase"
          },
          {
            "name": "Burger Joint",
            "time_pref": "AM",
            "vibe_pref": "Friend",
            "notes": "Best smash burgers"
          }
        ]
      },
      {
        "name": "Outdoor",
        "icon": "outdoor",
        "items": [
          {
            "name": "Hiking Mont-Royal",
            "time_pref": "AM",
            "vibe_pref": "Friend",
            "notes": "1h loop trail"
          }
        ]
      }
    ]
  }
]
```

---

## CSV Format

```csv
User,Category,Icon,Item,TimePref,VibePref,Notes
"your_name","Restaurant","restaurant","Sushi Place","PM","Date","Great omakase"
"your_name","Restaurant","restaurant","Burger Joint","AM","Friend","Best smash burgers"
"your_name","Outdoor","outdoor","Hiking Mont-Royal","AM","Friend","1h loop trail"
"your_name","Outdoor","outdoor","Picnic at the park","PM","Date","Bring a blanket"
```

---

## ZIP Format (with images)

For imports that include images, create a ZIP file with:

```
my_import.zip
├── data.json          (same JSON format as above)
└── images/
    └── pick_1.jpg     (optional images for picks)
```

Max file sizes: **10MB** for JSON/CSV/Markdown, **50MB** for ZIP.
