<script lang="ts">
  export let label: string;
  export let htmlFor: string = '';
</script>

<label for={htmlFor}>{label}</label>
