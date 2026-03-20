<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
  import { handleMarkdownClick } from '$lib/markdown';
  import { selectedUser } from '../stores/userStore';
  import { t, currentLang, formatDateLocalized } from '../stores/i18nStore';
  import { get } from 'svelte/store';
  import { renderMarkdown } from '$lib/markdown';
  import { TIME_OPTIONS, VIBE_OPTIONS } from '$lib/constants';
  import Title from '../components/atoms/Title.svelte';
  import FormLabel from '../components/atoms/FormLabel.svelte';
  import Icon from '../components/atoms/Icon.svelte';
  import OptionToggle from '../components/molecules/OptionToggle.svelte';
  import SelectDropdown from '../components/molecules/SelectDropdown.svelte';
  import Modal from '../components/organisms/Modal.svelte';
  import { showToast } from '../stores/toastStore';
  import type { Category, Pick as PickType, AppSetting } from '$lib/types';

  // marked is configured globally by $lib/markdown

  const timeOptions = [...TIME_OPTIONS];
  const vibeOptions = [...VIBE_OPTIONS];

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

  // Persist modal state across app background/resume
  function saveModalState() {
    if (showResult && rollResult) {
      sessionStorage.setItem('hazardo_pick_modal', JSON.stringify({ rollResult, aiRecommendation }));
    } else {
      sessionStorage.removeItem('hazardo_pick_modal');
    }
  }

  function restoreModalState() {
    try {
      const saved = sessionStorage.getItem('hazardo_pick_modal');
      if (saved) {
        const data = JSON.parse(saved);
        rollResult = data.rollResult;
        aiRecommendation = data.aiRecommendation || '';
        showResult = true;
      }
    } catch (_) {}
  }

  $: if (showResult || rollResult || aiRecommendation) saveModalState();

  // Only clear sessionStorage when user explicitly dismisses the modal
  // (not on initial component mount where showResult starts as false)
  let modalWasShown = false;
  $: if (showResult) modalWasShown = true;
  $: if (!showResult && modalWasShown) {
    sessionStorage.removeItem('hazardo_pick_modal');
    modalWasShown = false;
  }

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
  let aiRequestId = 0;
  let locationEnabled = true;
  let locationOverride = '';

  $: if ($selectedUser) {
    loadCategories($selectedUser.user_id);
    loadLlmSettings($selectedUser.user_id);
    loadLocationSetting($selectedUser.user_id);
  }

  onMount(() => {
    // Restore modal state if app was in background
    restoreModalState();

    // Restore modal when returning from another app (e.g. Google Maps)
    const handleVisibility = () => {
      if (document.visibilityState === 'visible' && !showResult) {
        restoreModalState();
      }
    };
    document.addEventListener('visibilitychange', handleVisibility);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibility);
    };
  });

  async function loadLocationSetting(userId: number) {
    try {
      const settings = await invoke<AppSetting[]>('get_all_settings', { userId });
      const map = new Map(settings.map(s => [s.setting_key, s.setting_value]));
      const locationSetting = map.get('location_enabled');
      // Default to true (use GPS) if the setting hasn't been configured yet
      locationEnabled = locationSetting !== 'false';
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
      // setting may not exist yet — use GPS by default
      locationEnabled = true;
      requestLocation();
    }
  }

  function requestLocation() {
    if (!navigator.geolocation) {
      fallbackIpLocation();
      return;
    }
    // Try fast cached position first, then accurate position
    navigator.geolocation.getCurrentPosition(
      async (pos) => {
        userLat = pos.coords.latitude;
        userLon = pos.coords.longitude;
        locationError = '';
        await reverseGeocode(userLat, userLon);
        await fetchWeather(userLat, userLon);
      },
      () => {
        // GPS failed or denied — try IP-based fallback
        fallbackIpLocation();
      },
      { enableHighAccuracy: false, timeout: 3000, maximumAge: 300000 }
    );
  }

  async function fallbackIpLocation() {
    try {
      const resp = await tauriFetch('https://ipwho.is/', { method: 'GET' });
      if (!resp.ok) return;
      const data = await resp.json();
      if (data.success !== false && data.latitude && data.longitude) {
        userLat = data.latitude;
        userLon = data.longitude;
        locationName = data.city || data.region || '';
        locationError = '';
        await fetchWeather(data.latitude, data.longitude);
      }
    } catch (_) {
      locationError = 'Location unavailable';
    }
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
      updateCategoryOptions();
    } catch (e) {
      console.error('get_categories failed', e);
    }
  }

  function updateCategoryOptions() {
    categoryOptions = [
      { value: '', label: $t('home.all_lists'), icon: '' },
      ...categories.map(c => ({ value: String(c.category_id), label: c.category_name, icon: c.category_icon }))
    ];
  }

  $: if ($currentLang && categories) updateCategoryOptions();

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
      if (!result) {
        showToast($t('home.no_items'), 'error');
        rolling = false;
        return;
      }
      showResult = true;

      // If advance roll enabled and LLM configured, get AI recommendation
      if (advanceRoll && result && llmProvider) {
        getAdvanceRecommendation(result);
      }
    } catch (e: any) {
      console.error('roll_dice failed', e);
      const msg = typeof e === 'string' ? e : e?.message || $t('home.roll_error');
      showToast(msg, 'error');
    }
    rolling = false;
  }

  async function getAdvanceRecommendation(pick: PickType) {
    const currentRequestId = ++aiRequestId;
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

      const langInstruction = $t('ai.respond_instruction');

      const tr = get(t);

      const systemPrompt = `${tr('ai.system_prompt')}

${locationInfo}
${weatherInfo}

${langInstruction}

Use this format:

## ${tr('ai.format_quick_take')}
${tr('ai.format_quick_take_desc')}

## ${tr('ai.format_top_picks')}
${tr('ai.format_top_picks_desc')}
- ${tr('ai.format_top_picks_item')}
- ${tr('ai.format_top_picks_links')}

## ${tr('ai.format_pro_tips')}
${tr('ai.format_pro_tips_desc')}

## ${tr('ai.format_what_to_bring')}
${tr('ai.format_what_to_bring_desc')}

## ${tr('ai.format_alternative_ideas')}
${tr('ai.format_alternative_desc')}

${tr('ai.format_rules')}`;

      const userMsg = `${tr('ai.user_msg_prefix')} **${pick.item_name}** (category: ${pick.category_name}, time: ${pick.time_pref}, vibe: ${pick.vibe_pref}). ${tr('ai.user_msg_suffix')}`;

      aiRecommendation = await callLlm(systemPrompt, userMsg);
    } catch (e: any) {
      if (currentRequestId !== aiRequestId) return;
      aiRecommendation = `AI error: ${e.message || e}`;
    }
    if (currentRequestId !== aiRequestId) return;
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
    aiRequestId++;
    showResult = false;
    rollResult = null;
    aiRecommendation = '';
    aiLoading = false;
    handleRoll();
  }
