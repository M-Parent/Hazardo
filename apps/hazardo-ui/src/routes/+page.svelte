<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { marked } from 'marked';
  import { selectedUser } from '../stores/userStore';
  import Title from '../components/atoms/Title.svelte';
  import FormLabel from '../components/atoms/FormLabel.svelte';
  import Icon from '../components/atoms/Icon.svelte';
  import OptionToggle from '../components/molecules/OptionToggle.svelte';
  import SelectDropdown from '../components/molecules/SelectDropdown.svelte';
  import Modal from '../components/organisms/Modal.svelte';
  import type { Category, Pick as PickType, AppSetting } from '$lib/types';

  marked.setOptions({ breaks: true, gfm: true });

  const renderer = new marked.Renderer();
  renderer.link = function ({ href, title, text }: { href: string; title?: string | null | undefined; text: string }) {
    const titleAttr = title ? ` title="${title}"` : '';
    return `<a href="${href}"${titleAttr} target="_blank" rel="noopener noreferrer">${text}</a>`;
  };
  marked.use({ renderer });

  const timeOptions = ['AM', 'PM', 'Night', 'Mixed'];
  const vibeOptions = ['Friend', 'Date', 'Family', 'Mixed'];

  let selectedTime = 'Mixed';
  let selectedVibe = 'Mixed';
  let selectedCategoryId = '';
  let selectedDate = new Date().toISOString().split('T')[0];
  let advanceRoll = false;
  let rolling = false;

  let categories: Category[] = [];
  let categoryOptions: { value: string; label: string; icon?: string }[] = [];

  let showResult = false;
  let rollResult: PickType | null = null;

  // Weather & Location
  let userLat: number | null = null;
  let userLon: number | null = null;
  let locationName = '';
  let weatherTemp: number | null = null;
  let weatherDesc = '';
  let locationError = '';
  let weatherLoading = false;

  // LLM settings
  let llmProvider = '';
  let llmApiKey = '';
  let llmEndpoint = '';
  let llmModel = '';
  let aiLoading = false;
  let aiRecommendation = '';
  let locationEnabled = false;
  let locationOverride = '';

  $: if ($selectedUser) {
    loadCategories($selectedUser.user_id);
    loadLlmSettings($selectedUser.user_id);
    loadLocationSetting($selectedUser.user_id);
  }

  onMount(() => {
    // Intercept link clicks in markdown content to open in external browser
    document.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      const anchor = target.closest('a[href]') as HTMLAnchorElement | null;
      if (anchor && anchor.href && (anchor.href.startsWith('http://') || anchor.href.startsWith('https://'))) {
        e.preventDefault();
        openUrl(anchor.href);
      }
    });
  });

  async function loadLocationSetting(userId: number) {
    try {
      const settings = await invoke<AppSetting[]>('get_all_settings', { userId });
      const map = new Map(settings.map(s => [s.setting_key, s.setting_value]));
      locationEnabled = map.get('location_enabled') === 'true';
      locationOverride = map.get('location_override') || '';

      if (locationOverride) {
        // Use override location: "CityName|lat|lon"
        const parts = locationOverride.split('|');
        if (parts.length >= 3) {
          locationName = parts[0].split(',')[0].trim();
          userLat = parseFloat(parts[1]);
          userLon = parseFloat(parts[2]);
          locationError = '';
          await fetchWeather(userLat, userLon);
        }
      } else if (locationEnabled) {
        requestLocation();
      }
    } catch (_) {
      // setting may not exist yet
    }
  }

  function requestLocation() {
    if (!navigator.geolocation) {
      locationError = 'Geolocation not supported';
      return;
    }
    navigator.geolocation.getCurrentPosition(
      async (pos) => {
        userLat = pos.coords.latitude;
        userLon = pos.coords.longitude;
        locationError = '';
        await fetchWeather(userLat, userLon);
        await reverseGeocode(userLat, userLon);
      },
      (err) => {
        locationError = err.message || 'Location denied';
      },
      { enableHighAccuracy: false, timeout: 10000 }
    );
  }

  async function fetchWeather(lat: number, lon: number) {
    weatherLoading = true;
    try {
      const url = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,weather_code&timezone=auto`;
      const resp = await tauriFetch(url, { method: 'GET' });
      if (!resp.ok) throw new Error(`Weather API returned ${resp.status}`);
      const data = await resp.json();
      weatherTemp = Math.round(data.current?.temperature_2m ?? 0);
      const code = data.current?.weather_code ?? 0;
      weatherDesc = weatherCodeToDesc(code);
    } catch (e) {
      console.error('Weather fetch failed', e);
    }
    weatherLoading = false;
  }

  async function reverseGeocode(lat: number, lon: number) {
    try {
      const url = `https://nominatim.openstreetmap.org/reverse?lat=${lat}&lon=${lon}&format=json&zoom=10`;
      const resp = await tauriFetch(url, {
        method: 'GET',
        headers: { 'User-Agent': 'HazardoApp/1.0' },
      });
      if (!resp.ok) return;
      const data = await resp.json();
      locationName = data.address?.city || data.address?.town || data.address?.village || data.display_name?.split(',')[0] || '';
    } catch (_) {}
  }

  function weatherCodeToDesc(code: number): string {
    if (code === 0) return 'Clear sky';
    if (code <= 3) return 'Partly cloudy';
    if (code <= 48) return 'Fog';
    if (code <= 57) return 'Drizzle';
    if (code <= 65) return 'Rain';
    if (code <= 67) return 'Freezing rain';
    if (code <= 77) return 'Snow';
    if (code <= 82) return 'Rain showers';
    if (code <= 86) return 'Snow showers';
    if (code >= 95) return 'Thunderstorm';
    return 'Cloudy';
  }

  async function loadLlmSettings(userId: number) {
    try {
      const settings = await invoke<AppSetting[]>('get_all_settings', { userId });
      const map = new Map(settings.map(s => [s.setting_key, s.setting_value]));
      llmProvider = map.get('llm_provider') || '';
      llmApiKey = map.get('llm_api_key') || '';
      llmEndpoint = map.get('llm_endpoint') || '';
      llmModel = map.get('llm_model') || '';
    } catch (_) {}
  }

  async function loadCategories(userId: number) {
    try {
      categories = await invoke<Category[]>('get_categories', { userId });
      categoryOptions = [
        { value: '', label: 'All Lists', icon: '' },
        ...categories.map(c => ({ value: String(c.category_id), label: c.category_name, icon: c.category_icon }))
      ];
    } catch (e) {
      console.error('get_categories failed', e);
    }
  }

  async function handleRoll() {
    if (!$selectedUser) return;
    rolling = true;
    aiRecommendation = '';
    try {
      const result = await invoke<PickType | null>('roll_dice', {
        userId: $selectedUser.user_id,
        timePref: selectedTime,
        vibePref: selectedVibe,
        categoryId: selectedCategoryId ? Number(selectedCategoryId) : null,
      });
      rollResult = result;
      showResult = true;

      // If advance roll enabled and LLM configured, get AI recommendation
      if (advanceRoll && result && llmProvider) {
        getAdvanceRecommendation(result);
      }
    } catch (e) {
      console.error('roll_dice failed', e);
    }
    rolling = false;
  }

  async function getAdvanceRecommendation(pick: PickType) {
    aiLoading = true;
    try {
      const weatherInfo = weatherTemp !== null
        ? `Current weather: ${weatherTemp}°C, ${weatherDesc}.`
        : 'Weather data not available.';
      const locationInfo = locationName
        ? `User location: ${locationName} (lat: ${userLat}, lon: ${userLon}).`
        : userLat !== null
          ? `User coordinates: lat ${userLat}, lon ${userLon}.`
          : 'Location not available.';

      const systemPrompt = `You are Hazardo, a smart activity recommendation AI. The user just rolled a random activity pick. You MUST provide a LONG, DETAILED, and COMPREHENSIVE recommendation. Do NOT be brief. Write at least 300 words.

${locationInfo}
${weatherInfo}

You MUST follow this exact format with ALL sections filled out in detail:

## Quick Take
One paragraph (3-4 sentences) summarizing your recommendation and why it's a great choice given the weather, time, and vibe.

## Top Picks Near You
List AT LEAST 5 real, specific places near the user's location relevant to the activity. For EACH place include:
- **Place Name** — A 2-3 sentence description of why it's great, what to expect, price range if relevant
- [Get Directions on Google Maps](https://www.google.com/maps/dir/?api=1&destination=PLACE+NAME+CITY)
- [Search on Google](https://www.google.com/search?q=PLACE+NAME+CITY)

## Pro Tips
Give 4-5 detailed, practical tips based on weather, time of day, vibe, and the specific activity. Each tip should be 1-2 sentences.

## What to Bring
List 3-5 items the user should bring or prepare for this activity, based on weather and context.

## Alternative Ideas
Suggest 3 backup activities in case this one doesn't work out, each with a one-sentence explanation.

CRITICAL RULES:
- You MUST write a LONG and DETAILED response — at least 300 words total.
- If the category is restaurant/food: suggest 5+ specific restaurants with cuisine types, price ranges, and popular dishes.
- If it's outdoor (hiking, cycling, etc.): suggest 5+ parks, trails, or spots with difficulty level and distance.
- If it's a gift: suggest 5+ stores and also 3+ specific gift ideas with price ranges.
- If it's a board game/indoor activity: suggest 5+ cafes, game stores, or venues with specialties.
- If weather is bad (rain, snow, cold below 5°C): prioritize indoor alternatives and explain why.
- ALL place names must be REAL places in or near the user's actual city.
- ALL Google Maps links must use the exact format shown above with + for spaces.
- Use markdown headers (##), bold (**), bullet points (-), and links throughout.
- Do NOT give generic or vague advice. Be hyper-specific to the user's city and situation.
- NEVER cut your response short. Complete ALL sections fully.`;

      const userMsg = `I just rolled: **${pick.item_name}** (category: ${pick.category_name}, time: ${pick.time_pref}, vibe: ${pick.vibe_pref}). What do you recommend?`;

      aiRecommendation = await callLlm(systemPrompt, userMsg);
    } catch (e: any) {
      aiRecommendation = `AI error: ${e.message || e}`;
    }
    aiLoading = false;
  }

  async function callLlm(systemPrompt: string, userMessage: string): Promise<string> {
    const body: Record<string, any> = {
      messages: [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: userMessage },
      ],
      max_tokens: 2048,
    };

    let endpoint = '';
    let apiKey: string | null = null;

    if (llmProvider === 'gemini') {
      endpoint = 'https://generativelanguage.googleapis.com/v1beta/openai/chat/completions';
      apiKey = llmApiKey;
      body.model = llmModel || 'gemini-2.0-flash';
    } else if (llmProvider === 'openai') {
      endpoint = 'https://api.openai.com/v1/chat/completions';
      apiKey = llmApiKey;
      body.model = llmModel || 'gpt-4o-mini';
    } else {
      let base = llmEndpoint || 'http://localhost:11434';
      if (base.endsWith('/')) base = base.slice(0, -1);
      if (!base.includes('/v1/')) base += '/v1/chat/completions';
      endpoint = base;
      body.model = llmModel || undefined;
      apiKey = null;
    }

    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (apiKey) headers['Authorization'] = `Bearer ${apiKey}`;
    if (llmProvider === 'local') headers['Origin'] = 'http://localhost';

    const resp = await tauriFetch(endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });

    if (!resp.ok) {
      let errorDetail = '';
      try { errorDetail = await resp.text(); } catch {}
      throw new Error(`${resp.status} - ${errorDetail.slice(0, 200)}`);
    }

    const data = await resp.json();
    return data.choices?.[0]?.message?.content || 'No response from AI.';
  }

  function sanitizeHtml(html: string): string {
    return html
      .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
      .replace(/on\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]*)/gi, '')
      .replace(/<iframe\b[^>]*>[\s\S]*?<\/iframe>/gi, '')
      .replace(/<object\b[^>]*>[\s\S]*?<\/object>/gi, '')
      .replace(/<embed\b[^>]*>/gi, '');
  }

  function renderMarkdown(content: string): string {
    const html = marked.parse(content) as string;
    return sanitizeHtml(html);
  }

  async function handlePickThis() {
    if (!rollResult || !$selectedUser) return;
    try {
      await invoke('create_pick', {
        userId: $selectedUser.user_id,
        itemId: rollResult.item_id,
        categoryId: rollResult.category_id,
        pickDate: selectedDate,
        timePref: selectedTime,
        vibePref: selectedVibe,
        aiRecommendation: aiRecommendation || rollResult.ai_recommendation,
        location: locationName ? `${locationName}${weatherTemp !== null ? ` (${weatherTemp}°C, ${weatherDesc})` : ''}` : null,
      });
      showResult = false;
      rollResult = null;
      aiRecommendation = '';
    } catch (e) {
      console.error('create_pick failed', e);
    }
  }

  function handleRollAgain() {
    showResult = false;
    rollResult = null;
    aiRecommendation = '';
    handleRoll();
  }
