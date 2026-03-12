<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
  import { selectedUser, users, loadUsers } from '../../stores/userStore';
  import Title from '../../components/atoms/Title.svelte';
  import Icon from '../../components/atoms/Icon.svelte';
  import FormLabel from '../../components/atoms/FormLabel.svelte';
  import FormInput from '../../components/atoms/FormInput.svelte';
  import Modal from '../../components/organisms/Modal.svelte';
  import SelectDropdown from '../../components/molecules/SelectDropdown.svelte';
  import SearchBar from '../../components/molecules/SearchBar.svelte';
  import ScrollToTop from '../../components/atoms/ScrollToTop.svelte';
  import { showToast } from '../../stores/toastStore';
  import type { AppSetting, Device, Item, Category, Pick as PickType, User } from '$lib/types';

  let activeSection = '';

  // Device
  let device: Device | null = null;
  let editDeviceName = '';

  // LLM Settings
  let llmProvider = '';
  let llmApiKey = '';
  let llmEndpoint = '';
  let llmModel = '';
  let llmApiKeys: Record<string, string> = {};
  let chatPromptFormat = 'You are Hazardo AI assistant. Help the user with activity planning and recommendations.';
  let chatAutoCreate = 'false';
  let locationEnabled = false;
  let locationOverride = '';
  let locationSearch = '';
  let locationSearching = false;
  let locationSearchResults: { name: string; lat: string; lon: string }[] = [];
  let locationPermissionStatus: 'unknown' | 'requesting' | 'granted' | 'denied' = 'unknown';
  let ollamaModels: { value: string; label: string }[] = [];
  let ollamaTestStatus: 'idle' | 'testing' | 'success' | 'error' = 'idle';
  let ollamaTestMsg = '';
  let openaiModels: { value: string; label: string }[] = [];
  let openaiTestStatus: 'idle' | 'testing' | 'success' | 'error' = 'idle';
  let openaiTestMsg = '';
  let geminiModels: { value: string; label: string }[] = [];
  let geminiTestStatus: 'idle' | 'testing' | 'success' | 'error' = 'idle';
  let geminiTestMsg = '';

  // Recycle bin
  let recycleBinTab: 'vault' | 'categories' | 'picked' = 'vault';
  let deletedItems: Item[] = [];
  let deletedCategories: Category[] = [];
  let deletedPicks: PickType[] = [];
  let recycleSearch = '';
  let recycleContainer: HTMLElement;

  // Manage User
  let showManageUser = false;

  // Export
  let exportUserSelection: Record<number, boolean> = {};
  let exportAllUsers = false;
  let exportCategories: Category[] = [];
  let exportCategorySelection: Record<number, boolean> = {};
  let exportAllCategories = false;
  let exportFormat = 'json';
  let exportResult = '';
  let exportLoading = false;
  let exportFilePath = '';
  let exportSaving = false;
  let exportSharing = false;
  const exportFormatOptions = [
    { value: 'json', label: 'JSON' },
    { value: 'csv', label: 'CSV' },
    { value: 'markdown', label: 'Markdown' },
    { value: 'zip', label: 'ZIP (with images)' },
  ];

  // Import
  let importFormat = 'json';
  let importFileContent = '';
  let importResult = '';
  let importLoading = false;
  let importFileInput: HTMLInputElement;

  const llmProviderOptions = [
    { value: '', label: 'Not configured' },
    { value: 'openai', label: 'OpenAI (GPT)' },
    { value: 'gemini', label: 'Google Gemini' },
    { value: 'local', label: 'Local LLM (Ollama/LM Studio)' },
  ];

  const settingsSections = [
    { key: 'documentation', label: 'Documentation', icon: 'file-text' as const },
    { key: 'recycle-bin', label: 'Recycle Bin', icon: 'recycle' as const },
    { key: 'bulk-import', label: 'Bulk Import', icon: 'import' as const },
    { key: 'bulk-export', label: 'Bulk Export', icon: 'export' as const },
    { key: 'theme', label: 'Theme', extra: '(Auto System)', icon: 'palette' as const },
    { key: 'language', label: 'Language', extra: '(English)', icon: 'globe' as const },
    { key: 'device-name', label: 'Device Name', icon: 'monitor' as const },
    { key: 'manage-user', label: 'Manage User', icon: 'users' as const },
    { key: 'manage-sync', label: 'Manage Synchronization', icon: 'sync' as const },
    { key: 'manage-location', label: 'Location Services', icon: 'map-pin' as const },
    { key: 'manage-llm', label: 'Manage LLM Model (AI)', icon: 'brain' as const },
  ];

  $: if ($selectedUser) {
    loadSettings($selectedUser.user_id);
    loadDevice();
  }

  // When provider changes, swap API key to the saved one for that provider
  $: {
    if (llmProvider && llmProvider !== 'local') {
      llmApiKey = llmApiKeys[llmProvider] || '';
    } else if (llmProvider === 'local') {
      llmApiKey = '';
    }
  }

  $: {
    if (exportAllUsers) {
      $users.forEach(u => exportUserSelection[u.user_id] = true);
    }
    exportUserSelection = exportUserSelection;
  }

  $: selectedExportUserIds = Object.entries(exportUserSelection).filter(([_, v]) => v).map(([k]) => Number(k));

  $: {
    if (selectedExportUserIds.length > 0 && activeSection === 'bulk-export') {
      loadExportCategories(selectedExportUserIds);
    }
  }

  $: {
    if (exportAllCategories) {
      exportCategories.forEach(c => exportCategorySelection[c.category_id] = true);
    }
    exportCategorySelection = exportCategorySelection;
  }

  async function loadExportCategories(uids: number[]) {
    let cats: Category[] = [];
    for (const uid of uids) {
      try {
        const c = await invoke<Category[]>('get_categories', { userId: uid });
        cats = [...cats, ...c];
      } catch (_) {}
    }
    exportCategories = cats;
  }

  async function loadDevice() {
    try {
      device = await invoke<Device | null>('get_device');
      editDeviceName = device?.device_name || '';
    } catch (e) {
      console.error('get_device failed', e);
    }
  }

  async function loadSettings(userId: number) {
    try {
      const settings = await invoke<AppSetting[]>('get_all_settings', { userId });
      const map = new Map(settings.map(s => [s.setting_key, s.setting_value]));
      llmProvider = map.get('llm_provider') || '';
      llmEndpoint = map.get('llm_endpoint') || '';
      llmModel = map.get('llm_model') || '';
      chatPromptFormat = map.get('chat_prompt_format') || 'You are Hazardo AI assistant. Help the user with activity planning and recommendations.';
      chatAutoCreate = map.get('chat_auto_create') || 'false';
      locationEnabled = map.get('location_enabled') === 'true';
      locationOverride = map.get('location_override') || '';
      // Load per-provider API keys
      llmApiKeys = {
        openai: map.get('llm_api_key_openai') || '',
        gemini: map.get('llm_api_key_gemini') || '',
      };
      // Also migrate old single key if provider-specific not set
      const oldKey = map.get('llm_api_key') || '';
      if (oldKey && llmProvider && llmProvider !== 'local' && !llmApiKeys[llmProvider]) {
        llmApiKeys[llmProvider] = oldKey;
      }
      llmApiKey = llmApiKeys[llmProvider] || '';
      // If local and endpoint set, auto-test
      if (llmProvider === 'local' && llmEndpoint) {
        testOllamaConnection();
      }
    } catch (e) {
      console.error('load settings failed', e);
    }
  }

  async function saveSetting(key: string, value: string) {
    if (!$selectedUser) return;
    try {
      await invoke('set_setting', { userId: $selectedUser.user_id, key, value });
    } catch (e) {
      console.error('set_setting failed', e);
    }
  }

  async function saveDeviceName() {
    if (!editDeviceName.trim()) return;
    try {
      await invoke('update_device_name', { deviceName: editDeviceName.trim() });
      await loadDevice();
      activeSection = '';
    } catch (e) {
      console.error('update_device_name failed', e);
    }
  }

  async function testOllamaConnection() {
    if (!llmEndpoint.trim()) return;
    ollamaTestStatus = 'testing';
    ollamaTestMsg = '';
    ollamaModels = [];
    try {
      let base = llmEndpoint.trim().replace(/\/+$/, '');
      const url = `${base}/api/tags`;
      const resp = await tauriFetch(url, {
        method: 'GET',
        headers: { 'Origin': 'http://localhost' },
      });
      if (!resp.ok) {
        const body = await resp.text().catch(() => '');
        if (resp.status === 403 && !body) {
          throw new Error('403 Forbidden – Ollama is blocking the request (CORS). Set OLLAMA_ORIGINS=* on the server and restart Ollama.');
        }
        throw new Error(`Status ${resp.status}: ${body.slice(0, 200)}`);
      }
      const data = await resp.json();
      const models = (data.models || []) as { name: string }[];
      ollamaModels = models.map((m: { name: string }) => ({ value: m.name, label: m.name }));
      ollamaTestStatus = 'success';
      ollamaTestMsg = `Connected! ${models.length} model(s) found.`;
      if (models.length > 0 && !llmModel) {
        llmModel = models[0].name;
      }
    } catch (e: any) {
      ollamaTestStatus = 'error';
      ollamaTestMsg = `Connection failed: ${e.message || e}`;
    }
  }

  async function testOpenAIConnection() {
    if (!llmApiKey.trim()) return;
    openaiTestStatus = 'testing';
    openaiTestMsg = '';
    openaiModels = [];
    try {
      const resp = await tauriFetch('https://api.openai.com/v1/models', {
        method: 'GET',
        headers: { 'Authorization': `Bearer ${llmApiKey.trim()}` },
      });
      if (!resp.ok) {
        const body = await resp.text().catch(() => '');
        throw new Error(`Status ${resp.status}: ${body.slice(0, 200)}`);
      }
      const data = await resp.json();
      const models = (data.data || []) as { id: string }[];
      const chatModels = models
        .filter((m: { id: string }) => m.id.startsWith('gpt-') || m.id.startsWith('o1') || m.id.startsWith('o3') || m.id.startsWith('chatgpt'))
        .sort((a: { id: string }, b: { id: string }) => a.id.localeCompare(b.id));
      openaiModels = chatModels.map((m: { id: string }) => ({ value: m.id, label: m.id }));
      openaiTestStatus = 'success';
      openaiTestMsg = `Connected! ${chatModels.length} chat model(s) found.`;
      if (chatModels.length > 0 && !llmModel) {
        llmModel = chatModels[0].id;
      }
    } catch (e: any) {
      openaiTestStatus = 'error';
      openaiTestMsg = `Connection failed: ${e.message || e}`;
    }
  }

  async function testGeminiConnection() {
    if (!llmApiKey.trim()) return;
    geminiTestStatus = 'testing';
    geminiTestMsg = '';
    geminiModels = [];
    try {
      const resp = await tauriFetch(`https://generativelanguage.googleapis.com/v1beta/models?key=${llmApiKey.trim()}`, {
        method: 'GET',
      });
      if (!resp.ok) {
        const body = await resp.text().catch(() => '');
        throw new Error(`Status ${resp.status}: ${body.slice(0, 200)}`);
      }
      const data = await resp.json();
      const models = (data.models || []) as { name: string; displayName: string; supportedGenerationMethods?: string[] }[];
      const genModels = models
        .filter((m: any) => m.supportedGenerationMethods?.includes('generateContent'))
        .sort((a: any, b: any) => a.name.localeCompare(b.name));
      geminiModels = genModels.map((m: any) => ({
        value: m.name.replace('models/', ''),
        label: m.displayName || m.name.replace('models/', ''),
      }));
      geminiTestStatus = 'success';
      geminiTestMsg = `Connected! ${genModels.length} model(s) found.`;
      if (genModels.length > 0 && !llmModel) {
        llmModel = genModels[0].name.replace('models/', '');
      }
    } catch (e: any) {
      geminiTestStatus = 'error';
      geminiTestMsg = `Connection failed: ${e.message || e}`;
    }
  }

  async function saveLlmSettings() {
    if (llmProvider === 'local') llmApiKey = '';
    // Save current key to provider-specific storage
    if (llmProvider && llmProvider !== 'local') {
      llmApiKeys[llmProvider] = llmApiKey;
      llmApiKeys = llmApiKeys;
    }
    await saveSetting('llm_provider', llmProvider);
    await saveSetting('llm_api_key', llmApiKey);
    await saveSetting('llm_api_key_openai', llmApiKeys['openai'] || '');
    await saveSetting('llm_api_key_gemini', llmApiKeys['gemini'] || '');
    await saveSetting('llm_endpoint', llmEndpoint);
    await saveSetting('llm_model', llmModel);
    await saveSetting('chat_prompt_format', chatPromptFormat);
    await saveSetting('chat_auto_create', chatAutoCreate);
    activeSection = '';
  }

  function requestLocationPermission() {
    if (!navigator.geolocation) {
      locationPermissionStatus = 'denied';
      return;
    }
    locationPermissionStatus = 'requesting';
    navigator.geolocation.getCurrentPosition(
      () => {
        locationPermissionStatus = 'granted';
        locationEnabled = true;
        saveSetting('location_enabled', 'true');
      },
      () => { locationPermissionStatus = 'denied'; },
      { enableHighAccuracy: true, timeout: 15000 }
    );
  }

  async function searchLocation() {
    if (!locationSearch.trim()) return;
    locationSearching = true;
    locationSearchResults = [];
    try {
      const q = encodeURIComponent(locationSearch.trim());
      const resp = await tauriFetch(`https://nominatim.openstreetmap.org/search?q=${q}&format=json&limit=5`, {
        method: 'GET',
        headers: { 'User-Agent': 'HazardoApp/1.0' },
      });
      if (resp.ok) {
        const data = await resp.json();
        locationSearchResults = (data as any[]).map(r => ({
          name: r.display_name?.split(',').slice(0, 3).join(',') || r.display_name,
          lat: r.lat,
          lon: r.lon,
        }));
      }
    } catch (_) {}
    locationSearching = false;
  }

  async function selectLocationOverride(loc: { name: string; lat: string; lon: string }) {
    const cityName = loc.name.split(',')[0].trim();
    locationOverride = `${cityName}|${loc.lat}|${loc.lon}`;
    await saveSetting('location_override', locationOverride);
    locationSearchResults = [];
    locationSearch = '';
  }

  async function clearLocationOverride() {
    locationOverride = '';
    await saveSetting('location_override', '');
  }

  async function saveLocationSettings() {
    await saveSetting('location_enabled', locationEnabled ? 'true' : 'false');
    await saveSetting('location_override', locationOverride);
    activeSection = '';
  }

  async function loadRecycleBin() {
    if (!$selectedUser) return;
    try {
      deletedItems = await invoke<Item[]>('get_deleted_items', { userId: $selectedUser.user_id });
      deletedCategories = await invoke<Category[]>('get_deleted_categories', { userId: $selectedUser.user_id });
      deletedPicks = await invoke<PickType[]>('get_deleted_picks', { userId: $selectedUser.user_id });
    } catch (e) {
      console.error('load recycle bin failed', e);
    }
  }

  async function restoreItem(itemId: number) {
    try { await invoke('restore_item', { itemId }); await loadRecycleBin(); } catch (e) { console.error(e); }
  }
  async function permanentDeleteItem(itemId: number) {
    try { await invoke('permanent_delete_item', { itemId }); await loadRecycleBin(); } catch (e) { console.error(e); }
  }
  async function restoreCategory(categoryId: number) {
    try { await invoke('restore_category', { categoryId }); await loadRecycleBin(); } catch (e) { console.error(e); }
  }
  async function permanentDeleteCategory(categoryId: number) {
    try { await invoke('permanent_delete_category', { categoryId }); await loadRecycleBin(); } catch (e) { console.error(e); }
  }
  async function restorePick(pickId: number) {
    try { await invoke('restore_pick', { pickId }); await loadRecycleBin(); } catch (e) { console.error(e); }
  }
  async function permanentDeletePick(pickId: number) {
    try { await invoke('permanent_delete_pick', { pickId }); await loadRecycleBin(); } catch (e) { console.error(e); }
  }

  function openSection(key: string) {
    activeSection = activeSection === key ? '' : key;
    if (key === 'recycle-bin') loadRecycleBin();
    if (key === 'bulk-export') {
      exportUserSelection = {};
      exportCategorySelection = {};
      exportResult = '';
      exportAllUsers = false;
      exportAllCategories = false;
    }
    if (key === 'bulk-import') {
      importResult = '';
      importFileContent = '';
    }
  }

  async function deleteUser(userId: number) {
    try { await invoke('delete_user', { userId }); await loadUsers(); } catch (e) { console.error(e); }
  }

  // Export functions
  async function handleExport() {
    exportLoading = true;
    exportResult = '';
    exportFilePath = '';
    try {
      const userIds = selectedExportUserIds;
      const catIds = Object.entries(exportCategorySelection).filter(([_, v]) => v).map(([k]) => Number(k));
      if (exportFormat === 'zip') {
        exportFilePath = await invoke<string>('export_zip', {
          userIds,
          categoryIds: catIds,
          target: 'downloads',
        });
        exportResult = 'ZIP file exported successfully with data and images.';
        showToast('Export successful!', 'success');
        activeSection = '';
      } else {
        const jsonStr = await invoke<string>('export_data', { userIds, categoryIds: catIds });
        if (exportFormat === 'json') {
          exportResult = jsonStr;
        } else if (exportFormat === 'csv') {
          exportResult = jsonToCsv(JSON.parse(jsonStr));
        } else if (exportFormat === 'markdown') {
          exportResult = jsonToMarkdown(JSON.parse(jsonStr));
        }
      }
    } catch (e: any) {
      exportResult = `Export error: ${e.message || e}`;
    }
    exportLoading = false;
  }

  function jsonToCsv(data: any[]): string {
    let csv = 'User,Category,Icon,Item,TimePref,VibePref,Notes\n';
    for (const u of data) {
      for (const c of u.categories || []) {
        for (const i of c.items || []) {
          csv += `"${u.user}","${c.name}","${c.icon}","${i.name}","${i.time_pref}","${i.vibe_pref}","${(i.notes || '').replace(/"/g, '""')}"\n`;
        }
      }
    }
    csv += '\nPicked\nUser,ItemName,Category,PickDate,TimePref,VibePref,Notes,HasImage\n';
    for (const u of data) {
      for (const p of u.picks || []) {
        csv += `"${u.user}","${p.item_name}","${p.category_name}","${p.pick_date}","${p.time_pref || ''}","${p.vibe_pref || ''}","${(p.notes || '').replace(/"/g, '""')}","${p.image_path ? 'Yes' : 'No'}"\n`;
      }
    }
    return csv;
  }

  function jsonToMarkdown(data: any[]): string {
    let md = '';
    for (const u of data) {
      md += `# ${u.user}\n\n`;
      for (const c of u.categories || []) {
        md += `## ${c.name} (${c.icon})\n\n`;
        md += `| Item | Time | Vibe | Notes |\n|------|------|------|-------|\n`;
        for (const i of c.items || []) {
          md += `| ${i.name} | ${i.time_pref} | ${i.vibe_pref} | ${i.notes || ''} |\n`;
        }
        md += '\n';
      }
      if (u.picks?.length > 0) {
        md += `## Picked Items\n\n`;
        md += `| Date | Item | Category | Time | Vibe | Notes | Image |\n|------|------|----------|------|------|-------|-------|\n`;
        for (const p of u.picks) {
          md += `| ${p.pick_date} | ${p.item_name} | ${p.category_name} | ${p.time_pref || ''} | ${p.vibe_pref || ''} | ${p.notes || ''} | ${p.image_path ? 'Yes' : 'No'} |\n`;
        }
        md += '\n';
      }
    }
    return md;
  }

  async function copyExportResult() {
    try { await navigator.clipboard.writeText(exportResult); } catch (_) {}
  }

  async function saveExportToFile() {
    if (!exportResult) return;
    exportSaving = true;
    try {
      const ext = exportFormat === 'csv' ? 'csv' : exportFormat === 'markdown' ? 'md' : 'json';
      const filename = `hazardo_export_${Date.now()}.${ext}`;
      exportFilePath = await invoke<string>('save_export_file', { content: exportResult, filename, target: 'downloads' });
      showToast('File saved to Downloads!', 'success');
      activeSection = '';
    } catch (e: any) {
      exportFilePath = `Error: ${e.message || e}`;
    }
    exportSaving = false;
  }

  async function shareExport() {
    exportSharing = true;
    try {
      let file: File;
      if (exportFormat === 'zip' && exportFilePath) {
        const b64 = await invoke<string>('read_file_bytes', { path: exportFilePath });
        const bytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
        const blob = new Blob([bytes], { type: 'application/zip' });
        file = new File([blob], 'hazardo_export.zip', { type: 'application/zip' });
      } else {
        const ext = exportFormat === 'csv' ? 'csv' : exportFormat === 'markdown' ? 'md' : 'json';
        const mime = exportFormat === 'csv' ? 'text/csv' : exportFormat === 'markdown' ? 'text/markdown' : 'application/json';
        const blob = new Blob([exportResult], { type: mime });
        file = new File([blob], `hazardo_export.${ext}`, { type: mime });
      }
      if (navigator.canShare && navigator.canShare({ files: [file] })) {
        await navigator.share({ files: [file], title: 'Hazardo Export' });
      } else {
        await saveExportToFile();
      }
    } catch (e: any) {
      if (e.name !== 'AbortError') {
        exportFilePath = `Share error: ${e.message || e}`;
      }
    }
    exportSharing = false;
  }

  // Import functions
  function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => { importFileContent = reader.result as string; };
    reader.readAsText(file);
  }

  async function handleImport() {
    if (!importFileContent.trim() || !$selectedUser) return;
    importLoading = true;
    importResult = '';
    try {
      let jsonData: string;
      if (importFormat === 'json') {
        jsonData = importFileContent;
      } else if (importFormat === 'csv') {
        jsonData = JSON.stringify(csvToJson(importFileContent));
      } else {
        jsonData = JSON.stringify(markdownToJson(importFileContent));
      }
      importResult = await invoke<string>('import_data', { userId: $selectedUser.user_id, data: jsonData });
      showToast(importResult, 'success');
      activeSection = '';
    } catch (e: any) {
      importResult = `Import error: ${e.message || e}`;
    }
    importLoading = false;
  }

  function csvToJson(csv: string): any[] {
    const lines = csv.trim().split('\n');
    if (lines.length < 2) return [];
    const cats: Record<string, { name: string; icon: string; items: any[] }> = {};
    for (let i = 1; i < lines.length; i++) {
      const cols = lines[i].match(/("(?:[^"]|"")*"|[^,]*)/g)?.map(c => c.replace(/^"|"$/g, '').replace(/""/g, '"')) || [];
      const catKey = cols[1] || 'Imported';
      if (!cats[catKey]) cats[catKey] = { name: catKey, icon: cols[2] || 'misc', items: [] };
      cats[catKey].items.push({ name: cols[3] || 'Unnamed', time_pref: cols[4] || 'Mixed', vibe_pref: cols[5] || 'Mixed', notes: cols[6] || null });
    }
    return [{ user: 'import', categories: Object.values(cats) }];
  }

  function markdownToJson(md: string): any[] {
    const cats: { name: string; icon: string; items: any[] }[] = [];
    let currentCat: { name: string; icon: string; items: any[] } | null = null;
    for (const line of md.split('\n')) {
      const catMatch = line.match(/^## (.+?) \((.+?)\)/);
      if (catMatch) {
        currentCat = { name: catMatch[1], icon: catMatch[2], items: [] };
        cats.push(currentCat);
        continue;
      }
      if (currentCat && line.startsWith('|') && !line.includes('---') && !line.includes('Item')) {
        const parts = line.split('|').map(s => s.trim()).filter(Boolean);
        if (parts.length >= 3) {
          currentCat.items.push({ name: parts[0], time_pref: parts[1], vibe_pref: parts[2], notes: parts[3] || null });
        }
      }
    }
    return [{ user: 'import', categories: cats }];
  }

  function formatDate(dateStr: string): string {
    if (!dateStr) return '';
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString('fr-FR', { day: '2-digit', month: 'short', year: 'numeric' });
    } catch { return dateStr; }
  }
