<script lang="ts">
  import { tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
  import { handleMarkdownClick, renderMarkdown } from '$lib/markdown';
  import { selectedUser } from '../../stores/userStore';
  import { showToast } from '../../stores/toastStore';
  import { t, currentLang } from '../../stores/i18nStore';
  import Icon from '../../components/atoms/Icon.svelte';
  import SelectDropdown from '../../components/molecules/SelectDropdown.svelte';
  import type { ChatMessage, AppSetting, Category } from '$lib/types';

  // marked is configured globally by $lib/markdown

  let messages: ChatMessage[] = [];
  let inputText = '';
  let chatContainer: HTMLElement;
  let sending = false;
  let llmConfigured = false;
  let llmProvider = '';
  let llmApiKey = '';
  let llmEndpoint = '';
  let llmModel = '';
  let promptFormat = '';
  let chatAutoCreate = 'false';
  let userCategories: Category[] = [];
  let availableModels: { value: string; label: string }[] = [];

  let pendingActions: any[] = [];
  let pendingResponse = '';

  $: if ($selectedUser) {
    loadChat($selectedUser.user_id);
    loadLlmSettings($selectedUser.user_id);
    loadCategories($selectedUser.user_id);
  }



  async function loadChat(userId: number) {
    try {
      messages = await invoke<ChatMessage[]>('get_chat_messages', { userId });
      await tick();
      scrollToBottom();
      // Ensure scroll after full render with multiple fallbacks
      requestAnimationFrame(() => scrollToBottom());
      setTimeout(() => scrollToBottom(), 100);
      setTimeout(() => scrollToBottom(), 300);
    } catch (e) {
      console.error('get_chat_messages failed', e);
    }
  }

  async function loadLlmSettings(userId: number) {
    try {
      const settings = await invoke<AppSetting[]>('get_all_settings', { userId });
      const map = new Map(settings.map(s => [s.setting_key, s.setting_value]));
      llmProvider = map.get('llm_provider') || '';
      llmApiKey = map.get('llm_api_key') || '';
      llmEndpoint = map.get('llm_endpoint') || '';
      llmModel = map.get('llm_model') || '';
      promptFormat = map.get('chat_prompt_format') || 'You are Hazardo AI assistant. Help the user with activity planning and recommendations.';
      chatAutoCreate = map.get('chat_auto_create') || 'false';
      llmConfigured = !!(llmProvider && (llmApiKey || llmEndpoint));
      // Load available models for current provider
      await loadAvailableModels();
    } catch (e) {
      console.error('load settings failed', e);
    }
  }

  async function loadAvailableModels() {
    availableModels = [];
    try {
      if (llmProvider === 'local' && llmEndpoint) {
        let base = llmEndpoint.trim().replace(/\/+$/, '');
        const resp = await tauriFetch(`${base}/api/tags`, { method: 'GET' });
        if (resp.ok) {
          const data = await resp.json();
          availableModels = ((data.models || []) as { name: string }[]).map(m => ({ value: m.name, label: m.name }));
        }
      } else if (llmProvider === 'openai' && llmApiKey) {
        const resp = await tauriFetch('https://api.openai.com/v1/models', {
          method: 'GET',
          headers: { 'Authorization': `Bearer ${llmApiKey}` },
        });
        if (resp.ok) {
          const data = await resp.json();
          availableModels = ((data.data || []) as { id: string }[])
            .filter(m => m.id.startsWith('gpt-') || m.id.startsWith('o1') || m.id.startsWith('o3') || m.id.startsWith('chatgpt'))
            .sort((a, b) => a.id.localeCompare(b.id))
            .map(m => ({ value: m.id, label: m.id }));
        }
      } else if (llmProvider === 'gemini' && llmApiKey) {
        const resp = await tauriFetch(`https://generativelanguage.googleapis.com/v1beta/models?key=${llmApiKey}`, { method: 'GET' });
        if (resp.ok) {
          const data = await resp.json();
          availableModels = ((data.models || []) as any[])
            .filter(m => m.supportedGenerationMethods?.includes('generateContent'))
            .map(m => ({ value: m.name.replace('models/', ''), label: m.displayName || m.name.replace('models/', '') }));
        }
      }
    } catch (_) {}
    // Always add current model if set
    if (llmModel && !availableModels.find(m => m.value === llmModel)) {
      availableModels = [{ value: llmModel, label: llmModel }, ...availableModels];
    }
  }

  async function loadCategories(userId: number) {
    try {
      userCategories = await invoke<Category[]>('get_categories', { userId });
    } catch (e) {
      console.error('load categories failed', e);
    }
  }

  function buildSystemPrompt(): string {
    let prompt = promptFormat;
    prompt += '\n\nAlways format your responses in clean markdown. Use headers, bullet points, bold text, code blocks, and tables when appropriate for readability.';
    if (userCategories.length > 0) {
      prompt += `\n\nThe user's current activity categories:\n`;
      for (const cat of userCategories) {
        prompt += `- ${cat.category_name} (icon: ${cat.category_icon})\n`;
      }
    }
    prompt += `\n\n${$t('ai.respond_instruction')}`;
    return prompt;
  }

  function buildToolInstructions(): string {
    const existingNames = userCategories.map(c => c.category_name);
    const catList = existingNames.length > 0 ? `\nExisting categories: ${existingNames.join(', ')}` : '';
    return `CRITICAL — DATABASE ACTION INSTRUCTIONS (you MUST follow these):
You can create categories and items in the user's Hazardo vault using special tags.

RULES:
1. When the user asks to add, create, suggest, or generate items or categories, you MUST include the EXACT tags below. Without the tags, NOTHING gets saved.
2. If the user asks to "add items" without specifying which, YOU MUST generate creative random items that fit the category or context.
3. ALWAYS include the tags when the user asks to add anything — no matter how long the conversation has been going.
4. You can include multiple tags in one response.
5. NEVER translate the JSON keys. NEVER wrap the tags in markdown code blocks or backticks. Write them as plain text.
6. Always accompany the tags with a short natural language summary listing what you're adding.
7. At the end of normal conversations about activities, restaurants, travel, movies, etc., suggest: "Would you like me to add these to your vault?" — and if they say yes, include the tags.
8. For normal conversation (not about adding items), respond naturally WITHOUT any tags.
9. IMPORTANT: Before creating a new category, check the existing categories listed below. If a matching or similar category already exists, use its EXACT name in the "category" field instead of creating a new one. Only create a new category if nothing similar exists.${catList}

Available icons: activity, restaurant, board-game, valentine, movie, gift, workout, travel, cooking, misc
Time preferences: AM, PM, Night, Mixed
Vibe preferences: Friend, Date, Family, Mixed

Tag format for creating a category:
[HAZARDO_CREATE_CATEGORY]{"name": "Category Name", "icon": "icon-name"}[/HAZARDO_CREATE_CATEGORY]

Tag format for creating an item:
[HAZARDO_CREATE_ITEM]{"category": "Existing Category Name", "name": "Item Name", "time_pref": "Mixed", "vibe_pref": "Mixed", "notes": "optional"}[/HAZARDO_CREATE_ITEM]

EXAMPLE — user says "add some restaurant ideas":
Here are some restaurant ideas I'm adding to your vault! 🎲

- **Sushi Night** — Perfect for a date evening
- **Sunday Brunch** — Great with friends

[HAZARDO_CREATE_ITEM]{"category": "Restaurants", "name": "Sushi Night", "time_pref": "PM", "vibe_pref": "Date", "notes": "Try the omakase"}[/HAZARDO_CREATE_ITEM]
[HAZARDO_CREATE_ITEM]{"category": "Restaurants", "name": "Sunday Brunch", "time_pref": "AM", "vibe_pref": "Friend", "notes": ""}[/HAZARDO_CREATE_ITEM]`;
  }

  function parseActions(response: string): { text: string; actions: any[] } {
    const actions: any[] = [];
    // First, unwrap any code blocks that contain the tags
    let text = response.replace(/```[\s\S]*?```/g, (block) => {
      // If the code block contains HAZARDO tags, unwrap it
      if (block.includes('[HAZARDO_CREATE_') || block.includes('[/HAZARDO_CREATE_')) {
        return block.replace(/^```[^\n]*\n?/, '').replace(/\n?```$/, '');
      }
      return block;
    });
    const catRegex = /\[HAZARDO_CREATE_CATEGORY\]\s*([\s\S]*?)\s*\[\/HAZARDO_CREATE_CATEGORY\]/g;
    text = text.replace(catRegex, (_, json) => {
      try { actions.push({ type: 'create_category', ...JSON.parse(json.trim()) }); } catch {}
      return '';
    });
    const itemRegex = /\[HAZARDO_CREATE_ITEM\]\s*([\s\S]*?)\s*\[\/HAZARDO_CREATE_ITEM\]/g;
    text = text.replace(itemRegex, (_, json) => {
      try { actions.push({ type: 'create_item', ...JSON.parse(json.trim()) }); } catch {}
      return '';
    });
    return { text: text.trim(), actions };
  }

  function normalizeForMatch(s: string): string {
    return s.trim().toLowerCase()
      .normalize('NFD').replace(/[\u0300-\u036f]/g, '')
      .replace(/s$/, '');
  }

  function findMatchingCategory(cats: Category[], name: string): Category | undefined {
    const target = name.toLowerCase().trim();
    // Exact case-insensitive match
    let match = cats.find(c => c.category_name.toLowerCase() === target);
    if (match) return match;

    // Normalized match (strip accents + trailing plural 's')
    const normTarget = normalizeForMatch(name);
    match = cats.find(c => normalizeForMatch(c.category_name) === normTarget);
    if (match) return match;

    // Substring/contains match (one contains the other)
    match = cats.find(c => {
      const normCat = normalizeForMatch(c.category_name);
      return normCat.includes(normTarget) || normTarget.includes(normCat);
    });
    return match;
  }

  async function executeActions(actions: any[], overrideCategory?: Category) {
    if (!$selectedUser) return;
    let createdCount = 0;
    let errorCount = 0;

    for (const action of actions) {
      try {
        if (action.type === 'create_category') {
          // Skip category creation when user picked an existing category
          if (overrideCategory) continue;
          // Skip if a similar category already exists
          const cats = await invoke<Category[]>('get_categories', { userId: $selectedUser.user_id });
          const existing = findMatchingCategory(cats, action.name);
          if (!existing) {
            await invoke('create_category', {
              userId: $selectedUser.user_id,
              categoryName: action.name,
              categoryIcon: action.icon || 'misc',
            });
            createdCount++;
          }
        } else if (action.type === 'create_item') {
          let cats = await invoke<Category[]>('get_categories', { userId: $selectedUser.user_id });
          let cat: Category | undefined;

          if (overrideCategory) {
            // User chose a specific category — use it
            cat = cats.find(c => c.category_id === overrideCategory.category_id);
          } else {
            // Fuzzy match or auto-create
            cat = findMatchingCategory(cats, action.category || '');
            if (!cat && action.category) {
              await invoke('create_category', {
                userId: $selectedUser.user_id,
                categoryName: action.category,
                categoryIcon: action.icon || 'misc',
              });
              cats = await invoke<Category[]>('get_categories', { userId: $selectedUser.user_id });
              cat = findMatchingCategory(cats, action.category);
            }
          }

          if (cat) {
            await invoke('create_item', {
              userId: $selectedUser.user_id,
              categoryId: cat.category_id,
              itemName: action.name,
              timePref: action.time_pref || 'Mixed',
              vibePref: action.vibe_pref || 'Mixed',
              notes: action.notes || null,
              location: null,
              description: null,
            });
            createdCount++;
          } else {
            errorCount++;
          }
        }
      } catch (e) {
        console.error('execute action failed', action, e);
        errorCount++;
      }
    }
    if ($selectedUser) await loadCategories($selectedUser.user_id);

    if (createdCount > 0 && errorCount === 0) {
      showToast($t('chatbot.items_added'), 'success');
    } else if (createdCount > 0 && errorCount > 0) {
      showToast(`${createdCount} added, ${errorCount} failed`, 'error');
    } else {
      showToast($t('chatbot.error') || 'Failed to add items', 'error');
    }
  }

  function scrollToBottom() {
    if (chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  }

  async function sendMessage() {
    if (!inputText.trim() || !$selectedUser || !llmConfigured || sending) return;

    const userText = inputText.trim();
    inputText = '';
    sending = true;

    try {
      const userMsg = await invoke<ChatMessage>('save_chat_message', {
        userId: $selectedUser.user_id,
        role: 'user',
        content: userText,
      });
      messages = [...messages, userMsg];
      await tick();
      scrollToBottom();

      const aiResponse = await callLlm(userText);

      // Always parse actions from AI response
      const { text: cleanResponse, actions } = parseActions(aiResponse);

      const assistantMsg = await invoke<ChatMessage>('save_chat_message', {
        userId: $selectedUser.user_id,
        role: 'assistant',
        content: cleanResponse,
      });
      messages = [...messages, assistantMsg];

      if (actions.length > 0) {
        pendingActions = actions;
        pendingResponse = cleanResponse;
        await tick();
        scrollToBottom();
      }
      await tick();
      scrollToBottom();
    } catch (e) {
      console.error('send message failed', e);
      try {
        const errMsg = await invoke<ChatMessage>('save_chat_message', {
          userId: $selectedUser!.user_id,
          role: 'assistant',
          content: `Sorry, there was an error: ${e instanceof Error ? e.message : e}`,
        });
        messages = [...messages, errMsg];
      } catch (_) { /* silent */ }
    }
    sending = false;
  }

  async function callLlm(userMessage: string): Promise<string> {
    const systemPrompt = buildSystemPrompt();
    const toolInstructions = buildToolInstructions();

    // Detect if user is asking to add/create items for a stronger reminder
    const isAddRequest = /\b(add|ajoute|créé|crée|create|génère|genere|generate|suggest|met|mets|random)\b/i.test(userMessage);
    const reminder = isAddRequest
      ? 'The user is asking you to ADD or CREATE items RIGHT NOW. You MUST include [HAZARDO_CREATE_ITEM] and/or [HAZARDO_CREATE_CATEGORY] tags in your response. Generate items if none were specified. Do NOT just describe them — include the actual tags.'
      : '';

    const body: Record<string, any> = {
      messages: [
        { role: 'system', content: systemPrompt },
        ...messages.slice(-20).map(m => ({ role: m.role, content: m.content })),
        // Always inject tool instructions close to the user message so the LLM never forgets them
        { role: 'system', content: toolInstructions + (reminder ? '\n\n' + reminder : '') },
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
    if (apiKey) {
      headers['Authorization'] = `Bearer ${apiKey}`;
    }
    if (llmProvider === 'local') {
      headers['Origin'] = 'http://localhost';
    }

    const resp = await tauriFetch(endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });

    if (!resp.ok) {
      let errorDetail = '';
      try { errorDetail = await resp.text(); } catch {}
      const parsed = (() => { try { return JSON.parse(errorDetail); } catch { return null; } })();
      const msg = parsed?.error?.message || errorDetail.slice(0, 300) || 'Unknown error';
      throw new Error(`${resp.status} - ${msg}`);
    }

    const data = await resp.json();
    return data.choices?.[0]?.message?.content || 'No response from AI.';
  }

  async function clearHistory() {
    if (!$selectedUser) return;
    try {
      await invoke('clear_chat', { userId: $selectedUser.user_id });
      messages = [];
    } catch (e) {
      console.error('clear_chat failed', e);
    }
  }

  async function saveModelSelection() {
    if (!$selectedUser) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_setting', { userId: $selectedUser.user_id, key: 'llm_model', value: llmModel });
    } catch (_) {}
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }
</script>

<main class="max-w-lg mx-auto px-4 flex flex-col h-[calc(100vh-8rem)]">
  {#if !llmConfigured}
    <div class="flex-1 flex flex-col items-center justify-center text-center gap-4">
      <Icon name="wifi-off" size={48} />
      <p class="text-hazardo-text font-semibold">{$t('chatbot.needs_config')}</p>
      <p class="text-sm text-hazardo-lightGray max-w-xs">
        {$t('chatbot.config_msg').split('Settings')[0]}<a href="/setting" class="text-hazardo-accent underline">{$t('chatbot.settings')}</a>{$t('chatbot.config_msg').split('Settings').slice(1).join('Settings')}
      </p>
    </div>
  {:else}
    <!-- Chat messages -->
    <div class="flex-1 overflow-y-auto py-4 flex flex-col gap-3" bind:this={chatContainer}>
      {#if messages.length === 0}
        <div class="flex-1 flex items-center justify-center">
          <p class="text-sm text-hazardo-lightGray">{$t('chatbot.start_convo')}</p>
        </div>
      {:else}
        {#each messages as msg}
          <div class="flex {msg.role === 'user' ? 'justify-end' : 'justify-start'}">
            <div class="max-w-[80%] rounded-lg px-3 py-2 text-sm {msg.role === 'user' ? 'bg-hazardo-accent text-white' : 'bg-hazardo-surface border border-hazardo-lightGray text-hazardo-text'}">
              {#if msg.role === 'assistant'}
                <!-- svelte-ignore a11y-click-events-have-key-events -->
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                <div class="markdown-content" on:click={handleMarkdownClick}>{@html renderMarkdown(msg.content)}</div>
              {:else}
                {msg.content}
              {/if}
            </div>
          </div>
        {/each}
        {#if sending}
          <div class="flex justify-start">
            <div class="bg-hazardo-surface border border-hazardo-lightGray rounded-lg px-3 py-2 text-sm text-hazardo-lightGray">
              {$t('chatbot.thinking')}
            </div>
          </div>
        {/if}
        {#if pendingActions.length > 0}
          <div class="flex justify-start">
            <div class="max-w-[85%] bg-hazardo-accent/10 border border-hazardo-accent/30 rounded-lg px-3 py-2 text-sm">
              <p class="font-medium text-hazardo-text mb-2">{$t('chatbot.ai_wants_add')}</p>
              <ul class="text-xs text-hazardo-text/70 mb-2 space-y-1">
                {#each pendingActions.filter(a => a.type === 'create_item') as action}
                  <li>
                    <span class="font-medium">{$t('chatbot.item')}</span> {action.name}
                  </li>
                {/each}
              </ul>
              <p class="text-xs font-medium text-hazardo-text mb-1">{$t('chatbot.select_category')}</p>
              <div class="flex flex-wrap gap-1.5 mb-2">
                {#each userCategories as cat}
                  <button
                    class="flex items-center gap-1 px-2 py-1 rounded-full bg-hazardo-surface border border-hazardo-lightGray text-xs text-hazardo-text hover:border-hazardo-accent hover:bg-hazardo-accent/10 transition-colors"
                    on:click={async () => { const actions = [...pendingActions]; pendingActions = []; await executeActions(actions, cat); }}
                  >
                    <Icon name={cat.category_icon} size={14} />
                    {cat.category_name}
                  </button>
                {/each}
              </div>
              <div class="flex gap-2">
                <button class="px-3 py-1 rounded bg-hazardo-primary text-white text-xs font-medium" on:click={async () => { const actions = [...pendingActions]; pendingActions = []; await executeActions(actions); }}>
                  {$t('chatbot.create_new')}
                </button>
                <button class="px-3 py-1 rounded border border-hazardo-lightGray text-hazardo-text text-xs" on:click={() => { pendingActions = []; }}>
                  {$t('chatbot.cancel')}
                </button>
              </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Input area -->
    <div class="border-t border-hazardo-lightGray py-3 mb-4">
      {#if availableModels.length > 0}
        <div class="flex items-center gap-2 mb-2">
          <span class="text-xs text-hazardo-lightGray shrink-0">{$t('chatbot.model')}</span>
          <div class="flex-1">
            <SelectDropdown options={availableModels} bind:selected={llmModel} placeholder="Select model..." on:change={saveModelSelection} />
          </div>
        </div>
      {/if}
      <div class="flex items-end gap-2">
        <textarea
          rows="3"
          class="flex-1 border rounded-2xl px-4 py-2 text-sm border-hazardo-lightGray focus:outline-hazardo-accent bg-hazardo-surface text-hazardo-text resize-none"
          placeholder={$t('chatbot.placeholder')}
          bind:value={inputText}
          on:keydown={handleKeydown}
          disabled={sending}
        ></textarea>
        <button
          class="bg-hazardo-accent text-white rounded-full w-9 h-9 flex items-center justify-center hover:bg-hazardo-primary transition-colors disabled:opacity-50"
          on:click={sendMessage}
          disabled={sending || !inputText.trim()}
        >
          <Icon name="send" size={16} />
        </button>
      </div>
    </div>
  {/if}
</main>

<style>
  :global(.markdown-content h1) { font-size: 1.25rem; font-weight: 700; margin: 0.5rem 0; }
  :global(.markdown-content h2) { font-size: 1.125rem; font-weight: 600; margin: 0.5rem 0; }
  :global(.markdown-content h3) { font-size: 1rem; font-weight: 600; margin: 0.25rem 0; }
  :global(.markdown-content p) { margin: 0.25rem 0; }
  :global(.markdown-content ul),
  :global(.markdown-content ol) { margin: 0.25rem 0; padding-left: 1.5rem; }
  :global(.markdown-content li) { margin: 0.125rem 0; }
  :global(.markdown-content code) { background: var(--color-hazardo-background); padding: 0.125rem 0.25rem; border-radius: 0.25rem; font-size: 0.8em; }
  :global(.markdown-content pre) { background: var(--color-hazardo-background); padding: 0.5rem; border-radius: 0.375rem; overflow-x: auto; margin: 0.5rem 0; }
  :global(.markdown-content pre code) { background: none; padding: 0; }
  :global(.markdown-content strong) { font-weight: 600; }
  :global(.markdown-content em) { font-style: italic; }
  :global(.markdown-content a) { color: var(--color-hazardo-accent); text-decoration: underline; }
  :global(.markdown-content blockquote) { border-left: 3px solid var(--color-hazardo-lightGray); padding-left: 0.75rem; margin: 0.5rem 0; color: var(--color-hazardo-lightGray); }
  :global(.markdown-content table) { border-collapse: collapse; margin: 0.5rem 0; width: 100%; }
  :global(.markdown-content th),
  :global(.markdown-content td) { border: 1px solid var(--color-hazardo-lightGray); padding: 0.25rem 0.5rem; font-size: 0.85em; }
  :global(.markdown-content th) { background: var(--color-hazardo-background); font-weight: 600; }
  :global(.markdown-content hr) { border: none; border-top: 1px solid var(--color-hazardo-lightGray); margin: 0.5rem 0; }
</style>
