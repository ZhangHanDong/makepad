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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="preface.html">前言</a></li><li class="chapter-item expanded affix "><li class="part-title">第一部分：协议核心</li><li class="chapter-item expanded "><a href="01-goals-and-boundaries.html"><strong aria-hidden="true">1.</strong> 目标与边界</a></li><li class="chapter-item expanded "><a href="02-domain-model.html"><strong aria-hidden="true">2.</strong> 通用协议内核</a></li><li class="chapter-item expanded "><a href="03-runtime-modes.html"><strong aria-hidden="true">3.</strong> 传输绑定</a></li><li class="chapter-item expanded "><a href="04-envelope-and-scope.html"><strong aria-hidden="true">4.</strong> Envelope 与 Scope</a></li><li class="chapter-item expanded "><a href="05-data-model.html"><strong aria-hidden="true">5.</strong> 数据模型</a></li><li class="chapter-item expanded "><a href="05-state-model.html"><strong aria-hidden="true">6.</strong> 状态模型</a></li><li class="chapter-item expanded "><a href="06-rendering-and-templates.html"><strong aria-hidden="true">7.</strong> 渲染与模板</a></li><li class="chapter-item expanded "><a href="07-actions.html"><strong aria-hidden="true">8.</strong> Action 协议</a></li><li class="chapter-item expanded "><a href="08-a2ui-compatibility.html"><strong aria-hidden="true">9.</strong> A2UI 兼容 Profile</a></li><li class="chapter-item expanded affix "><li class="part-title">第二部分：应用场景</li><li class="chapter-item expanded "><a href="08-aichat-profile.html"><strong aria-hidden="true">10.</strong> aichat 应用场景</a></li><li class="chapter-item expanded "><a href="09-robrix-profile.html"><strong aria-hidden="true">11.</strong> Robrix2 应用场景</a></li><li class="chapter-item expanded "><a href="10-mission-room-profile.html"><strong aria-hidden="true">12.</strong> Mission Room 应用场景</a></li><li class="chapter-item expanded affix "><li class="part-title">第三部分：工程约束</li><li class="chapter-item expanded "><a href="11-safety-model.html"><strong aria-hidden="true">13.</strong> 安全模型</a></li><li class="chapter-item expanded "><a href="12-versioning-validation.html"><strong aria-hidden="true">14.</strong> 版本、兼容与验证</a></li><li class="chapter-item expanded "><a href="13-non-goals-roadmap.html"><strong aria-hidden="true">15.</strong> 非目标与后续扩展</a></li><li class="chapter-item expanded affix "><a href="appendix-json.html">附录：JSON 示例</a></li></ol>';
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
