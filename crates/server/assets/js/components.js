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
            if (this.isDark) {
                this.meta.content = "dark";
                this.darkIcon.style.display = "";
                this.lightIcon.style.display = "none";
            } else {
                this.meta.content = "light";
                this.darkIcon.style.display = "none";
                this.lightIcon.style.display = "";
            }
        }
    });
})();
