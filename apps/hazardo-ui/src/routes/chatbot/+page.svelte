<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { marked } from 'marked';
  import { selectedUser } from '../../stores/userStore';
  import { showToast } from '../../stores/toastStore';
  import Icon from '../../components/atoms/Icon.svelte';
  import SelectDropdown from '../../components/molecules/SelectDropdown.svelte';
  import type { ChatMessage, AppSetting, Category } from '$lib/types';

  marked.setOptions({ breaks: true, gfm: true });

  const renderer = new marked.Renderer();
  renderer.link = function ({ href, title, text }: { href: string; title?: string | null | undefined; text: string }) {
    const titleAttr = title ? ` title="${title}"` : '';
    return `<a href="${href}"${titleAttr} target="_blank" rel="noopener noreferrer">${text}</a>`;
  };
  marked.use({ renderer });

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
  let isRecording = false;
  let speechRecognition: any = null;

  $: if ($selectedUser) {
    loadChat($selectedUser.user_id);
    loadLlmSettings($selectedUser.user_id);
    loadCategories($selectedUser.user_id);
  }

  onMount(() => {
    document.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      const anchor = target.closest('a[href]') as HTMLAnchorElement | null;
      if (anchor && anchor.href && (anchor.href.startsWith('http://') || anchor.href.startsWith('https://'))) {
        e.preventDefault();
        openUrl(anchor.href);
      }
    });
  });

  async function loadChat(userId: number) {
    try {
      messages = await invoke<ChatMessage[]>('get_chat_messages', { userId });
      await tick();
      scrollToBottom();
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
    if (chatAutoCreate === 'true') {
      prompt += `\n\nYou can create categories and items in the user's Hazardo database.
Database structure:
- Categories: name, icon (one of: activity, restaurant, board-game, valentine, movie, gift, workout, travel, cooking, misc)
- Items: name, belongs to a category, time_pref (AM, PM, Night, or Mixed), vibe_pref (Friend, Date, Family, or Mixed), optional notes

When the user asks you to add/create a category or item, include the action in your response using these exact tags:

To create a category:
[HAZARDO_CREATE_CATEGORY]{"name": "Category Name", "icon": "icon-name"}[/HAZARDO_CREATE_CATEGORY]

To create an item in an existing category:
[HAZARDO_CREATE_ITEM]{"category": "Category Name", "name": "Item Name", "time_pref": "Mixed", "vibe_pref": "Mixed", "notes": "optional"}[/HAZARDO_CREATE_ITEM]

You can include multiple actions in one response. Always also provide a natural language confirmation of what you created.`;
    }
    return prompt;
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

  function parseActions(response: string): { text: string; actions: any[] } {
    const actions: any[] = [];
    let text = response;
    const catRegex = /\[HAZARDO_CREATE_CATEGORY\]([\s\S]*?)\[\/HAZARDO_CREATE_CATEGORY\]/g;
    text = text.replace(catRegex, (_, json) => {
      try { actions.push({ type: 'create_category', ...JSON.parse(json.trim()) }); } catch {}
      return '';
    });
    const itemRegex = /\[HAZARDO_CREATE_ITEM\]([\s\S]*?)\[\/HAZARDO_CREATE_ITEM\]/g;
    text = text.replace(itemRegex, (_, json) => {
      try { actions.push({ type: 'create_item', ...JSON.parse(json.trim()) }); } catch {}
      return '';
    });
    return { text: text.trim(), actions };
  }

  async function executeActions(actions: any[]) {
    if (!$selectedUser || chatAutoCreate !== 'true') return;
    for (const action of actions) {
      try {
        if (action.type === 'create_category') {
          await invoke('create_category', {
            userId: $selectedUser.user_id,
            categoryName: action.name,
            categoryIcon: action.icon || 'misc',
          });
        } else if (action.type === 'create_item') {
          const cats = await invoke<Category[]>('get_categories', { userId: $selectedUser.user_id });
          const cat = cats.find(c => c.category_name.toLowerCase() === (action.category || '').toLowerCase());
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
          }
        }
      } catch (e) {
        console.error('execute action failed', action, e);
      }
    }
    if ($selectedUser) await loadCategories($selectedUser.user_id);
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

      // Parse actions if auto-create enabled
      const { text: cleanResponse, actions } = chatAutoCreate === 'true'
        ? parseActions(aiResponse)
        : { text: aiResponse, actions: [] };

      if (actions.length > 0) {
        await executeActions(actions);
      }

      const assistantMsg = await invoke<ChatMessage>('save_chat_message', {
        userId: $selectedUser.user_id,
        role: 'assistant',
        content: cleanResponse,
      });
      messages = [...messages, assistantMsg];
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
    const body: Record<string, any> = {
      messages: [
        { role: 'system', content: systemPrompt },
        ...messages.slice(-20).map(m => ({ role: m.role, content: m.content })),
        { role: 'user', content: userMessage },
      ],
      max_tokens: 1024,
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

  function toggleSpeechRecognition() {
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) {
      showToast('Voice input is not supported on this device', 'error');
      return;
    }
    if (isRecording && speechRecognition) {
      speechRecognition.stop();
      isRecording = false;
      return;
    }
    speechRecognition = new SpeechRecognition();
    speechRecognition.continuous = false;
    speechRecognition.interimResults = false;
    speechRecognition.lang = 'en-US';
    speechRecognition.onresult = (event: any) => {
      const transcript = event.results[0][0].transcript;
      inputText = (inputText ? inputText + ' ' : '') + transcript;
      isRecording = false;
    };
    speechRecognition.onerror = () => { isRecording = false; };
    speechRecognition.onend = () => { isRecording = false; };
    speechRecognition.start();
    isRecording = true;
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
      <p class="text-hazardo-text font-semibold">ChatBot needs configuration</p>
      <p class="text-sm text-hazardo-lightGray max-w-xs">
        Go to <a href="/setting" class="text-hazardo-accent underline">Settings</a> &gt; Manage LLM Model (AI) to configure your AI connection.
      </p>
    </div>
  {:else}
    <!-- Chat messages -->
    <div class="flex-1 overflow-y-auto py-4 flex flex-col gap-3" bind:this={chatContainer}>
      {#if messages.length === 0}
        <div class="flex-1 flex items-center justify-center">
          <p class="text-sm text-hazardo-lightGray">Start a conversation with Hazardo AI!</p>
        </div>
      {:else}
        {#each messages as msg}
          <div class="flex {msg.role === 'user' ? 'justify-end' : 'justify-start'}">
            <div class="max-w-[80%] rounded-lg px-3 py-2 text-sm {msg.role === 'user' ? 'bg-hazardo-accent text-white' : 'bg-white border border-hazardo-lightGray text-hazardo-text'}">
              {#if msg.role === 'assistant'}
                <div class="markdown-content">{@html renderMarkdown(msg.content)}</div>
              {:else}
                {msg.content}
              {/if}
            </div>
          </div>
        {/each}
        {#if sending}
          <div class="flex justify-start">
            <div class="bg-white border border-hazardo-lightGray rounded-lg px-3 py-2 text-sm text-hazardo-lightGray">
              Thinking...
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Input area -->
    <div class="border-t border-hazardo-lightGray py-3 mb-4">
      {#if availableModels.length > 0}
        <div class="flex items-center gap-2 mb-2">
          <span class="text-xs text-hazardo-lightGray shrink-0">Model:</span>
          <div class="flex-1">
            <SelectDropdown options={availableModels} bind:selected={llmModel} placeholder="Select model..." on:change={saveModelSelection} />
          </div>
        </div>
      {/if}
      <div class="flex items-center gap-2">
        <button
          class="rounded-full w-9 h-9 flex items-center justify-center transition-colors {isRecording ? 'bg-red-500 text-white animate-pulse' : 'text-hazardo-lightGray hover:text-hazardo-accent'}"
          on:click={toggleSpeechRecognition}
          title={isRecording ? 'Stop recording' : 'Voice input'}
        >
          <Icon name="microphone" size={18} />
        </button>
        <input
          type="text"
          class="flex-1 border rounded-full px-4 py-2 text-sm border-hazardo-lightGray focus:outline-hazardo-accent bg-white"
          placeholder="Ask me anything..."
          bind:value={inputText}
          on:keydown={handleKeydown}
          disabled={sending}
        />
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
  :global(.markdown-content code) { background: #f1f5f9; padding: 0.125rem 0.25rem; border-radius: 0.25rem; font-size: 0.8em; }
  :global(.markdown-content pre) { background: #f1f5f9; padding: 0.5rem; border-radius: 0.375rem; overflow-x: auto; margin: 0.5rem 0; }
  :global(.markdown-content pre code) { background: none; padding: 0; }
  :global(.markdown-content strong) { font-weight: 600; }
  :global(.markdown-content em) { font-style: italic; }
  :global(.markdown-content a) { color: #3b82f6; text-decoration: underline; }
  :global(.markdown-content blockquote) { border-left: 3px solid #d1d5db; padding-left: 0.75rem; margin: 0.5rem 0; color: #6b7280; }
  :global(.markdown-content table) { border-collapse: collapse; margin: 0.5rem 0; width: 100%; }
  :global(.markdown-content th),
  :global(.markdown-content td) { border: 1px solid #d1d5db; padding: 0.25rem 0.5rem; font-size: 0.85em; }
  :global(.markdown-content th) { background: #f9fafb; font-weight: 600; }
  :global(.markdown-content hr) { border: none; border-top: 1px solid #e5e7eb; margin: 0.5rem 0; }
</style>
