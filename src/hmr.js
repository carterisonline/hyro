const socket = new WebSocket(`ws://${location.host}/hmr`);

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
			}

			socket.send("c");
			socket.send(indexes);
			for (const element of elements) {
				const response = await fetch(event.data).then((res) => res.text());
				element.outerHTML = response;
				if (response.includes("hx-")) {
					// @ts-ignore
					htmx.process(document.body);
				}
			}

			let j = 0;
			document
				.querySelectorAll(`[hmr-path="${event.data}"]:not([hmr-index])`)
				.forEach((element) => {
					element.setAttribute("hmr-index", indexes[j].toString());
					j++;
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

let hmrIndexes = {};
document.addEventListener("DOMContentLoaded", () => {
	document.body.addEventListener("htmx:load", (event) => {
		let hmrPath;
		// @ts-ignore
		if (event.detail.elt.attributes.getNamedItem("hmr-path") !== null) {
			// @ts-ignore
			hmrPath = event.detail.elt.attributes.getNamedItem("hmr-path");
		} else {
			hmrPath =
				// @ts-ignore
				event.detail.elt.parentElement.attributes.getNamedItem("hmr-path");
		}
		if (!hmrIndexes[hmrPath.value]) {
			hmrIndexes[hmrPath.value] = 0;
		}

		document
			.querySelectorAll(`[hmr-path="${hmrPath.value}"]:not([hmr-index])`)
			.forEach((element) => {
				element.setAttribute(
					"hmr-index",
					(hmrIndexes[hmrPath.value]++).toString()
				);
			});
	});
});
