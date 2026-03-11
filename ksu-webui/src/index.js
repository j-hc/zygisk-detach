import { exec, toast } from 'kernelsu';

const template = document.getElementById('app-template').content;
const appsList = document.getElementById('apps-list');

async function run(cmd) {
	const LOG_DIR = "/sdcard/zygisk-detach.log";
	const { errno, stdout, stderr } = await exec(cmd);
	if (errno != 0) {
		toast(`Command '${cmd}' failed.`);
		toast(stderr);
		// this is not properly escaped, whatever
		const fullLog = `\
CMD: ${cmd}

STDERR:
${stderr}

STDOUT:
${stdout}`.replaceAll("'", "\\'");
		exec(`echo '${fullLog}' > '${LOG_DIR}'`).then(() => {
			toast(`Full logs are saved in '${LOG_DIR}'`);
		});
		return undefined;
	} else {
		return stdout;
	}
}

function sortChecked() {
	[...appsList.children]
		.sort((a, _b) => a.querySelector('.checkbox').checked ? -1 : 1)
		.forEach((node, index) => {
			node.style.animationDelay = `${index * 0.02}s`;
			appsList.appendChild(node);
		});
}

function generateAppLabel(pkg) {
	let parts = pkg.split('.');
	// Remove common reverse-DNS prefixes and generic words
	parts = parts.filter(p => !['com', 'org', 'net', 'android', 'app', 'vending'].includes(p));
	if (parts.length === 0) return pkg;
	
	// Pick the most relevant part (usually the last or longest)
	let label = parts[parts.length - 1];
	
	// Add space before capital letters (camelCase splitting) and capitalize first letter
	return label
		.replace(/([A-Z])/g, ' $1')
		.replace(/^./, str => str.toUpperCase())
		.trim();
}

const detach_list = new Set();

function populateApp(pkg, checked) {
	const node = document.importNode(template, true);
	
	const label = generateAppLabel(pkg);
	node.querySelector('.app-label').textContent = label;
	node.querySelector('.app-package').textContent = pkg;
	
	const checkbox = node.querySelector('.checkbox');
	checkbox.checked = checked;
	
	if (checked) detach_list.add(pkg);
	
	checkbox.addEventListener('change', () => {
		if (checkbox.checked) {
			detach_list.add(pkg);
		} else {
			detach_list.delete(pkg);
		}
	});
	appsList.appendChild(node);
}

async function main() {
	const pkgsOut = await run("pm list packages");
	if (pkgsOut === undefined) return;

	const detached_list_out = await run("/data/adb/modules/zygisk-detach/detach list");
	if (detached_list_out === undefined) return;
	
	const detached = detached_list_out ? detached_list_out.split('\n').filter(Boolean) : [];
	const uninstalled = new Set(detached);
	
	// Parse package list safely
	const allPackages = pkgsOut.split('\n')
		.map(line => line.split(':')[1])
		.filter(Boolean); // Filter out empty strings

	for (const pkg of allPackages) {
		const isDetached = detached.includes(pkg);
		populateApp(pkg, isDetached);
		if (isDetached) {
			uninstalled.delete(pkg);
		}
	}
	
	// Remaining uninstalled packages that were in detach list
	for (const pkg of uninstalled) {
		populateApp(pkg, true);
	}
	
	sortChecked();

	document.getElementById("search").addEventListener('input', (e) => {
		if (!e.target.value) {
			sortChecked();
			return;
		}
		const searchVal = e.target.value.toLowerCase();
		// Replicate exactly the same sorting behavior: matched items moved up
		[...appsList.children]
			.sort((a, _b) => {
				const matches = a.innerText.toLowerCase().includes(searchVal);
				return matches ? -1 : 1;
			})
			.forEach(node => appsList.appendChild(node));
	});

	document.getElementById("detach").addEventListener('click', (e) => {
		if (detach_list.size === 0) {
			run("/data/adb/modules/zygisk-detach/detach reset").then(() => toast('Reset successful'));
		} else {
			const detach_arg = Array.from(detach_list).join(' ');
			run(`/data/adb/modules/zygisk-detach/detach detachall ${detach_arg}`).then((out) => toast(out || 'Changes applied'));
		}
	});
}

await main();
