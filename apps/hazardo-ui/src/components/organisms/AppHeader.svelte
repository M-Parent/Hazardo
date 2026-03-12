<script lang="ts">
  import LogoTxt from '../atoms/LogoTxt.svelte';
  import UserDropdown from '../molecules/UserDropdown.svelte';
</script>

<header class="flex items-center justify-between px-3">
  <LogoTxt />
  <div class="mt-8">
    <UserDropdown />
  </div>
</header>
