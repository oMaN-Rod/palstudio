<script lang="ts">
	import Icon from '$lib/components/ui/icons/Icon.svelte';
	import { CENTRIX } from '$lib/partners/centrixnodes';
	import { openExternalLink } from '$lib/utils/externalLink';
	import * as m from '$i18n/messages';
	import CentrixLogo from './CentrixLogo.svelte';

	let copied = $state(false);

	async function copyCode() {
		await navigator.clipboard?.writeText(CENTRIX.code).catch(() => {});
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}
</script>

<div
	class="border-primary-500/30 bg-primary-500/10 not-prose flex flex-col gap-3 rounded-md border p-4 text-sm"
>
	<CentrixLogo class="w-40" />
	<p>{m.about_partner_offer({ code: CENTRIX.code, discount: CENTRIX.discount })}</p>
	<div class="text-surface-300 flex flex-wrap items-center gap-2">
		<span
			class="border-primary-500/40 bg-surface-900 text-surface-50 rounded-sm border border-dashed px-2 py-0.5 font-mono font-medium"
			>{CENTRIX.code}</span
		>
		<button
			type="button"
			class="text-surface-400 hover:text-primary-400 inline-flex min-h-8 items-center gap-1 text-xs transition-colors"
			onclick={copyCode}
		>
			<Icon icon={copied ? 'tabler:check' : 'tabler:copy'} size={14} />
			{copied ? m.signal_pairing_copied() : m.copy()}
		</button>
	</div>
	<a
		href={CENTRIX.url}
		target="_blank"
		rel="noopener noreferrer"
		class="text-primary-400 hover:text-primary-300 inline-flex w-fit items-center gap-1 font-medium"
		onclick={(event) => openExternalLink(event, CENTRIX.url)}
	>
		{CENTRIX.name}
		<Icon icon="tabler:external-link" size={14} />
	</a>
	<p class="text-surface-400 text-xs">{m.about_partner_disclosure()}</p>
</div>
