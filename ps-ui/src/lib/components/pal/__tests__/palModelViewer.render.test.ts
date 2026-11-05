// @vitest-environment jsdom
import '$utils/__tests__/fixtures/animatePolyfill';
import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

type RendererRecord = { disposed: number; contextLost: number };

const { renderers, meshLibrary } = vi.hoisted(() => ({
	renderers: [] as { disposed: number; contextLost: number }[],
	meshLibrary: {
		requested: [] as string[],
		url: (key: string) => (key === 'missing' ? null : `/models/pals/${key}.glb`)
	}
}));

// jsdom has no GL context, so the renderer is counted, not run.
vi.mock('three', async (importOriginal) => {
	const actual = await importOriginal<Record<string, unknown>>();
	class FakeRenderer {
		record = { disposed: 0, contextLost: 0 };
		constructor() {
			renderers.push(this.record);
		}
		dispose() {
			this.record.disposed += 1;
		}
		forceContextLoss() {
			this.record.contextLost += 1;
		}
		setClearAlpha() {}
		setPixelRatio() {}
		setSize() {}
		render() {}
	}
	return { ...actual, WebGLRenderer: FakeRenderer };
});

vi.mock('$components/map/scene/pal/palMeshLibrary', () => ({
	palModelUrl: (key: string) => meshLibrary.url(key),
	requestPalMesh: (key: string) => {
		meshLibrary.requested.push(key);
		return null;
	},
	palMeshFailed: () => false,
	onPalMeshLoaded: () => () => {}
}));

const { default: PalModelViewer } = await import('../PalModelViewer.svelte');

function canvas(): HTMLElement | null {
	return document.querySelector('canvas');
}

beforeEach(() => {
	renderers.length = 0;
	meshLibrary.requested.length = 0;
	// jsdom has no ResizeObserver, and the viewer observes its canvas.
	vi.stubGlobal(
		'ResizeObserver',
		class {
			observe() {}
			disconnect() {}
		}
	);
});

describe('PalModelViewer', () => {
	it('mounts the model straight away', async () => {
		render(PalModelViewer, { props: { characterKey: 'testpal' } });
		await tick();

		expect(canvas()).not.toBeNull();
		expect(renderers).toHaveLength(1);
		expect(meshLibrary.requested).toEqual(['testpal']);
	});

	it('gives up its GL context when it goes away', async () => {
		const { unmount } = render(PalModelViewer, { props: { characterKey: 'testpal' } });
		await tick();

		unmount();
		await tick();

		expect(renderers[0]).toMatchObject<RendererRecord>({ disposed: 1, contextLost: 1 });
	});

	it('asks again for a different pal', async () => {
		const { rerender } = render(PalModelViewer, { props: { characterKey: 'testpal' } });
		await tick();

		await rerender({ characterKey: 'otherpal' });
		await tick();

		expect(canvas()).not.toBeNull();
		expect(meshLibrary.requested).toEqual(['testpal', 'otherpal']);
	});

	it('shows the fallback for a pal with no model', async () => {
		render(PalModelViewer, { props: { characterKey: 'missing' } });
		await tick();

		expect(canvas()).toBeNull();
		expect(renderers.every((renderer) => renderer.disposed === 1)).toBe(true);
	});
});
