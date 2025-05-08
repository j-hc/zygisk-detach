import { exec, spawn, toast } from 'kernelsu';

const template = document.getElementById('app-template').content;
const appsList = document.getElementById('apps-list');
const detachButton = document.getElementById('detach');
const searchInput = document.getElementById('search');

async function run(cmd) {
	const LOG_DIR = "/sdcard/zygisk-detach.log";
	try {
		const { errno, stdout, stderr } = await exec(cmd);
		if (errno != 0) {
			const firstLineError = stderr.split('\n')[0] || "Command failed";
			toast(`Error: ${firstLineError} (see log)`);
			console.error(`Command '${cmd}' failed. Full stderr:`, stderr);
			const fullLog = `\
CMD: ${cmd}

STDERR:
${stderr}

STDOUT:
${stdout}`.replaceAll("'", "\'");
			await exec(`echo '${fullLog}' >> '${LOG_DIR}'`);
			toast(`Full logs in '${LOG_DIR}'`);
			return undefined;
		}
		return stdout.trim();
	} catch (e) {
		toast("Critical error executing command.");
		console.error("Critical error in run function:", e);
		return undefined;
	}
}

function sortChecked() {
	if (!appsList.children.length) return;
	[...appsList.children]
		.sort((a, b) => {
			const aChecked = a.querySelector('.checkbox').checked;
			const bChecked = b.querySelector('.checkbox').checked;
			if (aChecked === bChecked) {
				return a.querySelector('.name').textContent.localeCompare(b.querySelector('.name').textContent);
			}
			return aChecked ? -1 : 1;
		})
		.forEach(node => appsList.appendChild(node));
}

const detach_list = new Set();

function populateApp(name, checked) {
	const node = document.importNode(template, true);
	const appNameElement = node.querySelector('.name');
	appNameElement.textContent = name;
	const checkbox = node.querySelector('.checkbox');
	checkbox.checked = checked;
	checkbox.setAttribute('aria-label', `Select ${name}`);

	if (checked) {
		detach_list.add(name);
	}

	checkbox.addEventListener('change', () => {
		if (checkbox.checked) {
			detach_list.add(name);
		} else {
			detach_list.delete(name);
		}
	});
	appsList.appendChild(node);
}

function setLoadingState(isLoading) {
	detachButton.disabled = isLoading;
	searchInput.disabled = isLoading;
	if (isLoading) {
		detachButton.innerHTML = '<span class="spinner"></span>';
		appsList.innerHTML = '<p style="text-align:center; padding:20px;">Loading apps...</p>';
	} else {
		detachButton.innerHTML = '&#x2714;';
		if (!appsList.hasChildNodes() || appsList.textContent.trim() === "Loading apps...") {
			appsList.innerHTML = '<p style="text-align:center; padding:20px;">No apps found or failed to load.</p>';
		}
	}
}

function debounce(func, delay) {
  let timeout;
  return function(...args) {
    clearTimeout(timeout);
    timeout = setTimeout(() => func.apply(this, args), delay);
  };
}

async function main() {
	setLoadingState(true);

	const pkgsOutput = await run("pm list packages");
	if (pkgsOutput === undefined) {
		toast("Failed to list packages.");
		setLoadingState(false);
		return;
	}

	appsList.innerHTML = '';

	const detached_list_out = await run("/data/adb/modules/zygisk-detach/detach list");

	if (detached_list_out === undefined) {
		toast("Warning: Could not get detached list. Proceeding as if none are detached.");
	}
	const detached = detached_list_out ? detached_list_out.split('\n').filter(pkg => pkg.trim() !== '') : [];
	const uninstalledButDetached = detached ? [...detached] : [];

	const packages = pkgsOutput.split('\n')
		.map(line => line.startsWith('package:') ? line.substring(8).trim() : line.trim())
		.filter(pkg => pkg);

	if (packages.length === 0) {
		toast("No packages found.");
		setLoadingState(false);
		return;
	}

	for (const pkg of packages) {
		if (!pkg) continue;
		const isDetached = detached.includes(pkg);
		populateApp(pkg, isDetached);
		if (isDetached) {
			const index = uninstalledButDetached.indexOf(pkg);
			if (index > -1) uninstalledButDetached.splice(index, 1);
		}
	}

	if (uninstalledButDetached.length > 0) {
		toast(`Warning: ${uninstalledButDetached.length} app(s) in detach list but not installed. They will be kept in the list.`);
		for (const pkg of uninstalledButDetached) {
			populateApp(pkg, true);
		}
	}

	if (appsList.children.length === 0) {
		appsList.innerHTML = '<p style="text-align:center; padding:20px;">No applications to display.</p>';
	}

	sortChecked();
	setLoadingState(false);

	const debouncedSearch = debounce((searchValue) => {
		if (appsList.children.length === 0 && !searchValue) {
            sortChecked();
            return;
        }
		let found = 0;
		[...appsList.children].forEach(node => {
			const appName = node.querySelector('.name').textContent.toLowerCase();
			if (appName.includes(searchValue)) {
				node.style.display = '';
				found++;
			} else {
				node.style.display = 'none';
			}
		});

        if (searchValue && found === 0) {
            if (!document.getElementById('no-results-message')) {
                const noResultsMessage = document.createElement('p');
                noResultsMessage.id = 'no-results-message';
                noResultsMessage.textContent = 'No apps match your search.';
                noResultsMessage.style.textAlign = 'center';
                noResultsMessage.style.padding = '20px';
                appsList.appendChild(noResultsMessage);
            }
        } else {
            const noResultsMessage = document.getElementById('no-results-message');
            if (noResultsMessage) {
                noResultsMessage.remove();
            }
            if (!searchValue) sortChecked();
        }
	}, 300);

	searchInput.addEventListener('input', (e) => {
		const searchVal = e.target.value.toLowerCase().trim();
		debouncedSearch(searchVal);
	});

	detachButton.addEventListener('click', async () => {
		setLoadingState(true);
		let result;
		if (detach_list.size === 0) {
			result = await run("/data/adb/modules/zygisk-detach/detach reset");
			if (result !== undefined) toast("Detach list reset.");
		} else {
			const detach_arg = Array.from(detach_list).join(' ');
			result = await run(`/data/adb/modules/zygisk-detach/detach detachall ${detach_arg}`);
			if (result !== undefined) toast("Detach list updated.");
		}

		setLoadingState(false);
	});
}

const style = document.createElement('style');
style.innerHTML = `
.spinner {
  display: inline-block;
  width: 20px;
  height: 20px;
  border: 3px solid rgba(255,255,255,.3);
  border-radius: 50%;
  border-top-color: #fff;
  animation: spin 1s ease-in-out infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
`;
document.head.appendChild(style);

main().catch(err => {
	console.error("Error in main execution:", err);
	toast("An unexpected error occurred. Check console.");
	setLoadingState(false);
});
