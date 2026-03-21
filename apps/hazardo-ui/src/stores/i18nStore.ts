import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type Lang = "en" | "fr";

export const currentLang = writable<Lang>("en");

const translations: Record<Lang, Record<string, string>> = {
  en: {
    // Home page
    "home.title": "Select Criteria",
    "home.time_pref": "Time Preference:",
    "home.vibe_pref": "Vibe Preference:",
    "home.list_pref": "List Preference:",
    "home.date_pref": "Date Preference:",
    "home.advance_roll": "Advance Roll (need AI setting to work)",
    "home.roll_dice": "Roll Dice",
    "home.rolling": "Rolling...",
    "home.pick_this": "Pick this!",
    "home.roll_again": "Roll Again",
    "home.no_items": "No items available. Add some in the Vault first!",
    "home.add_items_first": "Add items in the Vault first!",
    "home.close": "Close",
    "home.roll_error": "Roll failed — please try again.",
    "home.hazardo_pick": "Hazardo Pick",
    "home.date": "Date:",
    "home.time": "Time",
    "home.vibe": "Vibe",
    "home.location": "Location",
    "home.loading": "Loading...",
    "home.disabled": "Disabled",
    "home.na": "N/A",
    "home.ai_recommendation": "AI Recommendation",
    "home.thinking": "Thinking...",
    "home.all_lists": "All Lists",

    // Picked page
    "picked.title": "Hazardo Picked",
    "picked.filter": "Filter Categories:",
    "picked.all_categories": "All Categories",
    "picked.list_title": "List of Picked:",
    "picked.no_picks": "No picks yet. Roll the dice on the Home page!",
    "picked.notes": "Notes",
    "picked.image": "Image",
    "picked.add_notes": "Add notes about this event...",
    "picked.take_photo": "Take Photo",
    "picked.upload": "Upload",
    "picked.cancel": "Cancel",
    "picked.save": "Save",
    "picked.delete_image_title": "Delete Image?",
    "picked.delete_image_msg":
      "Are you sure you want to remove this image? This action cannot be undone.",
    "picked.delete": "Delete",
    "picked.pick_deleted": "Pick moved to recycle bin",

    // Vault page
    "vault.title": "Activity Vault",
    "vault.title_of": "Vault of",
    "vault.select_categories": "Select Categories:",
    "vault.all_lists": "All Lists",
    "vault.item_list": "Item List:",
    "vault.no_items": "No items yet. Add your first item!",
    "vault.add_category": "Add Category",
    "vault.edit_category": "Edit Category",
    "vault.category_name": "Category Name:",
    "vault.category_name_ph": "Enter category name...",
    "vault.icon": "Icon:",
    "vault.add": "Add",
    "vault.done": "Done",
    "vault.add_item": "Add Item",
    "vault.edit_item": "Edit Item",
    "vault.item_name": "Item Name:",
    "vault.item_name_ph": "Enter item name...",
    "vault.category": "Category:",
    "vault.select_category": "Select category",
    "vault.time_pref": "Time Preference:",
    "vault.vibe_pref": "Vibe Preference:",
    "vault.notes_optional": "Notes (optional):",
    "vault.add_notes_ph": "Add notes...",
    "vault.already_picked": "Already Picked (strikethrough in list)",
    "vault.item_deleted": "Item moved to recycle bin",
    "vault.category_deleted": "Category moved to recycle bin",
    "vault.category_has_items": "Can't delete — this category still has items",

    // Chatbot page
    "chatbot.needs_config": "ChatBot needs configuration",
    "chatbot.config_msg":
      "Go to Settings > Manage LLM Model (AI) to configure your AI connection.",
    "chatbot.start_convo": "Start a conversation with Hazardo AI!",
    "chatbot.thinking": "Thinking...",
    "chatbot.ai_wants_add": "AI wants to add to your database:",
    "chatbot.category": "Category:",
    "chatbot.item": "Item:",
    "chatbot.confirm": "Confirm",
    "chatbot.decline": "Decline",
    "chatbot.items_added": "Items added to your vault!",
    "chatbot.select_category": "Choose a category for the items:",
    "chatbot.create_new": "+ Create new",
    "chatbot.cancel": "Cancel",
    "chatbot.model": "Model:",
    "chatbot.placeholder": "Ask me anything...",
    "chatbot.settings": "Settings",

    // Bottom nav
    "nav.picked": "Picked",
    "nav.vault": "Vault",
    "nav.home": "Home",
    "nav.chatbot": "ChatBot",
    "nav.setting": "Setting",

    // Settings page
    "settings.title": "Settings",
    "settings.documentation": "Documentation",
    "settings.recycle_bin": "Recycle Bin",
    "settings.bulk_import": "Bulk Import",
    "settings.bulk_export": "Bulk Export",
    "settings.theme": "Theme",
    "settings.theme_light": "Light",
    "settings.theme_dark": "Dark",
    "settings.language": "Language",
    "settings.device_name": "Device Name",
    "settings.manage_user": "Manage Users",
    "settings.manage_sync": "Synchronization",
    "settings.location_services": "Location",
    "settings.manage_llm": "LLM Model (AI)",
    "settings.save": "Save",
    "settings.cancel": "Cancel",
    "settings.english": "English",
    "settings.french": "Français",

    // Manage User
    "user.edit": "Edit",
    "user.edit_name": "Edit Username",
    "user.new_name": "New Name:",
    "user.new_name_ph": "Enter new name...",

    // Recycle bin
    "recycle.vault": "Vault",
    "recycle.categories": "Categories",
    "recycle.picked": "Picked",

    // Export/Import
    "export.select_users": "Select Users:",
    "export.all_users": "All Users",
    "export.select_categories": "Select Categories:",
    "export.all_categories": "All Categories",
    "export.select_picks": "Select Picked Items:",
    "export.include_picks": "Include Picked Items (roll history)",
    "export.all_picks": "All Picks",
    "export.format": "Export Format:",
    "export.export": "Export",
    "export.exporting": "Exporting...",
    "export.result": "Export Result",
    "export.preview": "Preview",
    "import.format": "Import Format:",
    "import.file": "File:",
    "import.paste": "Or paste content directly:",
    "import.placeholder": "Paste JSON/CSV/Markdown content...",
    "import.import": "Import",
    "import.importing": "Importing...",

    // LLM settings
    "llm.provider": "Provider:",
    "llm.api_key": "API Key:",
    "llm.endpoint": "Endpoint URL:",
    "llm.model": "Model:",
    "llm.model_manual": "Model (manual):",
    "llm.system_prompt": "System Prompt:",
    "llm.allow_create": "Allow AI to create items/categories",
    "llm.test": "Test",
    "llm.not_configured": "Not configured",
    "llm.openai_label": "OpenAI (GPT)",
    "llm.gemini_label": "Google Gemini",
    "llm.local_label": "Local LLM (Ollama/LM Studio)",
    "llm.api_key_openai": "OpenAI API Key:",
    "llm.api_key_gemini": "Gemini API Key:",
    "llm.api_key_ph": "Enter API key...",
    "llm.model_ph": "Select model...",
    "llm.endpoint_ph": "http://localhost:11434",

    // Recycle bin actions
    "recycle.restore": "Restore",
    "recycle.delete_permanent": "Delete permanently",

    // Missing keys
    "chatbot.error": "Failed to add items",
    "picked.image_too_large": "Image must be under 5 MB",
    "picked.download_image": "Save to gallery",
    "picked.image_saved": "Image saved to gallery",
    "picked.image_save_failed": "Failed to save image",

    // Location
    "location.permission": "Location Permission",
    "location.enabled": "Location access enabled",
    "location.requesting": "Requesting permission...",
    "location.allow": "Allow Location Access",
    "location.denied":
      "Permission denied. You may need to enable it in your device settings.",
    "location.roll_dice": "Roll Dice Location",
    "location.description":
      "Choose a specific location for roll dice, or use your current GPS position.",
    "location.remove": "Remove",
    "location.using_gps": "Using current GPS location (default)",
    "location.search_city": "Search a city...",
    "location.search": "Search",

    // Sync
    "sync.description":
      "Synchronization requires the Hazardo desktop API server. Configure and start the Docker container on your network to enable sync.",
    "sync.in_development": "In Development",

    // Setup page
    "setup.welcome": "Welcome to Hazardo!",
    "setup.device_desc":
      "Identify this device to enable seamless syncing across all your devices.",
    "setup.device_name": "Device Name:",
    "setup.device_name_ph": "Enter Device name...",
    "setup.user_desc":
      "Create a username to get started. You can host multiple users on this device, each with their own unique categories and lists.",
    "setup.username": "Username:",
    "setup.username_ph": "Enter Username...",
    "setup.location_desc":
      "Allow location access for weather info and local recommendations when rolling dice.",
    "setup.location_granted": "Location access granted",
    "setup.location_denied": "Location access denied",
    "setup.location_try_again": "Try again",
    "setup.alert_device_name": "Please enter a device name",

    // Documentation
    "doc.tap_section": "Tap a section to learn more about each feature.",
    "doc.home_title": "Home (Roll Dice)",
    "doc.home_desc":
      "The Home page is where you roll the dice to get a random activity pick.",
    "doc.home_time":
      "Choose AM, PM, Night, or Mixed to filter activities by time of day.",
    "doc.home_vibe":
      "Choose Friend, Date, Family, or Mixed to match your mood.",
    "doc.home_list":
      'Pick a specific category or leave on "All Lists" for a random pick from everything.',
    "doc.home_date": "Set the date for your activity (defaults to today).",
    "doc.home_advance":
      "Enable this checkbox to get AI-powered recommendations alongside your roll. Requires an AI provider configured in Settings → LLM Model.",
    "doc.home_roll":
      "Tap the button to get a random pick! A modal will show the result with activity details, weather info, and AI recommendations (if enabled).",
    "doc.home_pick": "Save the roll to your Picked history.",
    "doc.home_roll_again":
      "Don't like the result? Roll again for a new random pick.",
    "doc.vault_title": "Vault",
    "doc.vault_desc":
      "The Vault is where you manage all your categories and items.",
    "doc.vault_categories":
      "Organize your activities into categories (e.g., Restaurant, Movie, Workout). Each category has a name and an icon.",
    "doc.vault_create_cat":
      'Tap the "+" button to add a new category. Choose a name and select an icon.',
    "doc.vault_add_items":
      'Inside a category, tap "+" to add activities. Each item can have:',
    "doc.vault_item_name": "The activity name",
    "doc.vault_item_time": "When the activity fits best (AM/PM/Night/Mixed)",
    "doc.vault_item_vibe": "Who it's best for (Friend/Date/Family/Mixed)",
    "doc.vault_item_notes": "Optional notes about the activity",
    "doc.vault_edit_delete":
      "Tap an item to edit it, or tap the trash icon to delete. Deleted items go to the Recycle Bin.",
    "doc.picked_title": "Picked",
    "doc.picked_desc": "View the history of all your picked activities.",
    "doc.picked_filter":
      "Use the category dropdown and search bar to find specific picks.",
    "doc.picked_detail":
      "Tap a pick to see full details including date, time, vibe, category, and location/weather at the time of the pick.",
    "doc.picked_ai":
      "If Advance Roll was enabled, tap the collapsible AI section to review the recommendation (places, tips, etc.).",
    "doc.picked_notes": "Add personal notes about how the activity went.",
    "doc.picked_image":
      "Attach a photo from your camera or gallery to remember the moment.",
    "doc.chatbot_title": "ChatBot",
    "doc.chatbot_desc":
      "An AI-powered assistant to help you plan activities and get recommendations.",
    "doc.chatbot_requires":
      "An AI provider must be configured in Settings → LLM Model (OpenAI, Gemini, or Local LLM).",
    "doc.chatbot_chat":
      "Type a message and press Send to ask the AI anything—activity ideas, restaurant recommendations, travel tips, etc.",
    "doc.chatbot_model":
      "If multiple models are available, use the dropdown above the input to switch models.",
    "doc.chatbot_auto_create":
      "If enabled in LLM settings, the AI can suggest creating categories and items in your database. You'll see a confirmation prompt before anything is added.",
    "doc.chatbot_clear":
      "Use the trash icon in the header to clear the conversation history.",
    "doc.settings_title": "Settings",
    "doc.settings_recycle":
      "View and restore deleted items, categories, or picks. You can also permanently delete them.",
    "doc.settings_import":
      "Import data from a JSON, CSV, or Markdown file to add items and categories in bulk.",
    "doc.settings_export":
      "Export your data (users, categories, items, picked history) in JSON, CSV, Markdown, or ZIP format.",
    "doc.settings_device":
      "Set a name for this device (used for sync identification).",
    "doc.settings_user": "View, edit, and delete user profiles.",
    "doc.settings_sync":
      "Configure sync with the Hazardo desktop API server (requires Docker).",
    "doc.settings_location":
      "Enable GPS location for weather-aware recommendations. You can also set a manual city override.",
    "doc.settings_llm": "Configure your AI provider:",
    "doc.settings_llm_openai": "Enter your API key. Supports GPT models.",
    "doc.settings_llm_gemini": "Enter your API key. Supports Gemini models.",
    "doc.settings_llm_local":
      "Enter the endpoint URL of your local Ollama/LM Studio server.",
    "doc.settings_llm_prompt": "Customize how the AI assistant behaves.",
    "doc.settings_llm_auto":
      "Let the ChatBot add categories and items to your database (with confirmation).",

    // AI system prompts
    "ai.system_prompt":
      "You are Hazardo, a smart activity recommendation AI. The user just rolled a random activity pick. Provide a useful, concise recommendation.",
    "ai.chatbot_default_prompt":
      "You are Hazardo AI assistant. Help the user with activity planning and recommendations.",
    "ai.respond_instruction": "Always respond in English.",
    "ai.format_quick_take": "Quick Take",
    "ai.format_top_picks": "Top Picks Near You",
    "ai.format_pro_tips": "Pro Tips",
    "ai.format_what_to_bring": "What to Bring",
    "ai.format_alternative_ideas": "Alternative Ideas",
    "ai.format_quick_take_desc":
      "2-3 sentences summarizing why this is a great pick given weather, time, and vibe.",
    "ai.format_top_picks_desc":
      "List 3-5 real, specific places near the user relevant to the activity.",
    "ai.format_top_picks_item": "**Place Name** — 1 sentence description",
    "ai.format_top_picks_links":
      "[Directions](https://www.google.com/maps/dir/?api=1&destination=PLACE+NAME+CITY) · [Search](https://www.google.com/search?q=PLACE+NAME+CITY)",
    "ai.format_pro_tips_desc":
      "3-4 short, practical tips for the activity based on weather and context.",
    "ai.format_what_to_bring_desc": "3-5 items to bring or prepare.",
    "ai.format_alternative_desc":
      "2-3 backup activities with one-line explanations.",
    "ai.format_rules":
      "RULES:\n- Keep it concise but specific to the user's city.\n- Use real place names near the user's location.\n- Use markdown formatting (##, bold, bullets, links).\n- If weather is bad, prioritize indoor options.",
    "ai.user_msg_prefix": "I just rolled:",
    "ai.user_msg_suffix": "What do you recommend?",
  },
  fr: {
    // Home page
    "home.title": "Critères",
    "home.time_pref": "Moment :",
    "home.vibe_pref": "Ambiance :",
    "home.list_pref": "Liste :",
    "home.date_pref": "Date :",
    "home.advance_roll": "Lancer avancé (IA requise)",
    "home.roll_dice": "Lancer le dé",
    "home.rolling": "En cours...",
    "home.pick_this": "Choisir !",
    "home.roll_again": "Relancer",
    "home.no_items":
      "Aucun élément disponible. Ajoutez-en dans le Vault d'abord !",
    "home.add_items_first": "Ajoutez des éléments dans le coffre !",
    "home.close": "Fermer",
    "home.roll_error": "Échec du tirage — réessayez.",
    "home.hazardo_pick": "Tirage Hazardo",
    "home.date": "Date :",
    "home.time": "Moment",
    "home.vibe": "Ambiance",
    "home.location": "Lieu",
    "home.loading": "Chargement...",
    "home.disabled": "Désactivé",
    "home.na": "N/D",
    "home.ai_recommendation": "Recommandation IA",
    "home.thinking": "Réflexion...",
    "home.all_lists": "Toutes les listes",

    // Picked page
    "picked.title": "Sélections",
    "picked.filter": "Filtrer :",
    "picked.all_categories": "Toutes",
    "picked.list_title": "Historique :",
    "picked.no_picks": "Aucune sélection. Lancez le dé !",
    "picked.notes": "Notes",
    "picked.image": "Image",
    "picked.add_notes": "Ajouter des notes...",
    "picked.take_photo": "Photo",
    "picked.upload": "Téléverser",
    "picked.cancel": "Annuler",
    "picked.save": "Enregistrer",
    "picked.delete_image_title": "Supprimer l'image ?",
    "picked.delete_image_msg":
      "Voulez-vous supprimer cette image ? Cette action est irréversible.",
    "picked.delete": "Supprimer",
    "picked.pick_deleted": "Pick envoyé à la corbeille",

    // Vault page
    "vault.title": "Coffre",
    "vault.title_of": "Coffre de",
    "vault.select_categories": "Catégories :",
    "vault.all_lists": "Toutes les listes",
    "vault.item_list": "Éléments :",
    "vault.no_items": "Aucun élément. Ajoutez le premier !",
    "vault.add_category": "Ajouter une catégorie",
    "vault.edit_category": "Modifier la catégorie",
    "vault.category_name": "Nom :",
    "vault.category_name_ph": "Nom de la catégorie...",
    "vault.icon": "Icône :",
    "vault.add": "Ajouter",
    "vault.done": "Terminé",
    "vault.add_item": "Ajouter un élément",
    "vault.edit_item": "Modifier l'élément",
    "vault.item_name": "Nom :",
    "vault.item_name_ph": "Nom de l'élément...",
    "vault.category": "Catégorie :",
    "vault.select_category": "Choisir une catégorie",
    "vault.time_pref": "Moment :",
    "vault.vibe_pref": "Ambiance :",
    "vault.notes_optional": "Notes (optionnel) :",
    "vault.add_notes_ph": "Ajouter des notes...",
    "vault.already_picked": "Déjà choisi (barré dans la liste)",
    "vault.item_deleted": "Élément envoyé à la corbeille",
    "vault.category_deleted": "Catégorie envoyée à la corbeille",
    "vault.category_has_items":
      "Impossible de supprimer — il y a encore des éléments dans cette catégorie",

    // Chatbot page
    "chatbot.needs_config": "Configuration requise",
    "chatbot.config_msg":
      "Allez dans Paramètres > Modèle LLM (IA) pour configurer la connexion IA.",
    "chatbot.start_convo": "Commencez une conversation avec Hazardo IA !",
    "chatbot.thinking": "Réflexion...",
    "chatbot.ai_wants_add": "L'IA veut ajouter à votre base :",
    "chatbot.category": "Catégorie :",
    "chatbot.item": "Élément :",
    "chatbot.confirm": "Confirmer",
    "chatbot.decline": "Refuser",
    "chatbot.items_added": "Éléments ajoutés au coffre !",
    "chatbot.select_category": "Choisissez une catégorie pour les éléments :",
    "chatbot.create_new": "+ Créer nouvelle",
    "chatbot.cancel": "Annuler",
    "chatbot.model": "Modèle :",
    "chatbot.placeholder": "Posez une question...",
    "chatbot.settings": "Paramètres",

    // Bottom nav
    "nav.picked": "Tirés",
    "nav.vault": "Coffre",
    "nav.home": "Accueil",
    "nav.chatbot": "ChatBot",
    "nav.setting": "Réglages",

    // Settings page
    "settings.title": "Paramètres",
    "settings.documentation": "Documentation",
    "settings.recycle_bin": "Corbeille",
    "settings.bulk_import": "Importer",
    "settings.bulk_export": "Exporter",
    "settings.theme": "Thème",
    "settings.theme_light": "Clair",
    "settings.theme_dark": "Sombre",
    "settings.language": "Langue",
    "settings.device_name": "Nom de l'appareil",
    "settings.manage_user": "Utilisateurs",
    "settings.manage_sync": "Synchronisation",
    "settings.location_services": "Localisation",
    "settings.manage_llm": "Modèle LLM (IA)",
    "settings.save": "Enregistrer",
    "settings.cancel": "Annuler",
    "settings.english": "English",
    "settings.french": "Français",

    // Manage User
    "user.edit": "Modifier",
    "user.edit_name": "Modifier le nom",
    "user.new_name": "Nouveau nom :",
    "user.new_name_ph": "Entrez le nouveau nom...",

    // Recycle bin
    "recycle.vault": "Coffre",
    "recycle.categories": "Catégories",
    "recycle.picked": "Sélections",

    // Export/Import
    "export.select_users": "Utilisateurs :",
    "export.all_users": "Tous",
    "export.select_categories": "Catégories :",
    "export.all_categories": "Toutes",
    "export.select_picks": "Sélections :",
    "export.include_picks": "Inclure les sélections (historique)",
    "export.all_picks": "Toutes",
    "export.format": "Format :",
    "export.export": "Exporter",
    "export.exporting": "Export...",
    "export.result": "Résultat",
    "export.preview": "Aperçu",
    "import.format": "Format :",
    "import.file": "Fichier :",
    "import.paste": "Ou collez le contenu :",
    "import.placeholder": "Collez le contenu JSON/CSV/Markdown...",
    "import.import": "Importer",
    "import.importing": "Import...",

    // LLM settings
    "llm.provider": "Fournisseur :",
    "llm.api_key": "Clé API :",
    "llm.endpoint": "URL :",
    "llm.model": "Modèle :",
    "llm.model_manual": "Modèle (manuel) :",
    "llm.system_prompt": "Prompt système :",
    "llm.allow_create": "Autoriser l'IA à créer des éléments",
    "llm.test": "Tester",
    "llm.not_configured": "Non configuré",
    "llm.openai_label": "OpenAI (GPT)",
    "llm.gemini_label": "Google Gemini",
    "llm.local_label": "LLM Local (Ollama/LM Studio)",
    "llm.api_key_openai": "Clé API OpenAI :",
    "llm.api_key_gemini": "Clé API Gemini :",
    "llm.api_key_ph": "Entrez la clé API...",
    "llm.model_ph": "Choisir un modèle...",
    "llm.endpoint_ph": "http://localhost:11434",

    // Recycle bin actions
    "recycle.restore": "Restaurer",
    "recycle.delete_permanent": "Supprimer définitivement",

    // Missing keys
    "chatbot.error": "Échec de l'ajout",
    "picked.image_too_large": "L'image doit faire moins de 5 Mo",
    "picked.download_image": "Enregistrer dans la galerie",
    "picked.image_saved": "Image enregistrée dans la galerie",
    "picked.image_save_failed": "Échec de l'enregistrement",

    // Location
    "location.permission": "Permission",
    "location.enabled": "Localisation activée",
    "location.requesting": "Demande en cours...",
    "location.allow": "Autoriser la localisation",
    "location.denied":
      "Permission refusée. Activez-la dans les paramètres de l'appareil.",
    "location.roll_dice": "Lieu du lancer",
    "location.description":
      "Choisissez un lieu ou utilisez votre position GPS.",
    "location.remove": "Supprimer",
    "location.using_gps": "Position GPS actuelle (par défaut)",
    "location.search_city": "Chercher une ville...",
    "location.search": "Chercher",

    // Sync
    "sync.description":
      "La synchronisation nécessite le serveur Hazardo. Configurez le conteneur Docker sur votre réseau.",
    "sync.in_development": "En développement",

    // Setup page
    "setup.welcome": "Bienvenue sur Hazardo !",
    "setup.device_desc":
      "Identifiez cet appareil pour activer la synchronisation entre vos appareils.",
    "setup.device_name": "Nom de l'appareil :",
    "setup.device_name_ph": "Entrez le nom...",
    "setup.user_desc":
      "Créez un nom d'utilisateur. Vous pouvez ajouter plusieurs utilisateurs sur cet appareil.",
    "setup.username": "Nom d'utilisateur :",
    "setup.username_ph": "Entrez le nom...",
    "setup.location_desc":
      "Autorisez la localisation pour la météo et les recommandations locales.",
    "setup.location_granted": "Localisation activée",
    "setup.location_denied": "Localisation refusée",
    "setup.location_try_again": "Réessayer",
    "setup.alert_device_name": "Entrez un nom d'appareil",

    // Documentation
    "doc.tap_section": "Appuyez sur une section pour en savoir plus.",
    "doc.home_title": "Accueil (Lancer le dé)",
    "doc.home_desc":
      "La page d'accueil permet de lancer le dé pour obtenir une activité aléatoire.",
    "doc.home_time":
      "Choisissez AM, PM, Nuit ou Mixte pour filtrer par moment de la journée.",
    "doc.home_vibe":
      "Choisissez Ami, Rendez-vous, Famille ou Mixte selon votre humeur.",
    "doc.home_list":
      "Choisissez une catégorie ou laissez sur « Toutes les listes » pour un tirage aléatoire parmi tout.",
    "doc.home_date":
      "Définissez la date de l'activité (aujourd'hui par défaut).",
    "doc.home_advance":
      "Cochez cette case pour obtenir des recommandations IA. Nécessite un fournisseur IA dans Paramètres → Modèle LLM.",
    "doc.home_roll":
      "Appuyez pour obtenir un tirage ! Un modal affiche le résultat avec les détails, la météo et les recommandations IA.",
    "doc.home_pick": "Enregistrez le tirage dans votre historique.",
    "doc.home_roll_again": "Pas satisfait ? Relancez pour un nouveau tirage.",
    "doc.vault_title": "Coffre",
    "doc.vault_desc": "Le coffre permet de gérer vos catégories et éléments.",
    "doc.vault_categories":
      "Organisez vos activités en catégories (ex : Restaurant, Film, Sport). Chaque catégorie a un nom et une icône.",
    "doc.vault_create_cat":
      "Appuyez sur « + » pour ajouter une catégorie. Choisissez un nom et une icône.",
    "doc.vault_add_items":
      "Dans une catégorie, appuyez sur « + » pour ajouter des activités. Chaque élément peut avoir :",
    "doc.vault_item_name": "Le nom de l'activité",
    "doc.vault_item_time": "Le moment idéal (AM/PM/Nuit/Mixte)",
    "doc.vault_item_vibe": "Pour qui (Ami/Rendez-vous/Famille/Mixte)",
    "doc.vault_item_notes": "Des notes optionnelles",
    "doc.vault_edit_delete":
      "Appuyez sur un élément pour le modifier, ou sur la corbeille pour le supprimer.",
    "doc.picked_title": "Sélections",
    "doc.picked_desc": "Consultez l'historique de toutes vos sélections.",
    "doc.picked_filter":
      "Utilisez le filtre et la recherche pour trouver des sélections.",
    "doc.picked_detail":
      "Appuyez sur une sélection pour voir les détails : date, moment, ambiance, catégorie et lieu/météo.",
    "doc.picked_ai":
      "Si le lancer avancé était activé, appuyez sur la section IA pour voir la recommandation.",
    "doc.picked_notes": "Ajoutez des notes personnelles sur l'activité.",
    "doc.picked_image": "Ajoutez une photo pour garder un souvenir.",
    "doc.chatbot_title": "ChatBot",
    "doc.chatbot_desc":
      "Un assistant IA pour planifier vos activités et obtenir des recommandations.",
    "doc.chatbot_requires":
      "Un fournisseur IA doit être configuré dans Paramètres → Modèle LLM (OpenAI, Gemini ou LLM local).",
    "doc.chatbot_chat":
      "Écrivez un message pour demander à l'IA des idées d'activités, de restaurants, de voyages, etc.",
    "doc.chatbot_model":
      "Si plusieurs modèles sont disponibles, utilisez le menu déroulant pour en changer.",
    "doc.chatbot_auto_create":
      "Si activé, l'IA peut proposer de créer des catégories et éléments. Une confirmation est demandée avant tout ajout.",
    "doc.chatbot_clear":
      "Utilisez l'icône corbeille pour effacer l'historique de conversation.",
    "doc.settings_title": "Paramètres",
    "doc.settings_recycle":
      "Voir et restaurer les éléments, catégories ou sélections supprimés.",
    "doc.settings_import":
      "Importer des données depuis un fichier JSON, CSV ou Markdown.",
    "doc.settings_export":
      "Exporter vos données en JSON, CSV, Markdown ou ZIP.",
    "doc.settings_device":
      "Définir un nom pour cet appareil (utilisé pour la synchronisation).",
    "doc.settings_user":
      "Voir, modifier et supprimer les profils utilisateurs.",
    "doc.settings_sync":
      "Configurer la synchronisation avec le serveur Hazardo (Docker requis).",
    "doc.settings_location":
      "Activer le GPS pour des recommandations basées sur la météo. Possibilité de définir une ville manuellement.",
    "doc.settings_llm": "Configurer votre fournisseur IA :",
    "doc.settings_llm_openai":
      "Entrez votre clé API. Supporte les modèles GPT.",
    "doc.settings_llm_gemini":
      "Entrez votre clé API. Supporte les modèles Gemini.",
    "doc.settings_llm_local":
      "Entrez l'URL de votre serveur Ollama/LM Studio local.",
    "doc.settings_llm_prompt":
      "Personnalisez le comportement de l'assistant IA.",
    "doc.settings_llm_auto":
      "Permettre au ChatBot d'ajouter des catégories et éléments (avec confirmation).",

    // AI system prompts
    "ai.system_prompt":
      "Tu es Hazardo, un assistant IA de recommandation d'activités. L'utilisateur vient de faire un tirage aléatoire. Fournis une recommandation utile et concise.",
    "ai.chatbot_default_prompt":
      "Tu es l'assistant IA Hazardo. Aide l'utilisateur à planifier ses activités.",
    "ai.respond_instruction": "Réponds toujours en français.",
    "ai.format_quick_take": "En bref",
    "ai.format_top_picks": "Meilleurs choix près de vous",
    "ai.format_pro_tips": "Conseils",
    "ai.format_what_to_bring": "À apporter",
    "ai.format_alternative_ideas": "Alternatives",
    "ai.format_quick_take_desc":
      "2-3 phrases résumant pourquoi c'est un bon choix selon la météo, le moment et l'ambiance.",
    "ai.format_top_picks_desc":
      "Liste de 3-5 vrais endroits près de l'utilisateur en lien avec l'activité.",
    "ai.format_top_picks_item": "**Nom du lieu** — 1 phrase de description",
    "ai.format_top_picks_links":
      "[Itinéraire](https://www.google.com/maps/dir/?api=1&destination=NOM+DU+LIEU+VILLE) · [Rechercher](https://www.google.com/search?q=NOM+DU+LIEU+VILLE)",
    "ai.format_pro_tips_desc":
      "3-4 conseils pratiques pour l'activité selon la météo et le contexte.",
    "ai.format_what_to_bring_desc": "3-5 choses à apporter ou préparer.",
    "ai.format_alternative_desc":
      "2-3 activités de secours avec une explication courte.",
    "ai.format_rules":
      "RÈGLES :\n- Sois concis mais précis pour la ville de l'utilisateur.\n- Utilise de vrais noms de lieux près de l'utilisateur.\n- Utilise le format markdown (##, gras, puces, liens).\n- Si la météo est mauvaise, privilégie les options intérieures.",
    "ai.user_msg_prefix": "Je viens de tirer :",
    "ai.user_msg_suffix": "Que recommandes-tu ?",
  },
};

export const t = derived(currentLang, ($lang) => {
  return (key: string): string => {
    return translations[$lang]?.[key] || translations["en"]?.[key] || key;
  };
});

export function formatDateLocalized(dateStr: string, lang: Lang): string {
  if (!dateStr) return "";
  try {
    const d = new Date(dateStr);
    const locale = lang === "fr" ? "fr-FR" : "en-US";
    return d.toLocaleDateString(locale, {
      day: "2-digit",
      month: "long",
      year: "numeric",
    });
  } catch {
    return dateStr;
  }
}

export async function loadLanguageSetting(userId: number): Promise<void> {
  try {
    const settings = await invoke<
      { setting_key: string; setting_value: string }[]
    >("get_all_settings", { userId });
    const map = new Map(settings.map((s) => [s.setting_key, s.setting_value]));
    const lang = map.get("app_language") as Lang;
    if (lang === "fr" || lang === "en") {
      currentLang.set(lang);
    }
  } catch (_) {}
}

export async function saveLanguageSetting(
  userId: number,
  lang: Lang,
): Promise<void> {
  currentLang.set(lang);
  try {
    await invoke("set_setting", { userId, key: "app_language", value: lang });
  } catch (_) {}
}
