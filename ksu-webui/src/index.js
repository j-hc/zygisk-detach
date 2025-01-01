import { exec, toast } from 'kernelsu';

const template = document.getElementById('app-template').content;
const appsList = document.getElementById('apps-list');

async function run(cmd) {
	const { errno, stdout, stderr } = await exec(cmd);
	if (errno != 0) {
		toast(`stderr: ${stderr}`);
		return undefined;
	} else {
		return stdout;
	}
}

function sortChecked() {
	[...appsList.children]
		.sort((a, _b) => a.querySelector('.checkbox').checked ? -1 : 1)
		.forEach(node => appsList.appendChild(node));
}

const detach_list = [];

function populateApp(name, checked) {
	const node = document.importNode(template, true);
	node.querySelector('.name').textContent = name;
	const checkbox = node.querySelector('.checkbox');
	checkbox.checked = checked;
	if (checked) detach_list.push(name);
	checkbox.addEventListener('change', () => {
		if (checkbox.checked) {
			detach_list.push(name);
		} else {
			const i = detach_list.indexOf(name);
			if (i !== -1) detach_list.splice(i, 1);
		}
	});
	appsList.appendChild(node);
}

async function main() {
	const pkgs = await run("pm list packages");
	if (pkgs === undefined) return;

	const detached_list_out = await run("/data/adb/modules/zygisk-detach/detach list");
	if (detached_list_out === undefined) return;
	const detached = detached_list_out ? detached_list_out.split('\n') : [];
	const uninstalled = detached ? [...detached] : [];
	for (const pkg of pkgs.split('\n').map((line) => line.split(':')[1])) {
		const incls = detached.includes(pkg);
		populateApp(pkg, incls);
		if (incls) {
			const index = uninstalled.indexOf(pkg);
			if (index > -1) uninstalled.splice(index, 1);
		}
	}
	for (const pkg of uninstalled) populateApp(pkg, true);
	sortChecked();

	document.getElementById("search").addEventListener('input', (e) => {
		if (!e.target.value) {
			sortChecked();
			return;
		};
		const searchVal = e.target.value.toLowerCase();
		[...appsList.children]
			.sort((a, _b) => a.querySelector('.name').textContent.toLowerCase().includes(searchVal) ? -1 : 1)
			.forEach(node => appsList.appendChild(node));
	});

	document.getElementById("detach").addEventListener('click', (e) => {
		const detach_arg = detach_list.join(' ');
		run(`/data/adb/modules/zygisk-detach/detach detachall "${detach_arg}"`).then((out) => toast(out));
	});
}

await main();