</script>

<main class="max-w-lg mx-auto px-4">
  <div class="flex flex-col items-center">
    <div class="mt-6 mb-8 bg-hazardo-accent px-2 rounded">
      <Title title={$t('home.title')} />
    </div>

    <div class="w-full max-w-78 flex flex-col gap-6">
      <!-- Time Preference -->
      <div>
        <FormLabel label={$t('home.time_pref')} />
        <div class="mt-2">
          <OptionToggle options={timeOptions} bind:selected={selectedTime} />
        </div>
      </div>

      <!-- Vibe Preference -->
      <div>
        <FormLabel label={$t('home.vibe_pref')} />
        <div class="mt-2">
          <OptionToggle options={vibeOptions} bind:selected={selectedVibe} />
        </div>
      </div>

      <!-- List Preference -->
      <div>
        <FormLabel label={$t('home.list_pref')} />
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
        <FormLabel label={$t('home.date_pref')} />
        <div class="relative mt-2">
          <input
            type="date"
            bind:value={selectedDate}
            class="w-full border rounded p-2 pr-8 border-hazardo-lightGray focus:outline-hazardo-accent bg-hazardo-surface"
          />
        </div>
      </div>

      <!-- Advance Roll -->
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={advanceRoll} class="w-4 h-4 accent-hazardo-accent" />
        {$t('home.advance_roll')}
      </label>

      <!-- Roll Dice -->
      <div class="flex justify-center mt-4">
        <button
          class="bg-hazardo-primary text-white title-font shadow-purple-700 py-2 px-8 rounded disabled:opacity-50"
          on:click={handleRoll}
          disabled={rolling || !$selectedUser}
        >
          {rolling ? $t('home.rolling') : $t('home.roll_dice')}
        </button>
      </div>
    </div>
  </div>
