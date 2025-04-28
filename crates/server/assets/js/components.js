(() => {
    class ThemeSwitcher extends HTMLElement {
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
    }

    customElements.define("theme-switcher", ThemeSwitcher);

    class ServerSwap extends HTMLElement {
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

                    const doSwap = () => {
                        if (target)
                            target.outerHTML = text;
                    };

                    if (document.startViewTransition) {
                        document.startViewTransition(doSwap);
                    } else {
                        doSwap();
                    };
                } catch (err) {
                    document.querySelector("main").innerText = `Error: ${err.message}`;
                }
            };

            if (trigger === "delay")
                setTimeout(swap, 2000);
            else
                this.addEventListener(trigger, swap);
        }
    }

    customElements.define("server-swap", ServerSwap);
})();
