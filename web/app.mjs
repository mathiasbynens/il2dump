import init, { dump_il2cpp_wasm } from './pkg/il2dump-lib.mjs';

let execFile = null;
let metaFile = null;
let wasmInitialized = false;

const execZone = document.getElementById('exec-zone');
const metaZone = document.getElementById('meta-zone');
const execInput = document.getElementById('exec-file');
const metaInput = document.getElementById('meta-file');
const dumpBtn = document.getElementById('dump-btn');
const statusContainer = document.getElementById('status-container');
const statusText = document.getElementById('status-text');

// Initialize the WebAssembly module.
async function start() {
	try {
		await init('il2dump.wasm');
		wasmInitialized = true;
		updateStatus('WebAssembly module initialized. Ready for files.');
		checkReady();
	} catch (err) {
		updateStatus(`Failed to initialize WebAssembly: ${err}`, true);
	}
}

function updateStatus(text, isError = false) {
	statusText.textContent = text;
	statusText.classList.toggle('error', isError);
	if (isError) {
		statusContainer.classList.remove('loading');
	}
}

// Set up the drag and drop handlers.
function setupDragZone(zone, input, onFileSelected) {
	zone.addEventListener('dragover', (event) => {
		event.preventDefault();
		zone.classList.add('dragover');
	});

	zone.addEventListener('dragleave', () => {
		zone.classList.remove('dragover');
	});

	zone.addEventListener('drop', (event) => {
		event.preventDefault();
		zone.classList.remove('dragover');
		if (event.dataTransfer.files.length > 0) {
			handleFile(event.dataTransfer.files[0], zone, onFileSelected);
		}
	});

	input.addEventListener('change', () => {
		if (input.files.length > 0) {
			handleFile(input.files[0], zone, onFileSelected);
		}
	});
}

function handleFile(file, zone, onFileSelected) {
	zone.classList.add('loaded');
	const textNode = zone.querySelector('.drop-zone-text');
	textNode.textContent = `${file.name} (${formatBytes(file.size)})`;
	onFileSelected(file);
	checkReady();
}

function formatBytes(bytes) {
	if (bytes === 0) return '0 bytes';
	const k = 1000;
	const sizes = ['bytes', 'kB', 'MB', 'GB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

setupDragZone(execZone, execInput, (file) => {
	execFile = file;
});
setupDragZone(metaZone, metaInput, (file) => {
	metaFile = file;
});

function checkReady() {
	dumpBtn.disabled = !(execFile && metaFile && wasmInitialized);
}

// Helper to read a file as an array buffer.
function readFileAsArrayBuffer(file) {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => resolve(reader.result);
		reader.onerror = () => reject(reader.error);
		reader.readAsArrayBuffer(file);
	});
}

// Helper to trigger a file download in the browser.
function triggerDownload(content, filename, mimeType) {
	const blob = new Blob([content], { type: mimeType });
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = filename;
	document.body.append(a);
	a.click();
	a.remove();
	URL.revokeObjectURL(url);
}

dumpBtn.addEventListener('click', async () => {
	if (!execFile || !metaFile) return;

	dumpBtn.disabled = true;
	statusContainer.classList.remove('hidden');
	statusContainer.classList.add('loading');

	try {
		updateStatus('Reading files into memory…');
		const [execBuffer, metaBuffer] = await Promise.all([
			readFileAsArrayBuffer(execFile),
			readFileAsArrayBuffer(metaFile),
		]);

		updateStatus(
			'Running decompiler in WebAssembly. This might take a few seconds…',
		);
		// Yield to the browser UI thread.
		await new Promise((r) => setTimeout(r, 100));

		const execBytes = new Uint8Array(execBuffer);
		const metaBytes = new Uint8Array(metaBuffer);

		const startTime = performance.now();
		const results = dump_il2cpp_wasm(execBytes, metaBytes);
		const duration = ((performance.now() - startTime) / 1000).toFixed(2);

		updateStatus(`Decompilation completed in ${duration}s! Downloading files…`);

		const dumpCs = results[0];
		const scriptJson = results[1];

		triggerDownload(dumpCs, 'dump.cs', 'text/plain');
		triggerDownload(scriptJson, 'script.json', 'application/json');

		statusContainer.classList.remove('loading');
	} catch (err) {
		updateStatus(`Error: ${err}`, true);
	} finally {
		dumpBtn.disabled = false;
	}
});

start();