</main>

<!-- Roll Result Modal -->
<Modal bind:show={showResult} title={$t('home.hazardo_pick')} width="w-96">
  {#if rollResult}
    <div class="flex flex-col gap-4">
      <p class="text-sm text-hazardo-text">{$t('home.date')} {formatDateLocalized(selectedDate, $currentLang)}</p>

      <div class="grid grid-cols-2 gap-3">
        <div class="border border-hazardo-lightGray rounded p-3 text-center">
          <div class="flex items-center justify-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="clock" size={14} /> {$t('home.time')}
          </div>
          <p class="font-semibold">{rollResult.time_pref}</p>
        </div>
        <div class="border border-hazardo-lightGray rounded p-3 text-center">
          <div class="flex items-center justify-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="users" size={14} /> {$t('home.vibe')}
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
            <Icon name="map-pin" size={14} /> {locationName || $t('home.location')}
          </div>
          {#if locationName}
            <p class="font-semibold">{weatherTemp !== null ? `${weatherTemp}°C` : ''}</p>
            <p class="text-xs text-hazardo-lightGray">{weatherDesc || ''}</p>
          {:else if weatherLoading}
            <p class="text-xs text-hazardo-lightGray">{$t('home.loading')}</p>
          {:else if !locationEnabled}
            <p class="text-xs text-hazardo-lightGray">{$t('home.disabled')}</p>
          {:else}
            <p class="text-xs text-hazardo-lightGray">{$t('home.na')}</p>
          {/if}
        </div>
      </div>

      {#if aiLoading}
        <div class="border border-hazardo-lightGray rounded p-3">
          <div class="flex items-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="chatbot" size={14} /> {$t('home.ai_recommendation')}
          </div>
          <p class="text-sm text-hazardo-lightGray animate-pulse">{$t('home.thinking')}</p>
        </div>
      {:else if aiRecommendation}
        <div class="border border-hazardo-lightGray rounded p-3">
          <div class="flex items-center gap-1 text-sm text-hazardo-text mb-2">
            <Icon name="chatbot" size={14} /> {$t('home.ai_recommendation')}
          </div>
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div class="text-sm markdown-content max-h-72 overflow-y-auto" on:click={handleMarkdownClick}>{@html renderMarkdown(aiRecommendation)}</div>
        </div>
      {:else if rollResult.ai_recommendation}
        <div class="border border-hazardo-lightGray rounded p-3">
          <div class="flex items-center gap-1 text-sm text-hazardo-text mb-1">
            <Icon name="chatbot" size={14} /> {$t('home.ai_recommendation')}
          </div>
          <p class="text-sm">{rollResult.ai_recommendation}</p>
        </div>
      {/if}

      <div class="flex items-center justify-center gap-4 mt-2">
        <button class="text-sm text-hazardo-text hover:text-hazardo-primary" on:click={handleRollAgain}>{$t('home.roll_again')}</button>
        <button class="bg-hazardo-primary text-white py-2 px-6 rounded title-font text-sm" on:click={handlePickThis}>{$t('home.pick_this')}</button>
      </div>
    </div>
  {:else}
    <div class="text-center py-8">
      <p class="text-hazardo-text">{$t('home.no_items')}</p>
      <p class="text-sm text-hazardo-lightGray mt-2">{$t('home.add_items_first')}</p>
      <button class="mt-4 text-sm text-hazardo-accent" on:click={() => showResult = false}>{$t('home.close')}</button>
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
  :global(.markdown-content code) { background: var(--color-hazardo-background); padding: 0.1rem 0.3rem; border-radius: 0.25rem; font-size: 0.85em; }
  :global(.markdown-content pre) { background: var(--color-hazardo-background); padding: 0.5rem; border-radius: 0.375rem; overflow-x: auto; margin: 0.3rem 0; }
  :global(.markdown-content strong) { font-weight: 600; }
  :global(.markdown-content a) { color: var(--color-hazardo-accent); text-decoration: underline; }
</style>