</script>

<main class="max-w-lg mx-auto px-4">
  <div class="flex flex-col items-center">
    <div class="mt-6 mb-8 bg-hazardo-accent px-2 rounded">
      <Title title="Select Criteria" />
    </div>

    <div class="w-full max-w-78 flex flex-col gap-6">
      <!-- Time Preference -->
      <div>
        <FormLabel label="Time Preference:" />
        <div class="mt-2">
          <OptionToggle options={timeOptions} bind:selected={selectedTime} />
        </div>
      </div>

      <!-- Vibe Preference -->
      <div>
        <FormLabel label="Vibe Preference:" />
        <div class="mt-2">
          <OptionToggle options={vibeOptions} bind:selected={selectedVibe} />
        </div>
      </div>

      <!-- List Preference -->
      <div>
        <FormLabel label="List Preference:" />
        <div class="mt-2">
          <SelectDropdown
            options={categoryOptions}
            bind:selected={selectedCategoryId}
            placeholder=""
          />
        </div>
      </div>

      <!-- Date Preference -->
      <div>
        <FormLabel label="Date Preference:" />
        <div class="relative mt-2">
          <input
            type="date"
            bind:value={selectedDate}
            class="w-full border rounded p-2 pr-8 border-hazardo-lightGray focus:outline-hazardo-accent bg-white"
          />
        </div>
      </div>

      <!-- Advance Roll -->
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={advanceRoll} class="w-4 h-4 accent-hazardo-accent" />
        Advance Roll (need AI setting to work)
      </label>

      <!-- Roll Dice -->
      <div class="flex justify-center mt-4">
        <button
          class="bg-hazardo-primary text-white title-font shadow-purple-700 py-2 px-8 rounded disabled:opacity-50"
          on:click={handleRoll}
          disabled={rolling || !$selectedUser}
        >
          {rolling ? 'Rolling...' : 'Roll Dice'}
        </button>
      </div>
    </div>
  </div>