</script>

<main class="max-w-lg mx-auto px-4">
  <div class="flex flex-col items-center">
    <div class="mt-6 mb-6 bg-hazardo-accent px-2 rounded">
      <Title title="Setting" />
    </div>
  </div>

  <div class="flex flex-col">
    {#each settingsSections as section}
      <button
        class="flex items-center justify-between py-4 border-b border-hazardo-lightGray/30 text-left hover:bg-hazardo-background/50 px-2 rounded transition-colors"
        on:click={() => openSection(section.key)}
      >
        <span class="text-sm font-medium">
          {section.label}
          {#if section.extra}
            <span class="text-hazardo-lightGray font-normal">{section.extra}</span>
          {/if}
        </span>
        <Icon name="chevron-right" size={16} />
      </button>
    {/each}
  </div>
</main>

<!-- Device Name Modal -->
<Modal show={activeSection === 'device-name'} title="Device Name" on:close={() => activeSection = ''}>
  <form on:submit|preventDefault={saveDeviceName} class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label="Device Name:" />
      <FormInput bind:value={editDeviceName} placeholder="Enter device name..." />
    </div>
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => activeSection = ''}>Cancel</button>
      <button type="submit" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm">Save</button>
    </div>
  </form>
</Modal>

<!-- Manage User Modal -->
<Modal show={activeSection === 'manage-user'} title="Manage Users" on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-2">
    {#each $users as u}
      <div class="flex items-center justify-between py-2 border-b border-hazardo-lightGray/30">
        <span class="text-sm {$selectedUser?.user_id === u.user_id ? 'text-hazardo-primary font-semibold' : ''}">{u.user_name}</span>
        <button class="p-1 text-hazardo-lightGray hover:text-red-500" on:click={() => deleteUser(u.user_id)}>
          <Icon name="trash" size={14} />
        </button>
      </div>
    {/each}
  </div>
</Modal>

<!-- LLM Settings Modal -->
<Modal show={activeSection === 'manage-llm'} title="Manage LLM Model (AI)" width="w-96" on:close={() => activeSection = ''}>
  <form on:submit|preventDefault={saveLlmSettings} class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label="Provider:" />
      <SelectDropdown options={llmProviderOptions} bind:selected={llmProvider} placeholder="" />
    </div>
    {#if llmProvider === 'openai'}
      <div class="flex flex-col">
        <FormLabel label="OpenAI API Key:" />
        <div class="flex gap-2">
          <div class="flex-1 relative">
            <FormInput bind:value={llmApiKey} placeholder="sk-..." type="password" />
            {#if llmApiKey}
              <button type="button" class="absolute right-2 top-1/2 -translate-y-1/2 text-hazardo-lightGray hover:text-red-500 text-sm" on:click={() => { llmApiKey = ''; llmApiKeys['openai'] = ''; }}>✕</button>
            {/if}
          </div>
          <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" on:click={testOpenAIConnection} disabled={openaiTestStatus === 'testing' || !llmApiKey.trim()}>
            {openaiTestStatus === 'testing' ? '...' : 'Test'}
          </button>
        </div>
        {#if openaiTestMsg}
          <p class="text-xs mt-1 {openaiTestStatus === 'success' ? 'text-green-600' : 'text-red-500'}">{openaiTestMsg}</p>
        {/if}
      </div>
      <div class="flex flex-col">
        <FormLabel label="Model:" />
        {#if openaiModels.length > 0}
          <SelectDropdown options={openaiModels} bind:selected={llmModel} placeholder="Select model..." />
        {:else}
          <FormInput bind:value={llmModel} placeholder="gpt-4o-mini" />
        {/if}
      </div>
    {:else if llmProvider === 'gemini'}
      <div class="flex flex-col">
        <FormLabel label="Gemini API Key:" />
        <div class="flex gap-2">
          <div class="flex-1 relative">
            <FormInput bind:value={llmApiKey} placeholder="Enter Gemini API key..." type="password" />
            {#if llmApiKey}
              <button type="button" class="absolute right-2 top-1/2 -translate-y-1/2 text-hazardo-lightGray hover:text-red-500 text-sm" on:click={() => { llmApiKey = ''; llmApiKeys['gemini'] = ''; }}>✕</button>
            {/if}
          </div>
          <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" on:click={testGeminiConnection} disabled={geminiTestStatus === 'testing' || !llmApiKey.trim()}>
            {geminiTestStatus === 'testing' ? '...' : 'Test'}
          </button>
        </div>
        {#if geminiTestMsg}
          <p class="text-xs mt-1 {geminiTestStatus === 'success' ? 'text-green-600' : 'text-red-500'}">{geminiTestMsg}</p>
        {/if}
      </div>
      <div class="flex flex-col">
        <FormLabel label="Model:" />
        {#if geminiModels.length > 0}
          <SelectDropdown options={geminiModels} bind:selected={llmModel} placeholder="Select model..." />
        {:else}
          <FormInput bind:value={llmModel} placeholder="gemini-2.0-flash" />
        {/if}
      </div>
    {:else if llmProvider === 'local'}
      <div class="flex flex-col">
        <FormLabel label="Endpoint URL:" />
        <div class="flex gap-2">
          <div class="flex-1">
            <FormInput bind:value={llmEndpoint} placeholder="http://localhost:11434" />
          </div>
          <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" on:click={testOllamaConnection} disabled={ollamaTestStatus === 'testing'}>
            {ollamaTestStatus === 'testing' ? '...' : 'Test'}
          </button>
        </div>
        {#if ollamaTestMsg}
          <p class="text-xs mt-1 {ollamaTestStatus === 'success' ? 'text-green-600' : 'text-red-500'}">{ollamaTestMsg}</p>
        {/if}
      </div>
      {#if ollamaModels.length > 0}
        <div class="flex flex-col">
          <FormLabel label="Model:" />
          <SelectDropdown options={ollamaModels} bind:selected={llmModel} placeholder="Select model..." />
        </div>
      {:else}
        <div class="flex flex-col">
          <FormLabel label="Model (manual):" />
          <FormInput bind:value={llmModel} placeholder="llama3.2" />
        </div>
      {/if}
    {/if}
    <div class="flex flex-col">
      <FormLabel label="System Prompt:" />
      <textarea
        bind:value={chatPromptFormat}
        class="border rounded p-2 border-hazardo-lightGray focus:outline-hazardo-accent w-full text-sm resize-none"
        rows="4"
      ></textarea>
    </div>
    <label class="flex items-center gap-2 text-sm">
      <input type="checkbox" checked={chatAutoCreate === 'true'} on:change={(e) => chatAutoCreate = e.currentTarget.checked ? 'true' : 'false'} class="w-4 h-4 accent-hazardo-accent" />
      Allow AI to create items/categories
    </label>
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => activeSection = ''}>Cancel</button>
      <button type="submit" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm">Save</button>
    </div>
  </form>
</Modal>

<!-- Recycle Bin Modal -->
<Modal show={activeSection === 'recycle-bin'} title="Recycle Bin" width="w-96" on:close={() => activeSection = ''}>
  <div class="flex gap-2 mb-3">
    <button
      class="px-3 py-1 rounded text-sm {recycleBinTab === 'vault' ? 'bg-hazardo-accent text-white' : 'border border-hazardo-lightGray'}"
      on:click={() => recycleBinTab = 'vault'}
    >Vault</button>
    <button
      class="px-3 py-1 rounded text-sm {recycleBinTab === 'categories' ? 'bg-hazardo-accent text-white' : 'border border-hazardo-lightGray'}"
      on:click={() => recycleBinTab = 'categories'}
    >Categories</button>
    <button
      class="px-3 py-1 rounded text-sm {recycleBinTab === 'picked' ? 'bg-hazardo-accent text-white' : 'border border-hazardo-lightGray'}"
      on:click={() => recycleBinTab = 'picked'}
    >Picked</button>
  </div>

  <div class="mb-3">
    <SearchBar bind:value={recycleSearch} />
  </div>

  <div class="max-h-60 overflow-y-auto" bind:this={recycleContainer}>
    {#if recycleBinTab === 'vault'}
      {#each deletedItems.filter(i => !recycleSearch || i.item_name.toLowerCase().includes(recycleSearch.toLowerCase())) as item}
        <div class="flex items-center justify-between py-2 border-b border-hazardo-lightGray/30">
          <span class="text-sm">{item.item_name}</span>
          <div class="flex gap-2">
            <button class="p-1 text-hazardo-accent hover:text-hazardo-primary" title="Restore" on:click={() => restoreItem(item.item_id)}>
              <Icon name="sync" size={20} />
            </button>
            <button class="p-1 text-hazardo-lightGray hover:text-red-500" title="Delete permanently" on:click={() => permanentDeleteItem(item.item_id)}>
              <Icon name="trash" size={20} />
            </button>
          </div>
        </div>
      {/each}
    {:else if recycleBinTab === 'categories'}
      {#each deletedCategories.filter(c => !recycleSearch || c.category_name.toLowerCase().includes(recycleSearch.toLowerCase())) as cat}
        <div class="flex items-center justify-between py-2 border-b border-hazardo-lightGray/30">
          <span class="text-sm">{cat.category_name}</span>
          <div class="flex gap-2">
            <button class="p-1 text-hazardo-accent hover:text-hazardo-primary" title="Restore" on:click={() => restoreCategory(cat.category_id)}>
              <Icon name="sync" size={20} />
            </button>
            <button class="p-1 text-hazardo-lightGray hover:text-red-500" title="Delete permanently" on:click={() => permanentDeleteCategory(cat.category_id)}>
              <Icon name="trash" size={20} />
            </button>
          </div>
        </div>
      {/each}
    {:else}
      {#each deletedPicks.filter(p => !recycleSearch || p.item_name.toLowerCase().includes(recycleSearch.toLowerCase())) as pick}
        <div class="flex items-center justify-between py-2 border-b border-hazardo-lightGray/30">
          <div class="flex items-center gap-2 text-sm min-w-0">
            <span class="text-hazardo-text whitespace-nowrap">{formatDate(pick.pick_date)}</span>
            <span class="text-hazardo-lightGray">-</span>
            <Icon name={pick.category_icon} size={14} />
            <span class="truncate">{pick.item_name}</span>
          </div>
          <div class="flex gap-2 shrink-0 ml-2">
            <button class="p-1 text-hazardo-accent hover:text-hazardo-primary" title="Restore" on:click={() => restorePick(pick.pick_id)}>
              <Icon name="sync" size={20} />
            </button>
            <button class="p-1 text-hazardo-lightGray hover:text-red-500" title="Delete permanently" on:click={() => permanentDeletePick(pick.pick_id)}>
              <Icon name="trash" size={20} />
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</Modal>

<!-- Documentation Modal -->
<Modal show={activeSection === 'documentation'} title="Documentation" on:close={() => activeSection = ''}>
  <div class="text-sm leading-relaxed flex flex-col gap-3">
    <p><strong>Hazardo</strong> is a random activity picker app. Add items to categories in the Vault, set your preferences, and roll the dice!</p>
    <p><strong>Home:</strong> Set time, vibe, and category preferences then roll to get a random pick.</p>
    <p><strong>Vault:</strong> Manage your categories and items. Each item can have time/vibe preferences.</p>
    <p><strong>Picked:</strong> View your history of all picked activities.</p>
    <p><strong>ChatBot:</strong> AI-powered assistant (requires LLM configuration in Settings).</p>
    <p><strong>Advance Roll:</strong> When enabled, the AI provides recommendations alongside your roll.</p>
  </div>
</Modal>

<!-- Theme Modal -->
<Modal show={activeSection === 'theme'} title="Theme" on:close={() => activeSection = ''}>
  <p class="text-sm text-hazardo-lightGray">Theme follows your system settings automatically. Custom themes coming soon!</p>
</Modal>

<!-- Language Modal -->
<Modal show={activeSection === 'language'} title="Language" on:close={() => activeSection = ''}>
  <p class="text-sm text-hazardo-lightGray">Currently only English is supported. More languages coming soon!</p>
</Modal>

<!-- Sync Modal -->
<Modal show={activeSection === 'manage-sync'} title="Manage Synchronization" on:close={() => activeSection = ''}>
  <p class="text-sm text-hazardo-lightGray">Synchronization requires the Hazardo desktop API server. Configure and start the Docker container on your network to enable sync.</p>
</Modal>

<!-- Location Modal -->
<Modal show={activeSection === 'manage-location'} title="Location Services" on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-4">
    <!-- Permission status -->
    <div>
      <p class="text-sm font-medium mb-2">Location Permission</p>
      {#if locationPermissionStatus === 'granted' || locationEnabled}
        <div class="flex items-center gap-2 text-sm text-green-600">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          Location access enabled
        </div>
      {:else if locationPermissionStatus === 'requesting'}
        <div class="flex items-center gap-2 text-sm text-hazardo-lightGray">
          <div class="w-4 h-4 border-2 border-hazardo-accent border-t-transparent rounded-full animate-spin"></div>
          Requesting permission...
        </div>
      {:else}
        <button type="button" class="flex items-center gap-2 px-4 py-2 rounded-lg bg-hazardo-accent text-white text-sm font-medium" on:click={requestLocationPermission}>
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg>
          Allow Location Access
        </button>
        {#if locationPermissionStatus === 'denied'}
          <p class="text-xs text-red-500 mt-1">Permission denied. You may need to enable it in your device settings.</p>
        {/if}
      {/if}
    </div>

    <!-- Location override -->
    <div>
      <p class="text-sm font-medium mb-2">Roll Dice Location</p>
      <p class="text-xs text-hazardo-lightGray mb-2">Choose a specific location for roll dice, or use your current GPS position.</p>
      {#if locationOverride}
        <div class="flex items-center justify-between bg-hazardo-background/50 rounded-lg px-3 py-2 border border-hazardo-lightGray/30">
          <span class="text-sm">{locationOverride.split('|')[0]}</span>
          <button type="button" class="text-xs text-red-500 hover:text-red-700" on:click={clearLocationOverride}>Remove</button>
        </div>
      {:else}
        <div class="flex items-center gap-2 text-sm text-hazardo-lightGray mb-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
          Using current GPS location (default)
        </div>
      {/if}
      <div class="flex gap-2 mt-2">
        <div class="flex-1">
          <FormInput bind:value={locationSearch} placeholder="Search a city..." on:keydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); searchLocation(); } }} />
        </div>
        <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" disabled={locationSearching || !locationSearch.trim()} on:click={searchLocation}>
          {locationSearching ? '...' : 'Search'}
        </button>
      </div>
      {#if locationSearchResults.length > 0}
        <div class="mt-2 border border-hazardo-lightGray/30 rounded-lg overflow-hidden max-h-40 overflow-y-auto">
          {#each locationSearchResults as result}
            <button type="button" class="w-full text-left px-3 py-2 text-sm hover:bg-hazardo-accent/10 border-b border-hazardo-lightGray/20 last:border-b-0" on:click={() => selectLocationOverride(result)}>
              {result.name}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => activeSection = ''}>Cancel</button>
      <button type="button" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm" on:click={saveLocationSettings}>Save</button>
    </div>
  </div>
</Modal>

<!-- Import Modal -->
<Modal show={activeSection === 'bulk-import'} title="Bulk Import" width="w-96" on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label="Import Format:" />
      <SelectDropdown options={exportFormatOptions} bind:selected={importFormat} placeholder="" />
    </div>
    <div class="flex flex-col">
      <FormLabel label="File:" />
      <input type="file" accept=".json,.csv,.md,.txt" class="text-sm" on:change={handleFileSelect} bind:this={importFileInput} />
    </div>
    {#if importFileContent}
      <div class="border border-hazardo-lightGray rounded p-2">
        <p class="text-xs text-hazardo-lightGray mb-1">Preview ({importFileContent.length} chars)</p>
        <pre class="text-xs text-hazardo-text max-h-32 overflow-auto whitespace-pre-wrap">{importFileContent.slice(0, 500)}{importFileContent.length > 500 ? '...' : ''}</pre>
      </div>
    {/if}
    <p class="text-xs text-hazardo-lightGray">Or paste content directly:</p>
    <textarea
      bind:value={importFileContent}
      placeholder="Paste JSON/CSV/Markdown content..."
      class="border rounded p-2 border-hazardo-lightGray focus:outline-hazardo-accent w-full text-xs resize-none font-mono"
      rows="4"
    ></textarea>
    {#if importResult}
      <p class="text-sm {importResult.startsWith('Import error') ? 'text-red-500' : 'text-green-600'}">{importResult}</p>
    {/if}
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => activeSection = ''}>Cancel</button>
      <button type="button" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm disabled:opacity-50" disabled={importLoading || !importFileContent.trim()} on:click={handleImport}>
        {importLoading ? 'Importing...' : 'Import'}
      </button>
    </div>
  </div>
</Modal>

<!-- Export Modal -->
<Modal show={activeSection === 'bulk-export'} title="Bulk Export" width="w-96" on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-4">
    <!-- User selection -->
    <div class="flex flex-col">
      <FormLabel label="Select Users:" />
      <label class="flex items-center gap-2 text-sm py-1">
        <input type="checkbox" bind:checked={exportAllUsers} class="w-4 h-4 accent-hazardo-accent" />
        All Users
      </label>
      {#if !exportAllUsers}
        {#each $users as u}
          <label class="flex items-center gap-2 text-sm py-1 pl-4">
            <input type="checkbox" bind:checked={exportUserSelection[u.user_id]} class="w-4 h-4 accent-hazardo-accent" />
            {u.user_name}
          </label>
        {/each}
      {/if}
    </div>
    <!-- Category selection -->
    {#if exportCategories.length > 0}
      <div class="flex flex-col">
        <FormLabel label="Select Categories:" />
        <label class="flex items-center gap-2 text-sm py-1">
          <input type="checkbox" bind:checked={exportAllCategories} class="w-4 h-4 accent-hazardo-accent" />
          All Categories
        </label>
        {#if !exportAllCategories}
          <div class="max-h-32 overflow-y-auto pl-4">
            {#each exportCategories as cat}
              <label class="flex items-center gap-2 text-sm py-1">
                <input type="checkbox" bind:checked={exportCategorySelection[cat.category_id]} class="w-4 h-4 accent-hazardo-accent" />
                <Icon name={cat.category_icon} size={14} />
                {cat.category_name}
              </label>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
    <!-- Format -->
    <div class="flex flex-col">
      <FormLabel label="Export Format:" />
      <SelectDropdown options={exportFormatOptions} bind:selected={exportFormat} placeholder="" />
    </div>
    <!-- Include picked items -->
    <label class="flex items-center gap-2 text-sm">
      <input type="checkbox" checked={true} disabled class="w-4 h-4 accent-hazardo-accent" />
      Include Picked Items (roll history)
    </label>
    <!-- Export button -->
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => activeSection = ''}>Cancel</button>
      <button type="button" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm disabled:opacity-50" disabled={exportLoading || selectedExportUserIds.length === 0} on:click={handleExport}>
        {exportLoading ? 'Exporting...' : 'Export'}
      </button>
    </div>
    <!-- Export result -->
    {#if exportResult}
      <div class="border border-hazardo-lightGray rounded p-2">
        {#if exportFormat === 'zip'}
          <p class="text-sm text-green-600">{exportResult}</p>
          <div class="flex gap-2 mt-2">
            <button class="w-8 h-8 rounded-full bg-hazardo-accent/10 text-hazardo-accent hover:bg-hazardo-accent/20 flex items-center justify-center transition-colors disabled:opacity-50" on:click={shareExport} disabled={exportSharing} title="Share">
              <Icon name="share" size={16} />
            </button>
          </div>
        {:else}
          <div class="flex items-center justify-between mb-1">
            <p class="text-xs text-hazardo-lightGray">Export Result</p>
            <div class="flex gap-2">
              <button class="w-8 h-8 rounded-full bg-hazardo-accent/10 text-hazardo-accent hover:bg-hazardo-accent/20 flex items-center justify-center transition-colors" on:click={copyExportResult} title="Copy">
                <Icon name="copy" size={16} />
              </button>
              <button class="w-8 h-8 rounded-full bg-hazardo-accent/10 text-hazardo-accent hover:bg-hazardo-accent/20 flex items-center justify-center transition-colors disabled:opacity-50" on:click={saveExportToFile} disabled={exportSaving} title="Save to Downloads">
                <Icon name="download" size={16} />
              </button>
              <button class="w-8 h-8 rounded-full bg-hazardo-accent/10 text-hazardo-accent hover:bg-hazardo-accent/20 flex items-center justify-center transition-colors disabled:opacity-50" on:click={shareExport} disabled={exportSharing} title="Share">
                <Icon name="share" size={16} />
              </button>
            </div>
          </div>
          <pre class="text-xs text-hazardo-text max-h-48 overflow-auto whitespace-pre-wrap font-mono">{exportResult.slice(0, 2000)}{exportResult.length > 2000 ? '\n...(truncated for display)' : ''}</pre>
        {/if}
        {#if exportFilePath}
          <p class="text-xs mt-2 {exportFilePath.startsWith('Error') ? 'text-red-500' : 'text-green-600'}">
            {exportFilePath.startsWith('Error') ? exportFilePath : `Saved to: ${exportFilePath}`}
          </p>
        {/if}
      </div>
    {/if}
  </div>
</Modal>
