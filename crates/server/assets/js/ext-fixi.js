(() => {
    document.addEventListener("fx:after", (evt) => {
        let cfg = evt.detail.cfg;

        // Error handling
        if (!cfg.response.ok) {
            cfg.text = `Error: ${cfg.response.statusText}: ${cfg.text}`;
            cfg.target = document.querySelector("main");
            cfg.swap = "innerHTML";
        }
    })

    // Polling
    setInterval(() => document.querySelectorAll('[fx-trigger="poll"]').forEach(
        (el) => el.dispatchEvent(new Event("poll"))
    ), 2000);
})();
