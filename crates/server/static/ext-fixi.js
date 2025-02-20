// Polling and multiple events
document.addEventListener("fx:init", (evt) => {
    let elt = evt.target;

    let interval = elt.getAttribute("ext-fx-poll-interval");
    if (interval) {
        setInterval(
            () => elt.dispatchEvent(new CustomEvent("poll")),
            parseInt(interval)
        );
    }

    let triggers = (elt.getAttribute("ext-fx-multi-trigger") ?? "").split(" ");
    triggers.forEach(
        (trigger) => elt.addEventListener(trigger, () => elt.dispatchEvent(new CustomEvent("multi")))
    );
})

// Error handling
document.addEventListener("fx:after", (evt) => {
    let cfg = evt.detail.cfg;
    if (!cfg.response.ok) {
        cfg.text = `Error: ${cfg.response.statusText}: ${cfg.text}`;
        cfg.target = document.querySelector("main");
        cfg.swap = "innerHTML"
    }
})

// Empty swaps
document.addEventListener("fx:config", (evt) => {
    let cfg = evt.detail.cfg;

    if (cfg.swap === "none") cfg.swap = () => { };
})