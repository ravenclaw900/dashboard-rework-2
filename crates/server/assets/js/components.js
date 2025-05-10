(() => {
    customElements.define("theme-switcher", class extends HTMLElement {
        connectedCallback() {
            let button = this.querySelector("button");
            this.meta = this.querySelector("meta[name='color-scheme']");
            this.darkIcon = this.querySelector("svg:has(use[href$='fa6-solid-moon'])");
            this.lightIcon = this.querySelector("svg:has(use[href$='fa6-solid-sun'])");
            this.isDark = localStorage.getItem("darkMode") === "true";

            this.toggle();

            button.addEventListener("click", () => {
                this.isDark = !this.isDark;
                this.toggle();
            })
        }

        toggle() {
            localStorage.setItem("darkMode", this.isDark);
            this.meta.content = this.isDark ? "dark" : "light";
            this.darkIcon.style.display = this.isDark ? "" : "none";
            this.lightIcon.style.display = this.isDark ? "none" : "";
        }
    });

    customElements.define("server-swap", class extends HTMLElement {
        connectedCallback() {
            let url = this.getAttribute("action") || window.location.href;
            const method = (this.getAttribute("method") || "GET").toUpperCase();
            const trigger = this.getAttribute("trigger") || "click";
            const targetAttr = this.getAttribute("target");
            const target = !targetAttr ? this : targetAttr === "none" ? null : document.querySelector(targetAttr);

            const swap = async () => {
                try {
                    const resp = await fetch(url, { method, headers: { "fx-request": "true" } });
                    const text = await resp.text();

                    if (!resp.ok)
                        throw new Error(`${resp.statusText}: ${text}`);

                    if (target)
                        target.outerHTML = text;
                } catch (err) {
                    document.querySelector("main").innerText = `Error: ${err.message}`;
                }
            };

            this.addEventListener(trigger, swap);

            setTimeout(() => this.dispatchEvent(new Event("delay")), 2000);
        }
    });

    customElements.define("web-terminal", class extends HTMLElement {
        connectedCallback() {
            const term = new Terminal();
            term.open(this);

            const socket = new WebSocket("/terminal/ws");
            socket.binaryType = "arraybuffer";

            socket.onmessage = (e) => term.write(new Uint8Array(e.data));

            term.onData((data) => socket.send(data));
        }
    });
})();