</main>

<!-- Roll Result Modal -->
<Modal bind:show={showResult} title="Hazardo Pick" width="w-96">
  {#if rollResult}
    <div class="flex flex-col gap-4">
      <p class="text-sm text-hazardo-text">Date: {selectedDate}</p>

      <div class="grid grid-cols-2 gap-3">
        <div class="border border-hazardo-lightGray rounded p-3 text-center">
          <div class="flex items-center justify-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="clock" size={14} /> Time
          </div>
          <p class="font-semibold">{rollResult.time_pref}</p>
        </div>
        <div class="border border-hazardo-lightGray rounded p-3 text-center">
          <div class="flex items-center justify-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="users" size={14} /> Vibe
          </div>
          <p class="font-semibold">{rollResult.vibe_pref}</p>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="border border-hazardo-lightGray rounded p-3 text-center">
          <div class="flex items-center justify-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name={rollResult.category_icon} size={14} /> {rollResult.category_name}
          </div>
          <p class="font-semibold">{rollResult.item_name}</p>
        </div>
        <div class="border border-hazardo-lightGray rounded p-3 text-center">
          <div class="flex items-center justify-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="map-pin" size={14} /> {locationName || 'Location'}
          </div>
          {#if locationName}
            <p class="font-semibold">{weatherTemp !== null ? `${weatherTemp}°C` : ''}</p>
            <p class="text-xs text-hazardo-lightGray">{weatherDesc || ''}</p>
          {:else if weatherLoading}
            <p class="text-xs text-hazardo-lightGray">Loading...</p>
          {:else if !locationEnabled}
            <p class="text-xs text-hazardo-lightGray">Disabled</p>
          {:else}
            <p class="text-xs text-hazardo-lightGray">N/A</p>
          {/if}
        </div>
      </div>

      {#if aiLoading}
        <div class="border border-hazardo-lightGray rounded p-3">
          <div class="flex items-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="chatbot" size={14} /> AI Recommendation
          </div>
          <p class="text-sm text-hazardo-lightGray animate-pulse">Thinking...</p>
        </div>
      {:else if aiRecommendation}
        <div class="border border-hazardo-lightGray rounded p-3">
          <div class="flex items-center gap-1 text-sm text-hazardo-text mb-2">
            <Icon name="chatbot" size={14} /> AI Recommendation
          </div>
          <div class="text-sm markdown-content max-h-72 overflow-y-auto">{@html renderMarkdown(aiRecommendation)}</div>
        </div>
      {:else if rollResult.ai_recommendation}
        <div class="border border-hazardo-lightGray rounded p-3">
          <div class="flex items-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="chatbot" size={14} /> AI Recommendation
          </div>
          <p class="text-sm">{rollResult.ai_recommendation}</p>
        </div>
      {/if}

      <div class="flex items-center justify-center gap-4 mt-2">
        <button class="text-sm text-hazardo-text hover:text-hazardo-primary" on:click={handleRollAgain}>Roll Again</button>
        <button class="bg-hazardo-primary text-white py-2 px-6 rounded title-font text-sm" on:click={handlePickThis}>Pick this!</button>
      </div>
      <button class="text-sm text-center text-hazardo-lightGray hover:text-hazardo-text" on:click={() => { showResult = false; rollResult = null; aiRecommendation = ''; }}>Cancel</button>
    </div>
  {:else}
    <div class="text-center py-8">
      <p class="text-hazardo-text">No items found matching your criteria.</p>
      <p class="text-sm text-hazardo-lightGray mt-2">Add items in the Vault first!</p>
      <button class="mt-4 text-sm text-hazardo-accent" on:click={() => showResult = false}>Close</button>
    </div>
  {/if}
</Modal>

<style>
  :global(.markdown-content h1) { font-size: 1.25rem; font-weight: 700; margin: 0.5rem 0; }
  :global(.markdown-content h2) { font-size: 1.1rem; font-weight: 600; margin: 0.4rem 0; }
  :global(.markdown-content h3) { font-size: 1rem; font-weight: 600; margin: 0.3rem 0; }
  :global(.markdown-content p) { margin: 0.3rem 0; }
  :global(.markdown-content ul),
  :global(.markdown-content ol) { padding-left: 1.25rem; margin: 0.3rem 0; }
  :global(.markdown-content li) { margin: 0.15rem 0; }
  :global(.markdown-content code) { background: #f3f4f6; padding: 0.1rem 0.3rem; border-radius: 0.25rem; font-size: 0.85em; }
  :global(.markdown-content pre) { background: #f3f4f6; padding: 0.5rem; border-radius: 0.375rem; overflow-x: auto; margin: 0.3rem 0; }
  :global(.markdown-content strong) { font-weight: 600; }
  :global(.markdown-content a) { color: #7c3aed; text-decoration: underline; }
</style>
