import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	optimizeDeps: {
		exclude: ['nostring-heir-ffi']
	},
	server: {
		fs: {
			allow: ['src/lib/wasm']
		}
	}
});
