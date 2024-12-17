// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="introduction.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="configuration.html"><strong aria-hidden="true">2.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="commands/index.html"><strong aria-hidden="true">3.</strong> Commands</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="commands/init.html"><strong aria-hidden="true">3.1.</strong> init</a></li><li class="chapter-item expanded "><a href="commands/new.html"><strong aria-hidden="true">3.2.</strong> new</a></li><li class="chapter-item expanded "><a href="commands/edit.html"><strong aria-hidden="true">3.3.</strong> edit</a></li><li class="chapter-item expanded "><a href="commands/link.html"><strong aria-hidden="true">3.4.</strong> link</a></li><li class="chapter-item expanded "><a href="commands/list.html"><strong aria-hidden="true">3.5.</strong> list</a></li><li class="chapter-item expanded "><a href="commands/config.html"><strong aria-hidden="true">3.6.</strong> config</a></li><li class="chapter-item expanded "><a href="commands/generate.html"><strong aria-hidden="true">3.7.</strong> generate</a></li></ol></li><li class="chapter-item expanded "><a href="caveats.html"><strong aria-hidden="true">4.</strong> Caveats</a></li><li class="chapter-item expanded "><a href="resources.html"><strong aria-hidden="true">5.</strong> Resources</a></li><li class="chapter-item expanded "><a href="contributing.html"><strong aria-hidden="true">6.</strong> Contributing</a></li><li class="chapter-item expanded "><a href="license.html"><strong aria-hidden="true">7.</strong> License</a></li><li class="chapter-item expanded "><a href="adrs.html"><strong aria-hidden="true">8.</strong> Project ADRs</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
