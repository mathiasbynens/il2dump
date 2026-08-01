import { rollup } from 'rollup';
import nodeResolve from '@rollup/plugin-node-resolve';
import terser from '@rollup/plugin-terser';
import fs from 'node:fs/promises';
import path from 'node:path';
import { minify } from 'html-minifier-terser';

const build = async () => {
	// Rename the generated JS wrapper to use the .mjs extension.
	try {
		await fs.rename('web/pkg/il2dump-lib.js', 'web/pkg/il2dump-lib.mjs');
	} catch (e) {
		// Ignore error if file is already renamed or doesn't exist.
	}

	// Bundle app.mjs and its imports.
	const bundle = await rollup({
		input: 'web/app.mjs',
		plugins: [nodeResolve(), terser()],
	});

	const { output } = await bundle.generate({
		format: 'es',
	});

	const bundledJs = output[0].code;

	// Read index.html and style.css.
	let html = await fs.readFile('web/index.html', 'utf8');
	const css = await fs.readFile('web/style.css', 'utf8');

	// Inline CSS.
	html = html.replace(
		/<link\s+rel="stylesheet"\s+href="style\.css"\s*\/?>/i,
		`<style>${css}</style>`,
	);

	// Inline JS.
	html = html.replace(
		/<script\s+type="module"\s+src="app\.mjs"\s*><\/script>/i,
		`<script type="module">${bundledJs}</script>`,
	);

	// Minify HTML.
	const minifiedHtml = await minify(html, {
		collapseWhitespace: true,
		removeComments: true,
		minifyCSS: true,
		minifyJS: true,
	});

	// Ensure dist/ exists and write output.
	await fs.mkdir('dist', { recursive: true });
	await fs.writeFile('dist/index.html', minifiedHtml, 'utf8');

	// Copy and rename the Wasm file to the dist and web flat directories.
	await fs.copyFile('web/pkg/il2dump-lib_bg.wasm', 'dist/il2dump.wasm');
	await fs.copyFile('web/pkg/il2dump-lib_bg.wasm', 'web/il2dump.wasm');

	console.log('Build completed successfully: dist/index.html');
};

build().catch((err) => {
	console.error('Build failed:', err);
	process.exit(1);
});
