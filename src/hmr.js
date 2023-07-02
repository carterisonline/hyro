const socket = new WebSocket(`ws://${location.host}/hmr`);

const hmrIndexes = {};

// i apologize for the rest of this file

socket.addEventListener("message", async (event) => {
	if (typeof event.data === "string") {
		if (event.data !== "you up?") {
			const elements = document.querySelectorAll(`[hmr-path="${event.data}"]`);
			const indexes = new Uint32Array(elements.length);
			let i = 0;
			for (const element of elements) {
				if (element.tagName === "HTML") {
					socket.send("r");
					location.reload();
				}

				indexes[i] = parseInt(element.getAttribute("hmr-index"), 10);
				i++;

				element.removeAttribute("hmr-index");
			}

			hmrIndexes[event.data] = 0;
			socket.send("c");
			socket.send(indexes);
			for (const element of elements) {
				const response = await fetch(event.data).then((res) => res.text());
				element.outerHTML = response;
				if (response.includes("hx-")) {
					htmx.process(document.body);
				}
			}

			document
				.querySelectorAll(`[hmr-path="${event.data}"]:not([hmr-index])`)
				.forEach((element) => {
					element.setAttribute("hmr-index", hmrIndexes[event.data]++);
				});
		}
	} else if (event.data.size === 1) {
		document.querySelector(
			"head > :is(link[href='/main.css'], style#hmr)"
		).outerHTML = `<style id="hmr">\n${await fetch("/main.css").then((res) =>
			res.text()
		)}\n</style>`;
	}
});

document.addEventListener("DOMContentLoaded", () => {
	document.body.addEventListener("htmx:load", (event) => {
		let hmrPath;
		if (event.detail.elt.attributes.getNamedItem("hmr-path") !== null) {
			hmrPath = event.detail.elt.attributes.getNamedItem("hmr-path");
		} else {
			hmrPath =
				event.detail.elt.parentElement.attributes.getNamedItem("hmr-path");
		}
		if (!hmrIndexes[hmrPath.value]) {
			hmrIndexes[hmrPath.value] = 0;
		}

		document
			.querySelectorAll(`[hmr-path="${hmrPath.value}"]:not([hmr-index])`)
			.forEach((element) => {
				element.setAttribute("hmr-index", hmrIndexes[hmrPath.value]++);
			});
	});
});
