<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
  import { selectedUser, users, loadUsers, updateUserName } from '../../stores/userStore';
  import Title from '../../components/atoms/Title.svelte';
  import Icon from '../../components/atoms/Icon.svelte';
  import FormLabel from '../../components/atoms/FormLabel.svelte';
  import FormInput from '../../components/atoms/FormInput.svelte';
  import Modal from '../../components/organisms/Modal.svelte';
  import SelectDropdown from '../../components/molecules/SelectDropdown.svelte';
  import SearchBar from '../../components/molecules/SearchBar.svelte';
  import ScrollToTop from '../../components/atoms/ScrollToTop.svelte';
  import { showToast } from '../../stores/toastStore';
  import { t, currentLang, formatDateLocalized, saveLanguageSetting, type Lang } from '../../stores/i18nStore';
  import { currentTheme, saveThemeSetting, type Theme } from '../../stores/themeStore';
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
  let editingUserId: number | null = null;
  let editingUserName = '';

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
  let exportIncludePicks = false;
  let exportPicks: PickType[] = [];
  let exportPickSelection: Record<number, boolean> = {};
  let exportAllPicks = false;
  const exportFormatOptions = [
    { value: 'json', label: 'JSON' },
    { value: 'csv', label: 'CSV' },
    { value: 'markdown', label: 'Markdown' },
    { value: 'zip', label: 'ZIP (with images)' },
  ];

  // Import
  let importFormat = 'json';
  let importFileContent = '';
  let importZipBase64 = '';
  let importResult = '';
  let importLoading = false;
  let importFileInput: HTMLInputElement;
  $: importAccept = importFormat === 'zip' ? '.zip' : importFormat === 'markdown' ? '.md,.txt' : importFormat === 'csv' ? '.csv,.txt' : '.json';
  $: if (importFormat) { importFileContent = ''; importZipBase64 = ''; if (importFileInput) importFileInput.value = ''; }

  $: llmProviderOptions = [
    { value: '', label: $t('llm.not_configured') },
    { value: 'openai', label: $t('llm.openai_label') },
    { value: 'gemini', label: $t('llm.gemini_label') },
    { value: 'local', label: $t('llm.local_label') },
  ];

  const settingsSections = [
    { key: 'documentation', label: 'Documentation', icon: 'file-text' as const },
    { key: 'recycle-bin', label: 'Recycle Bin', icon: 'recycle' as const },
    { key: 'bulk-import', label: 'Bulk Import', icon: 'import' as const },
    { key: 'bulk-export', label: 'Bulk Export', icon: 'export' as const },
    { key: 'theme', label: 'theme_label', extra: 'theme_extra', icon: 'palette' as const },
    { key: 'language', label: 'language_label', extra: 'language_extra', icon: 'globe' as const },
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

  $: {
    if (selectedExportUserIds.length > 0 && activeSection === 'bulk-export') {
      loadExportPicks(selectedExportUserIds);
    }
  }

  $: {
    if (exportAllPicks) {
      exportPicks.forEach(p => exportPickSelection[p.pick_id] = true);
    }
    exportPickSelection = exportPickSelection;
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

  async function loadExportPicks(uids: number[]) {
    let allPicks: PickType[] = [];
    for (const uid of uids) {
      try {
        const p = await invoke<PickType[]>('get_picks', { userId: uid, categoryId: null });
        allPicks = [...allPicks, ...p];
      } catch (_) {}
    }
    exportPicks = allPicks;
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
      // No geolocation API — try IP-based fallback
      ipFallbackPermission();
      return;
    }
    locationPermissionStatus = 'requesting';
    navigator.geolocation.getCurrentPosition(
      () => {
        locationPermissionStatus = 'granted';
        locationEnabled = true;
        saveSetting('location_enabled', 'true');
      },
      () => {
        // GPS failed — try IP-based fallback
        ipFallbackPermission();
      },
      { enableHighAccuracy: true, timeout: 15000 }
    );
  }

  async function ipFallbackPermission() {
    try {
      const resp = await tauriFetch('https://ipwho.is/', { method: 'GET' });
      if (resp.ok) {
        const data = await resp.json();
        if (data.success !== false && data.latitude && data.longitude) {
          locationPermissionStatus = 'granted';
          locationEnabled = true;
          saveSetting('location_enabled', 'true');
          return;
        }
      }
    } catch (_) {}
    locationPermissionStatus = 'denied';
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
      exportPickSelection = {};
      exportResult = '';
      exportAllUsers = false;
      exportAllCategories = false;
      exportAllPicks = false;
      exportIncludePicks = false;
      exportPicks = [];
    }
    if (key === 'bulk-import') {
      importResult = '';
      importFileContent = '';
      importZipBase64 = '';
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
        showToast(`Export saved to: ${exportFilePath}`, 'success');
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
      const mime = exportFormat === 'csv' ? 'text/csv' : exportFormat === 'markdown' ? 'text/markdown' : 'application/json';
      const filename = `hazardo_export_${Date.now()}.${ext}`;
      const blob = new Blob([exportResult], { type: mime });
      const file = new File([blob], filename, { type: mime });
      if (navigator.canShare && navigator.canShare({ files: [file] })) {
        await navigator.share({ files: [file], title: 'Hazardo Export' });
        showToast('Export shared!', 'success');
      } else {
        // Fallback: save to Downloads
        exportFilePath = await invoke<string>('save_export_file', { content: exportResult, filename, target: 'downloads' });
        showToast(`File saved to: ${exportFilePath}`, 'success');
      }
      activeSection = '';
    } catch (e: any) {
      if (e.name !== 'AbortError') {
        exportFilePath = `Error: ${e.message || e}`;
      }
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
    if (importFormat === 'zip') {
      const reader = new FileReader();
      reader.onload = () => {
        const arrayBuf = reader.result as ArrayBuffer;
        const bytes = new Uint8Array(arrayBuf);
        let binary = '';
        for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
        importZipBase64 = btoa(binary);
        importFileContent = `[ZIP file: ${file.name} — ${(bytes.length / 1024).toFixed(1)} KB]`;
      };
      reader.readAsArrayBuffer(file);
    } else {
      importZipBase64 = '';
      const reader = new FileReader();
      reader.onload = () => { importFileContent = reader.result as string; };
      reader.readAsText(file);
    }
  }

  async function handleImport() {
    if (!importFileContent.trim() || !$selectedUser) return;
    importLoading = true;
    importResult = '';
    try {
      if (importFormat === 'zip') {
        if (!importZipBase64) { importResult = 'Import error: No ZIP file selected'; importLoading = false; return; }
        importResult = await invoke<string>('import_zip', { userId: $selectedUser.user_id, zipBase64: importZipBase64 });
      } else {
        let jsonData: string;
        if (importFormat === 'json') {
          jsonData = importFileContent;
        } else if (importFormat === 'csv') {
          jsonData = JSON.stringify(csvToJson(importFileContent));
        } else {
          jsonData = JSON.stringify(markdownToJson(importFileContent));
        }
        importResult = await invoke<string>('import_data', { userId: $selectedUser.user_id, data: jsonData });
      }
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
    return formatDateLocalized(dateStr, $currentLang);
  }

  function getSectionLabel(section: any, _t: Function): string {
    const map: Record<string, string> = {
      'documentation': _t('settings.documentation'),
      'recycle-bin': _t('settings.recycle_bin'),
      'bulk-import': _t('settings.bulk_import'),
      'bulk-export': _t('settings.bulk_export'),
      'theme': _t('settings.theme'),
      'language': _t('settings.language'),
      'device-name': _t('settings.device_name'),
      'manage-user': _t('settings.manage_user'),
      'manage-sync': _t('settings.manage_sync'),
      'manage-location': _t('settings.location_services'),
      'manage-llm': _t('settings.manage_llm'),
    };
    return map[section.key] || section.label;
  }

  function getSectionExtra(section: any): string {
    if (section.extra === 'theme_extra') {
      return `(${$currentTheme === 'dark' ? $t('settings.theme_dark') : $t('settings.theme_light')})`;
    }
    if (section.extra === 'language_extra') {
      return `(${$currentLang === 'fr' ? $t('settings.french') : $t('settings.english')})`;
    }
    return section.extra || '';
  }

  async function handleThemeChange(theme: Theme) {
    if (!$selectedUser) return;
    await saveThemeSetting($selectedUser.user_id, theme);
  }

  async function handleLanguageChange(lang: Lang) {
    if (!$selectedUser) return;
    await saveLanguageSetting($selectedUser.user_id, lang);
  }
</script>

<main class="max-w-lg mx-auto px-4">
  <div class="flex flex-col items-center">
    <div class="mt-6 mb-6 bg-hazardo-accent px-2 rounded">
      <Title title={$t('settings.title')} />
    </div>
  </div>

  <div class="flex flex-col">
    {#each settingsSections as section}
      <button
        class="flex items-center justify-between py-4 border-b border-hazardo-lightGray/30 text-left hover:bg-hazardo-background/50 px-2 rounded transition-colors"
        on:click={() => openSection(section.key)}
      >
        <span class="text-sm font-medium">
          {getSectionLabel(section, $t)}
          {#if section.extra}
            <span class="text-hazardo-lightGray font-normal">{getSectionExtra(section)}</span>
          {/if}
        </span>
        <Icon name="chevron-right" size={16} />
      </button>
    {/each}
  </div>
</main>

<!-- Device Name Modal -->
<Modal show={activeSection === 'device-name'} title={$t('settings.device_name')} on:close={() => activeSection = ''}>
  <form on:submit|preventDefault={saveDeviceName} class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label={$t('settings.device_name') + ':'} />
      <FormInput bind:value={editDeviceName} placeholder="..." />
    </div>
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm text-hazardo-text" on:click={() => activeSection = ''}>{$t('settings.cancel')}</button>
      <button type="submit" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm">{$t('settings.save')}</button>
    </div>
  </form>
</Modal>

<!-- Manage User Modal -->
<Modal show={activeSection === 'manage-user'} title={$t('settings.manage_user')} on:close={() => { activeSection = ''; editingUserId = null; }}>
  <div class="flex flex-col gap-2">
    {#each $users as u}
      <div class="flex items-center justify-between py-2 border-b border-hazardo-lightGray/30">
        {#if editingUserId === u.user_id}
          <form class="flex items-center gap-2 flex-1" on:submit|preventDefault={async () => { if (editingUserName.trim()) { await updateUserName(u.user_id, editingUserName.trim()); editingUserId = null; } }}>
            <input type="text" bind:value={editingUserName} class="border rounded px-2 py-1 text-sm flex-1 border-hazardo-lightGray focus:outline-hazardo-accent bg-hazardo-surface text-hazardo-text" />
            <button type="submit" class="px-2 py-1 rounded bg-hazardo-primary text-white text-xs">{$t('settings.save')}</button>
            <button type="button" class="px-2 py-1 rounded border border-hazardo-lightGray text-xs text-hazardo-text" on:click={() => editingUserId = null}>{$t('settings.cancel')}</button>
          </form>
        {:else}
          <span class="text-sm {$selectedUser?.user_id === u.user_id ? 'text-hazardo-primary font-semibold' : ''}">{u.user_name}</span>
          <div class="flex items-center gap-1">
            <button class="p-1 text-hazardo-lightGray hover:text-hazardo-accent" on:click={() => { editingUserId = u.user_id; editingUserName = u.user_name; }}>
              <Icon name="edit" size={14} />
            </button>
            <button class="p-1 text-hazardo-lightGray hover:text-red-500" on:click={() => deleteUser(u.user_id)}>
              <Icon name="trash" size={14} />
            </button>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</Modal>

<!-- LLM Settings Modal -->
<Modal show={activeSection === 'manage-llm'} title={$t('settings.manage_llm')} width="w-96" on:close={() => activeSection = ''}>
  <form on:submit|preventDefault={saveLlmSettings} class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label={$t('llm.provider')} />
      <SelectDropdown options={llmProviderOptions} bind:selected={llmProvider} placeholder="" />
    </div>
    {#if llmProvider === 'openai'}
      <div class="flex flex-col">
        <FormLabel label={$t('llm.api_key_openai')} />
        <div class="flex gap-2">
          <div class="flex-1 relative">
            <FormInput bind:value={llmApiKey} placeholder={$t('llm.api_key_ph')} type="password" />
            {#if llmApiKey}
              <button type="button" class="absolute right-2 top-1/2 -translate-y-1/2 text-hazardo-lightGray hover:text-red-500 text-sm" on:click={() => { llmApiKey = ''; llmApiKeys['openai'] = ''; }}>✕</button>
            {/if}
          </div>
          <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" on:click={testOpenAIConnection} disabled={openaiTestStatus === 'testing' || !llmApiKey.trim()}>
            {openaiTestStatus === 'testing' ? '...' : $t('llm.test')}
          </button>
        </div>
        {#if openaiTestMsg}
          <p class="text-xs mt-1 {openaiTestStatus === 'success' ? 'text-green-600' : 'text-red-500'}">{openaiTestMsg}</p>
        {/if}
      </div>
      <div class="flex flex-col">
        <FormLabel label={$t('llm.model')} />
        {#if openaiModels.length > 0}
          <SelectDropdown options={openaiModels} bind:selected={llmModel} placeholder={$t('llm.model_ph')} />
        {:else}
          <FormInput bind:value={llmModel} placeholder="gpt-4o-mini" />
        {/if}
      </div>
    {:else if llmProvider === 'gemini'}
      <div class="flex flex-col">
        <FormLabel label={$t('llm.api_key_gemini')} />
        <div class="flex gap-2">
          <div class="flex-1 relative">
            <FormInput bind:value={llmApiKey} placeholder={$t('llm.api_key_ph')} type="password" />
            {#if llmApiKey}
              <button type="button" class="absolute right-2 top-1/2 -translate-y-1/2 text-hazardo-lightGray hover:text-red-500 text-sm" on:click={() => { llmApiKey = ''; llmApiKeys['gemini'] = ''; }}>✕</button>
            {/if}
          </div>
          <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" on:click={testGeminiConnection} disabled={geminiTestStatus === 'testing' || !llmApiKey.trim()}>
            {geminiTestStatus === 'testing' ? '...' : $t('llm.test')}
          </button>
        </div>
        {#if geminiTestMsg}
          <p class="text-xs mt-1 {geminiTestStatus === 'success' ? 'text-green-600' : 'text-red-500'}">{geminiTestMsg}</p>
        {/if}
      </div>
      <div class="flex flex-col">
        <FormLabel label={$t('llm.model')} />
        {#if geminiModels.length > 0}
          <SelectDropdown options={geminiModels} bind:selected={llmModel} placeholder={$t('llm.model_ph')} />
        {:else}
          <FormInput bind:value={llmModel} placeholder="gemini-2.0-flash" />
        {/if}
      </div>
    {:else if llmProvider === 'local'}
      <div class="flex flex-col">
        <FormLabel label={$t('llm.endpoint')} />
        <div class="flex gap-2">
          <div class="flex-1">
            <FormInput bind:value={llmEndpoint} placeholder={$t('llm.endpoint_ph')} />
          </div>
          <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" on:click={testOllamaConnection} disabled={ollamaTestStatus === 'testing'}>
            {ollamaTestStatus === 'testing' ? '...' : $t('llm.test')}
          </button>
        </div>
        {#if ollamaTestMsg}
          <p class="text-xs mt-1 {ollamaTestStatus === 'success' ? 'text-green-600' : 'text-red-500'}">{ollamaTestMsg}</p>
        {/if}
      </div>
      {#if ollamaModels.length > 0}
        <div class="flex flex-col">
          <FormLabel label={$t('llm.model')} />
          <SelectDropdown options={ollamaModels} bind:selected={llmModel} placeholder={$t('llm.model_ph')} />
        </div>
      {:else}
        <div class="flex flex-col">
          <FormLabel label={$t('llm.model_manual')} />
          <FormInput bind:value={llmModel} placeholder="llama3.2" />
        </div>
      {/if}
    {/if}
    <div class="flex flex-col">
      <FormLabel label={$t('llm.system_prompt')} />
      <textarea
        bind:value={chatPromptFormat}
        class="border rounded p-2 border-hazardo-lightGray focus:outline-hazardo-accent w-full text-sm resize-none bg-hazardo-surface text-hazardo-text"
        rows="4"
      ></textarea>
    </div>
    <label class="flex items-center gap-2 text-sm">
      <input type="checkbox" checked={chatAutoCreate === 'true'} on:change={(e) => chatAutoCreate = e.currentTarget.checked ? 'true' : 'false'} class="w-4 h-4 accent-hazardo-accent" />
      {$t('llm.allow_create')}
    </label>
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm text-hazardo-text" on:click={() => activeSection = ''}>{$t('settings.cancel')}</button>
      <button type="submit" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm">{$t('settings.save')}</button>
    </div>
  </form>
</Modal>

<!-- Recycle Bin Modal -->
<Modal show={activeSection === 'recycle-bin'} title={$t('settings.recycle_bin')} width="w-96" on:close={() => activeSection = ''}>
  <div class="flex gap-2 mb-3">
    <button
      class="px-3 py-1 rounded text-sm {recycleBinTab === 'vault' ? 'bg-hazardo-accent text-white' : 'border border-hazardo-lightGray text-hazardo-text'}"
      on:click={() => recycleBinTab = 'vault'}
    >{$t('recycle.vault')}</button>
    <button
      class="px-3 py-1 rounded text-sm {recycleBinTab === 'categories' ? 'bg-hazardo-accent text-white' : 'border border-hazardo-lightGray text-hazardo-text'}"
      on:click={() => recycleBinTab = 'categories'}
    >{$t('recycle.categories')}</button>
    <button
      class="px-3 py-1 rounded text-sm {recycleBinTab === 'picked' ? 'bg-hazardo-accent text-white' : 'border border-hazardo-lightGray text-hazardo-text'}"
      on:click={() => recycleBinTab = 'picked'}
    >{$t('recycle.picked')}</button>
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
            <button class="p-1 text-hazardo-accent hover:text-hazardo-primary" title={$t('recycle.restore')} on:click={() => restoreItem(item.item_id)}>
              <Icon name="sync" size={20} />
            </button>
            <button class="p-1 text-hazardo-lightGray hover:text-red-500" title={$t('recycle.delete_permanent')} on:click={() => permanentDeleteItem(item.item_id)}>
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
            <button class="p-1 text-hazardo-accent hover:text-hazardo-primary" title={$t('recycle.restore')} on:click={() => restoreCategory(cat.category_id)}>
              <Icon name="sync" size={20} />
            </button>
            <button class="p-1 text-hazardo-lightGray hover:text-red-500" title={$t('recycle.delete_permanent')} on:click={() => permanentDeleteCategory(cat.category_id)}>
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
            <button class="p-1 text-hazardo-accent hover:text-hazardo-primary" title={$t('recycle.restore')} on:click={() => restorePick(pick.pick_id)}>
              <Icon name="sync" size={20} />
            </button>
            <button class="p-1 text-hazardo-lightGray hover:text-red-500" title={$t('recycle.delete_permanent')} on:click={() => permanentDeletePick(pick.pick_id)}>
              <Icon name="trash" size={20} />
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</Modal>

<!-- Documentation Modal -->
<Modal show={activeSection === 'documentation'} title={$t('settings.documentation')} width="w-96" on:close={() => activeSection = ''}>
  <div class="text-base leading-relaxed flex flex-col gap-2">
    <p class="text-hazardo-lightGray mb-1">{$t('doc.tap_section')}</p>

    <!-- Home -->
    <details class="border border-hazardo-lightGray/30 rounded-lg">
      <summary class="flex items-center gap-2 px-3 py-2 cursor-pointer font-medium">
        <Icon name="home" size={16} /> {$t('doc.home_title')}
      </summary>
      <div class="px-3 pb-3 text-sm text-hazardo-text space-y-1">
        <p>{$t('doc.home_desc')}</p>
        <p><strong>{$t('home.time_pref')}</strong> {$t('doc.home_time')}</p>
        <p><strong>{$t('home.vibe_pref')}</strong> {$t('doc.home_vibe')}</p>
        <p><strong>{$t('home.list_pref')}</strong> {$t('doc.home_list')}</p>
        <p><strong>{$t('home.date_pref')}</strong> {$t('doc.home_date')}</p>
        <p><strong>{$t('home.advance_roll')}:</strong> {$t('doc.home_advance')}</p>
        <p><strong>{$t('home.roll_dice')}:</strong> {$t('doc.home_roll')}</p>
        <p><strong>{$t('home.pick_this')}:</strong> {$t('doc.home_pick')}</p>
        <p><strong>{$t('home.roll_again')}:</strong> {$t('doc.home_roll_again')}</p>
      </div>
    </details>

    <!-- Vault -->
    <details class="border border-hazardo-lightGray/30 rounded-lg">
      <summary class="flex items-center gap-2 px-3 py-2 cursor-pointer font-medium">
        <Icon name="vault" size={16} /> {$t('doc.vault_title')}
      </summary>
      <div class="px-3 pb-3 text-sm text-hazardo-text space-y-1">
        <p>{$t('doc.vault_desc')}</p>
        <p><strong>{$t('recycle.categories')}:</strong> {$t('doc.vault_categories')}</p>
        <p><strong>{$t('vault.add_category')}:</strong> {$t('doc.vault_create_cat')}</p>
        <p><strong>{$t('vault.add_item')}:</strong> {$t('doc.vault_add_items')}</p>
        <ul class="list-disc pl-4">
          <li><strong>{$t('vault.item_name')}</strong> {$t('doc.vault_item_name')}</li>
          <li><strong>{$t('vault.time_pref')}</strong> {$t('doc.vault_item_time')}</li>
          <li><strong>{$t('vault.vibe_pref')}</strong> {$t('doc.vault_item_vibe')}</li>
          <li><strong>Notes:</strong> {$t('doc.vault_item_notes')}</li>
        </ul>
        <p>{$t('doc.vault_edit_delete')}</p>
      </div>
    </details>

    <!-- Picked -->
    <details class="border border-hazardo-lightGray/30 rounded-lg">
      <summary class="flex items-center gap-2 px-3 py-2 cursor-pointer font-medium">
        <Icon name="picked" size={16} /> {$t('doc.picked_title')}
      </summary>
      <div class="px-3 pb-3 text-sm text-hazardo-text space-y-1">
        <p>{$t('doc.picked_desc')}</p>
        <p><strong>{$t('picked.filter')}</strong> {$t('doc.picked_filter')}</p>
        <p>{$t('doc.picked_detail')}</p>
        <p><strong>{$t('home.ai_recommendation')}:</strong> {$t('doc.picked_ai')}</p>
        <p><strong>Notes:</strong> {$t('doc.picked_notes')}</p>
        <p><strong>Image:</strong> {$t('doc.picked_image')}</p>
      </div>
    </details>

    <!-- ChatBot -->
    <details class="border border-hazardo-lightGray/30 rounded-lg">
      <summary class="flex items-center gap-2 px-3 py-2 cursor-pointer font-medium">
        <Icon name="chatbot" size={16} /> {$t('doc.chatbot_title')}
      </summary>
      <div class="px-3 pb-3 text-sm text-hazardo-text space-y-1">
        <p>{$t('doc.chatbot_desc')}</p>
        <p><strong>{$t('chatbot.settings')}:</strong> {$t('doc.chatbot_requires')}</p>
        <p>{$t('doc.chatbot_chat')}</p>
        <p><strong>{$t('chatbot.model')}</strong> {$t('doc.chatbot_model')}</p>
        <p>{$t('doc.chatbot_auto_create')}</p>
        <p>{$t('doc.chatbot_clear')}</p>
      </div>
    </details>

    <!-- Settings -->
    <details class="border border-hazardo-lightGray/30 rounded-lg">
      <summary class="flex items-center gap-2 px-3 py-2 cursor-pointer font-medium">
        <Icon name="setting" size={16} /> {$t('doc.settings_title')}
      </summary>
      <div class="px-3 pb-3 text-sm text-hazardo-text space-y-1">
        <p><strong>{$t('settings.recycle_bin')}:</strong> {$t('doc.settings_recycle')}</p>
        <p><strong>{$t('settings.bulk_import')}:</strong> {$t('doc.settings_import')}</p>
        <p><strong>{$t('settings.bulk_export')}:</strong> {$t('doc.settings_export')}</p>
        <p><strong>{$t('settings.device_name')}:</strong> {$t('doc.settings_device')}</p>
        <p><strong>{$t('settings.manage_user')}:</strong> {$t('doc.settings_user')}</p>
        <p><strong>{$t('settings.manage_sync')}:</strong> {$t('doc.settings_sync')}</p>
        <p><strong>{$t('settings.location_services')}:</strong> {$t('doc.settings_location')}</p>
        <p><strong>{$t('settings.manage_llm')}:</strong> {$t('doc.settings_llm')}</p>
        <ul class="list-disc pl-4">
          <li><strong>OpenAI:</strong> {$t('doc.settings_llm_openai')}</li>
          <li><strong>Gemini:</strong> {$t('doc.settings_llm_gemini')}</li>
          <li><strong>Local LLM:</strong> {$t('doc.settings_llm_local')}</li>
          <li><strong>{$t('llm.system_prompt')}</strong> {$t('doc.settings_llm_prompt')}</li>
          <li>{$t('doc.settings_llm_auto')}</li>
        </ul>
      </div>
    </details>
  </div>
</Modal>

<!-- Theme Modal -->
<Modal show={activeSection === 'theme'} title={$t('settings.theme')} on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-3">
    <button
      class="flex items-center justify-between px-4 py-3 rounded-lg border transition-colors {$currentTheme === 'light' ? 'border-hazardo-accent bg-hazardo-accent/10' : 'border-hazardo-lightGray/30'}"
      on:click={() => handleThemeChange('light')}
    >
      <span class="text-sm font-medium">{$t('settings.theme_light')}</span>
      {#if $currentTheme === 'light'}
        <Icon name="check" size={16} />
      {/if}
    </button>
    <button
      class="flex items-center justify-between px-4 py-3 rounded-lg border transition-colors {$currentTheme === 'dark' ? 'border-hazardo-accent bg-hazardo-accent/10' : 'border-hazardo-lightGray/30'}"
      on:click={() => handleThemeChange('dark')}
    >
      <span class="text-sm font-medium">{$t('settings.theme_dark')}</span>
      {#if $currentTheme === 'dark'}
        <Icon name="check" size={16} />
      {/if}
    </button>
  </div>
</Modal>

<!-- Language Modal -->
<Modal show={activeSection === 'language'} title={$t('settings.language')} on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-3">
    <button
      class="flex items-center justify-between px-4 py-3 rounded-lg border transition-colors {$currentLang === 'en' ? 'border-hazardo-accent bg-hazardo-accent/10' : 'border-hazardo-lightGray/30'}"
      on:click={() => handleLanguageChange('en')}
    >
      <span class="text-sm font-medium">{$t('settings.english')}</span>
      {#if $currentLang === 'en'}
        <Icon name="check" size={16} />
      {/if}
    </button>
    <button
      class="flex items-center justify-between px-4 py-3 rounded-lg border transition-colors {$currentLang === 'fr' ? 'border-hazardo-accent bg-hazardo-accent/10' : 'border-hazardo-lightGray/30'}"
      on:click={() => handleLanguageChange('fr')}
    >
      <span class="text-sm font-medium">{$t('settings.french')}</span>
      {#if $currentLang === 'fr'}
        <Icon name="check" size={16} />
      {/if}
    </button>
  </div>
</Modal>

<!-- Sync Modal -->
<Modal show={activeSection === 'manage-sync'} title={$t('settings.manage_sync')} on:close={() => activeSection = ''}>
  <div class="flex flex-col items-center gap-3 py-4">
    <div class="w-12 h-12 rounded-full bg-hazardo-accent/10 flex items-center justify-center">
      <Icon name="sync" size={24} />
    </div>
    <span class="inline-block px-3 py-1 rounded-full bg-yellow-100 text-yellow-700 text-xs font-semibold">{$t('sync.in_development')}</span>
    <p class="text-sm text-hazardo-lightGray text-center">{$t('sync.description')}</p>
  </div>
</Modal>

<!-- Location Modal -->
<Modal show={activeSection === 'manage-location'} title={$t('settings.location_services')} on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-4">
    <!-- Permission status -->
    <div>
      <p class="text-sm font-medium mb-2">{$t('location.permission')}</p>
      {#if locationPermissionStatus === 'granted' || locationEnabled}
        <div class="flex items-center gap-2 text-sm text-green-600">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          {$t('location.enabled')}
        </div>
      {:else if locationPermissionStatus === 'requesting'}
        <div class="flex items-center gap-2 text-sm text-hazardo-lightGray">
          <div class="w-4 h-4 border-2 border-hazardo-accent border-t-transparent rounded-full animate-spin"></div>
          {$t('location.requesting')}
        </div>
      {:else}
        <button type="button" class="flex items-center gap-2 px-4 py-2 rounded-lg bg-hazardo-accent text-white text-sm font-medium" on:click={requestLocationPermission}>
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg>
          {$t('location.allow')}
        </button>
        {#if locationPermissionStatus === 'denied'}
          <p class="text-xs text-red-500 mt-1">{$t('location.denied')}</p>
        {/if}
      {/if}
    </div>

    <!-- Location override -->
    <div>
      <p class="text-sm font-medium mb-2">{$t('location.roll_dice')}</p>
      <p class="text-xs text-hazardo-lightGray mb-2">{$t('location.description')}</p>
      {#if locationOverride}
        <div class="flex items-center justify-between bg-hazardo-background/50 rounded-lg px-3 py-2 border border-hazardo-lightGray/30">
          <span class="text-sm">{locationOverride.split('|')[0]}</span>
          <button type="button" class="text-xs text-red-500 hover:text-red-700" on:click={clearLocationOverride}>{$t('location.remove')}</button>
        </div>
      {:else}
        <div class="flex items-center gap-2 text-sm text-hazardo-lightGray mb-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
          {$t('location.using_gps')}
        </div>
      {/if}
      <div class="flex gap-2 mt-2">
        <div class="flex-1">
          <input type="text" bind:value={locationSearch} placeholder={$t('location.search_city')} on:keydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); searchLocation(); } }} class="border rounded p-2 border-hazardo-lightGray focus:outline-hazardo-accent w-full bg-hazardo-surface text-hazardo-text" />
        </div>
        <button type="button" class="px-3 py-1 rounded bg-hazardo-accent text-white text-sm shrink-0 disabled:opacity-50" disabled={locationSearching || !locationSearch.trim()} on:click={searchLocation}>
          {locationSearching ? '...' : $t('location.search')}
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
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm text-hazardo-text" on:click={() => activeSection = ''}>{$t('settings.cancel')}</button>
      <button type="button" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm" on:click={saveLocationSettings}>{$t('settings.save')}</button>
    </div>
  </div>
</Modal>

<!-- Import Modal -->
<Modal show={activeSection === 'bulk-import'} title={$t('settings.bulk_import')} width="w-96" on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label={$t('import.format')} />
      <SelectDropdown options={exportFormatOptions} bind:selected={importFormat} placeholder="" />
    </div>
    <div class="flex flex-col">
      <FormLabel label={$t('import.file')} />
      <input type="file" accept={importAccept} class="text-sm" on:change={handleFileSelect} bind:this={importFileInput} />
    </div>
    {#if importFileContent}
      <div class="border border-hazardo-lightGray rounded p-2">
        <p class="text-xs text-hazardo-lightGray mb-1">{$t('export.preview')} ({importFileContent.length} chars)</p>
        <pre class="text-xs text-hazardo-text max-h-32 overflow-auto whitespace-pre-wrap">{importFileContent.slice(0, 500)}{importFileContent.length > 500 ? '...' : ''}</pre>
      </div>
    {/if}
    {#if importFormat !== 'zip'}
    <p class="text-xs text-hazardo-lightGray">{$t('import.paste')}</p>
    <textarea
      bind:value={importFileContent}
      placeholder={$t('import.placeholder')}
      class="border rounded p-2 border-hazardo-lightGray focus:outline-hazardo-accent w-full text-xs resize-none font-mono bg-hazardo-surface text-hazardo-text"
      rows="4"
    ></textarea>
    {/if}
    {#if importResult}
      <p class="text-sm {importResult.startsWith('Import error') ? 'text-red-500' : 'text-green-600'}">{importResult}</p>
    {/if}
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm text-hazardo-text" on:click={() => activeSection = ''}>{$t('settings.cancel')}</button>
      <button type="button" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm disabled:opacity-50" disabled={importLoading || !importFileContent.trim()} on:click={handleImport}>
        {importLoading ? $t('import.importing') : $t('import.import')}
      </button>
    </div>
  </div>
</Modal>

<!-- Export Modal -->
<Modal show={activeSection === 'bulk-export'} title={$t('settings.bulk_export')} width="w-96" on:close={() => activeSection = ''}>
  <div class="flex flex-col gap-4">
    <!-- User selection -->
    <div class="flex flex-col">
      <FormLabel label={$t('export.select_users')} />
      <label class="flex items-center gap-2 text-sm py-1">
        <input type="checkbox" bind:checked={exportAllUsers} class="w-4 h-4 accent-hazardo-accent" />
        {$t('export.all_users')}
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
        <FormLabel label={$t('export.select_categories')} />
        <label class="flex items-center gap-2 text-sm py-1">
          <input type="checkbox" bind:checked={exportAllCategories} class="w-4 h-4 accent-hazardo-accent" />
          {$t('export.all_categories')}
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
    <!-- Include picked items -->
    <div class="flex flex-col">
      <FormLabel label={$t('export.select_picks')} />
      <label class="flex items-center gap-2 text-sm py-1">
        <input type="checkbox" bind:checked={exportIncludePicks} class="w-4 h-4 accent-hazardo-accent" />
        {$t('export.include_picks')}
      </label>
      {#if exportIncludePicks && exportPicks.length > 0}
        <label class="flex items-center gap-2 text-sm py-1">
          <input type="checkbox" bind:checked={exportAllPicks} class="w-4 h-4 accent-hazardo-accent" />
          {$t('export.all_picks')}
        </label>
        {#if !exportAllPicks}
          <div class="max-h-32 overflow-y-auto pl-4">
            {#each exportPicks as pick}
              <label class="flex items-center gap-2 text-sm py-1">
                <input type="checkbox" bind:checked={exportPickSelection[pick.pick_id]} class="w-4 h-4 accent-hazardo-accent" />
                <Icon name={pick.category_icon} size={14} />
                <span class="truncate">{pick.item_name} — {pick.pick_date}</span>
              </label>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
    <!-- Format -->
    <div class="flex flex-col">
      <FormLabel label={$t('export.format')} />
      <SelectDropdown options={exportFormatOptions} bind:selected={exportFormat} placeholder="" />
    </div>
    <!-- Export button -->
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm text-hazardo-text" on:click={() => activeSection = ''}>{$t('settings.cancel')}</button>
      <button type="button" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm disabled:opacity-50" disabled={exportLoading || selectedExportUserIds.length === 0} on:click={handleExport}>
        {exportLoading ? $t('export.exporting') : $t('export.export')}
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
            <p class="text-xs text-hazardo-lightGray">{$t('export.result')}</p>
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
